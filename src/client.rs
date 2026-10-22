//! HTTP transport for the Visor personal API: authentication, retries, and
//! the error envelope.

use std::fmt;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use reqwest::blocking::{Client as Http, Response};
use reqwest::header::{ACCEPT, HeaderMap, RETRY_AFTER};
use reqwest::{Method, StatusCode};
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::error::{ApiError, Error, Result};

pub const DEFAULT_API_URL: &str = "https://api.visorfinance.app";
const USER_AGENT: &str = concat!("visor-cli/", env!("CARGO_PKG_VERSION"));

/// How the client retries `429` and `5xx` responses and network errors.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_retries: u32,
    /// The first backoff for a `5xx` or a network error; it doubles per attempt.
    pub base_delay: Duration,
    /// A `429` with a longer `retry-after` fails instead of waiting.
    pub max_wait: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_millis(500),
            max_wait: Duration::from_secs(30),
        }
    }
}

/// Supplies the bearer token, and a new one when the API says it expired.
pub trait TokenSource: fmt::Debug + Send + Sync {
    /// The bearer to send now.
    fn bearer(&self) -> Result<String>;

    /// Called after the API answered `TOKEN_EXPIRED` for `rejected`. Returns
    /// the bearer to retry with, or `None` when this source cannot refresh.
    fn refresh(&self, rejected: &str) -> Result<Option<String>>;
}

/// A token that never changes: a personal API token or `VISOR_TOKEN`.
#[derive(Debug)]
struct StaticToken(String);

impl TokenSource for StaticToken {
    fn bearer(&self) -> Result<String> {
        Ok(self.0.clone())
    }

    fn refresh(&self, _rejected: &str) -> Result<Option<String>> {
        Ok(None)
    }
}

#[derive(Debug, Clone)]
pub struct Client {
    http: Http,
    base_url: String,
    token: Arc<dyn TokenSource>,
    retry: RetryPolicy,
}

/// A successful response body and the headers the CLI reads.
#[derive(Debug)]
pub struct Reply {
    pub body: Value,
    pub headers: HeaderMap,
}

impl Client {
    pub fn new(base_url: &str, token: impl Into<String>) -> Result<Self> {
        Self::with_source(base_url, Arc::new(StaticToken(token.into())))
    }

    pub fn with_source(base_url: &str, token: Arc<dyn TokenSource>) -> Result<Self> {
        Ok(Self {
            http: http()?,
            base_url: base_url.trim_end_matches('/').to_owned(),
            token,
            retry: RetryPolicy::default(),
        })
    }

    #[must_use]
    pub fn with_retry(mut self, retry: RetryPolicy) -> Self {
        self.retry = retry;
        self
    }

    pub fn get_json(&self, path: &str, query: &[(&str, String)]) -> Result<Value> {
        self.send(&Method::GET, path, query, None, None)
            .map(|reply| reply.body)
    }

    pub fn get<T: DeserializeOwned>(&self, path: &str, query: &[(&str, String)]) -> Result<T> {
        decode(self.get_json(path, query)?)
    }

    pub fn post_json(&self, path: &str, body: &Value, idempotency_key: &str) -> Result<Reply> {
        self.send(&Method::POST, path, &[], Some(body), Some(idempotency_key))
    }

    pub fn patch_json(&self, path: &str, body: &Value) -> Result<Reply> {
        self.send(&Method::PATCH, path, &[], Some(body), None)
    }

    pub fn delete(&self, path: &str) -> Result<()> {
        self.send(&Method::DELETE, path, &[], None, None).map(drop)
    }

    fn send(
        &self,
        method: &Method,
        path: &str,
        query: &[(&str, String)],
        body: Option<&Value>,
        idempotency_key: Option<&str>,
    ) -> Result<Reply> {
        // A create is safe to repeat because it carries an Idempotency-Key, and
        // an edit sets the same fields again. A repeated delete could answer
        // 404 for a delete that worked, so only a 429 (never processed) retries it.
        let retries_server_errors = *method != Method::DELETE;
        let mut attempt = 0;
        let mut refreshed = false;

        loop {
            let bearer = self.token.bearer()?;
            let mut request = self
                .http
                .request(method.clone(), format!("{}{path}", self.base_url))
                .bearer_auth(&bearer)
                .header(ACCEPT, "application/json")
                .query(query);
            if let Some(body) = body {
                request = request.json(body);
            }
            if let Some(key) = idempotency_key {
                request = request.header("Idempotency-Key", key);
            }

            let can_retry = attempt < self.retry.max_retries;
            match request.send() {
                Ok(response) if response.status().is_success() => return reply(response),
                Ok(response) if response.status() == StatusCode::UNAUTHORIZED => {
                    // An expired access token is refreshed once and the request
                    // sent again; the API never processed the rejected one.
                    let error = api_error(response);
                    if refreshed || error.api_code() != Some("TOKEN_EXPIRED") {
                        return Err(error);
                    }
                    match self.token.refresh(&bearer)? {
                        Some(_) => refreshed = true,
                        None => return Err(error),
                    }
                    continue;
                }
                Ok(response) => {
                    let status = response.status();
                    let wait = if status == StatusCode::TOO_MANY_REQUESTS {
                        retry_after(response.headers()).filter(|wait| *wait <= self.retry.max_wait)
                    } else if status.is_server_error() && retries_server_errors {
                        Some(self.backoff(attempt))
                    } else {
                        None
                    };

                    match wait {
                        Some(wait) if can_retry => thread::sleep(wait),
                        _ => return Err(api_error(response)),
                    }
                }
                Err(error) if can_retry && retries_server_errors && is_transient(&error) => {
                    thread::sleep(self.backoff(attempt));
                }
                Err(error) => return Err(Error::Network(error)),
            }
            attempt += 1;
        }
    }

    fn backoff(&self, attempt: u32) -> Duration {
        self.retry.base_delay.saturating_mul(2u32.saturating_pow(attempt))
    }
}

/// The HTTP client every request uses, API or OAuth.
pub fn http() -> Result<Http> {
    http_with_timeout(Duration::from_secs(60))
}

/// An HTTP client that gives up after `timeout`, for optional checks.
pub fn http_with_timeout(timeout: Duration) -> Result<Http> {
    Http::builder()
        .user_agent(USER_AGENT)
        .connect_timeout(Duration::from_secs(10).min(timeout))
        .timeout(timeout)
        .build()
        .map_err(Error::Network)
}

fn is_transient(error: &reqwest::Error) -> bool {
    error.is_connect() || error.is_timeout() || error.is_request()
}

fn retry_after(headers: &HeaderMap) -> Option<Duration> {
    headers
        .get(RETRY_AFTER)?
        .to_str()
        .ok()?
        .trim()
        .parse()
        .ok()
        .map(Duration::from_secs)
}

fn request_id(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned)
}

fn reply(response: Response) -> Result<Reply> {
    let headers = response.headers().clone();
    let text = response.text().map_err(Error::Network)?;
    let body = if text.trim().is_empty() {
        Value::Null
    } else {
        serde_json::from_str(&text).map_err(|error| Error::Decode(error.to_string()))?
    };
    Ok(Reply { body, headers })
}

fn api_error(response: Response) -> Error {
    let status = response.status().as_u16();
    let request_id = request_id(response.headers());
    let body = response.text().unwrap_or_default();
    Error::Api(ApiError::from_body(status, &body, request_id))
}

pub fn decode<T: DeserializeOwned>(value: Value) -> Result<T> {
    serde_json::from_value(value).map_err(|error| Error::Decode(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn client(server: &mockito::Server) -> Client {
        Client::new(&server.url(), "visor_pat_test_secret")
            .unwrap()
            .with_retry(RetryPolicy {
                max_retries: 2,
                base_delay: Duration::ZERO,
                max_wait: Duration::from_secs(1),
            })
    }

    #[test]
    fn sends_the_bearer_token_and_accept_header() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/v1/me")
            .match_header("authorization", "Bearer visor_pat_test_secret")
            .match_header("accept", "application/json")
            .match_header("user-agent", USER_AGENT)
            .with_body(r#"{"data":{"id":"1"}}"#)
            .create();

        let body = client(&server).get_json("/v1/me", &[]).unwrap();

        assert_eq!(body, json!({"data": {"id": "1"}}));
        mock.assert();
    }

    #[test]
    fn retries_rate_limited_requests_after_retry_after() {
        let mut server = mockito::Server::new();
        let limited = server
            .mock("GET", "/v1/spaces")
            .with_status(429)
            .with_header("retry-after", "0")
            .with_body(r#"{"errors":[{"code":"RATE_LIMITED","message":"Slow down."}]}"#)
            .expect(1)
            .create();
        let ok = server
            .mock("GET", "/v1/spaces")
            .with_body(r#"{"data":[]}"#)
            .create();

        let body = client(&server).get_json("/v1/spaces", &[]).unwrap();

        assert_eq!(body, json!({"data": []}));
        limited.assert();
        ok.assert();
    }

    #[test]
    fn gives_up_when_retry_after_exceeds_the_maximum_wait() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/v1/spaces")
            .with_status(429)
            .with_header("retry-after", "60")
            .with_body(r#"{"errors":[{"code":"RATE_LIMITED","message":"Slow down.","details":{"retry_after_seconds":60}}]}"#)
            .expect(1)
            .create();

        let error = client(&server).get_json("/v1/spaces", &[]).unwrap_err();

        assert_eq!(error.api_code(), Some("RATE_LIMITED"));
        mock.assert();
    }

    #[test]
    fn retries_server_errors_up_to_the_limit() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/v1/spaces")
            .with_status(500)
            .with_header("x-request-id", "req-1")
            .with_body(r#"{"errors":[{"code":"INTERNAL_ERROR","message":"Oops."}]}"#)
            .expect(3)
            .create();

        let error = client(&server).get_json("/v1/spaces", &[]).unwrap_err();

        assert_eq!(error.api_code(), Some("INTERNAL_ERROR"));
        assert!(error.to_string().contains("Request ID: req-1"));
        mock.assert();
    }

    #[test]
    fn does_not_retry_a_delete_after_a_server_error() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("DELETE", "/v1/spaces/s/transactions/t")
            .with_status(503)
            .expect(1)
            .create();

        let error = client(&server).delete("/v1/spaces/s/transactions/t").unwrap_err();

        assert_eq!(error.api_code(), Some("HTTP_503"));
        mock.assert();
    }

    #[test]
    fn does_not_retry_client_errors() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/v1/me")
            .with_status(401)
            .with_body(r#"{"errors":[{"code":"TOKEN_EXPIRED","message":"Expired."}]}"#)
            .expect(1)
            .create();

        let error = client(&server).get_json("/v1/me", &[]).unwrap_err();

        assert_eq!(error.api_code(), Some("TOKEN_EXPIRED"));
        mock.assert();
    }

    #[test]
    fn sends_the_idempotency_key_on_create() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/v1/spaces/s/transactions")
            .match_header("idempotency-key", "key-1")
            .match_body(mockito::Matcher::Json(json!({"amount": "1.00"})))
            .with_status(201)
            .with_header("idempotent-replayed", "false")
            .with_body(r#"{"data":{}}"#)
            .create();

        let reply = client(&server)
            .post_json("/v1/spaces/s/transactions", &json!({"amount": "1.00"}), "key-1")
            .unwrap();

        assert_eq!(reply.headers["idempotent-replayed"], "false");
        mock.assert();
    }

    #[test]
    fn treats_an_empty_success_body_as_null() {
        let mut server = mockito::Server::new();
        server.mock("DELETE", "/v1/x").with_status(204).create();

        client(&server).delete("/v1/x").unwrap();
    }
}
