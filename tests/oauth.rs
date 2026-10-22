//! End-to-end tests of the browser login and of token refresh, against a
//! mock API and authorization server.

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

use mockito::{Matcher, Server, ServerGuard};
use reqwest::Url;
use serde_json::{Value, json};
use tempfile::TempDir;

const CLIENT_ID: &str = "908603ea-465f-4d4c-b5ac-30955885ee1d";

struct Harness {
    server: ServerGuard,
    config: TempDir,
}

impl Harness {
    fn new() -> Self {
        Self {
            server: Server::new(),
            config: TempDir::new().unwrap(),
        }
    }

    fn command(&self) -> Command {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_visor"));
        cmd.env("VISOR_API_URL", self.server.url())
            .env("VISOR_AUTH_URL", self.server.url())
            .env("VISOR_CONFIG_DIR", self.config.path())
            .env("VISOR_CREDENTIAL_STORE", "file")
            .env_remove("VISOR_TOKEN")
            .env_remove("VISOR_SPACE");
        cmd
    }

    fn credentials_path(&self) -> std::path::PathBuf {
        self.config.path().join("credentials.json")
    }

    fn stored(&self) -> Value {
        serde_json::from_str(&std::fs::read_to_string(self.credentials_path()).unwrap()).unwrap()
    }

    fn store(&self, credential: &Value) {
        std::fs::write(self.credentials_path(), credential.to_string()).unwrap();
    }

    fn me(&mut self, bearer: &str) -> mockito::Mock {
        self.server
            .mock("GET", "/v1/spaces")
            .with_body(
                json!({"data": [{"id": "11111111-1111-4111-8111-111111111111", "name": "Pessoal",
                                 "type": "personal", "default": true, "role": "owner"}]})
                .to_string(),
            )
            .create();
        self.server
            .mock("GET", "/v1/me")
            .match_header("authorization", format!("Bearer {bearer}").as_str())
            .with_body(me().to_string())
            .create()
    }

    fn expired(&mut self, bearer: &str) -> mockito::Mock {
        self.server
            .mock("GET", "/v1/me")
            .match_header("authorization", format!("Bearer {bearer}").as_str())
            .with_status(401)
            .with_body(r#"{"errors":[{"code":"TOKEN_EXPIRED","message":"The token expired.","details":{}}]}"#)
            .create()
    }

    fn refresh(&mut self, refresh_token: &str, body: &str, status: usize) -> mockito::Mock {
        self.server
            .mock("POST", "/oauth/token")
            .match_body(Matcher::AllOf(vec![
                Matcher::UrlEncoded("grant_type".into(), "refresh_token".into()),
                Matcher::UrlEncoded("client_id".into(), CLIENT_ID.into()),
                Matcher::UrlEncoded("refresh_token".into(), refresh_token.into()),
            ]))
            .with_status(status)
            .with_body(body)
            .create()
    }
}

fn me() -> Value {
    json!({"data": {
        "id": "77777777-7777-4777-8777-777777777777", "name": "Ana", "email": "ana@example.com",
        "locale": "pt-BR", "display_currency_code": "BRL", "plan": "premium",
        "active_space_id": null, "created_at": "2026-01-02T03:04:05Z"
    }})
}

fn oauth(access: &str, refresh: &str, expires_at: i64) -> Value {
    json!({
        "kind": "oauth", "access_token": access, "refresh_token": refresh,
        "expires_at": expires_at, "scope": "pfm:read pfm:transactions:write"
    })
}

fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .try_into()
        .unwrap()
}

#[test]
fn browser_login_exchanges_the_code_and_stores_both_tokens() {
    let mut harness = Harness::new();
    let token = harness
        .server
        .mock("POST", "/oauth/token")
        .match_body(Matcher::AllOf(vec![
            Matcher::UrlEncoded("grant_type".into(), "authorization_code".into()),
            Matcher::UrlEncoded("client_id".into(), CLIENT_ID.into()),
            Matcher::UrlEncoded("code".into(), "the-code".into()),
            Matcher::Regex("code_verifier=[A-Za-z0-9_-]{43}".into()),
            Matcher::Regex(r"redirect_uri=http%3A%2F%2F127\.0\.0\.1%3A\d+%2Fcallback".into()),
        ]))
        .with_body(
            json!({"access_token": "a1", "refresh_token": "r1", "expires_in": 86400,
                   "token_type": "bearer", "scope": "pfm:read"})
            .to_string(),
        )
        .create();
    let me_mock = harness.me("a1");

    let mut child = harness
        .command()
        .args(["auth", "login", "--no-browser"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    // Act as the browser: read the login page URL the CLI prints, then follow
    // the redirect Visor would send after consent.
    let mut stderr = BufReader::new(child.stderr.take().unwrap());
    let authorize = loop {
        let mut line = String::new();
        assert_ne!(
            stderr.read_line(&mut line).unwrap(),
            0,
            "the CLI printed no login URL"
        );
        if let Some(url) = line
            .trim()
            .strip_prefix(&format!("{}/oauth/authorize?", harness.server.url()))
        {
            break Url::parse(&format!("http://x/?{url}")).unwrap();
        }
    };
    let param = |name: &str| {
        authorize
            .query_pairs()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.into_owned())
            .unwrap()
    };
    assert_eq!(param("client_id"), CLIENT_ID);
    assert_eq!(param("code_challenge_method"), "S256");
    assert_eq!(param("response_type"), "code");

    let callback = format!("{}?code=the-code&state={}", param("redirect_uri"), param("state"));
    let page = reqwest::blocking::get(callback).unwrap();
    assert_eq!(page.status().as_u16(), 200);
    assert!(page.text().unwrap().contains("You are logged in"));

    let status = child.wait().unwrap();
    let mut rest = String::new();
    std::io::Read::read_to_string(&mut stderr, &mut rest).unwrap();
    assert!(status.success(), "{rest}");
    assert!(
        rest.contains("Logged in to Visor as Ana (ana@example.com)") && rest.contains("Access     read only"),
        "{rest}"
    );
    token.assert();
    me_mock.assert();

    let stored = harness.stored();
    assert_eq!(stored["kind"], "oauth");
    assert_eq!(stored["access_token"], "a1");
    assert_eq!(stored["refresh_token"], "r1");
    assert!(stored["expires_at"].as_i64().unwrap() > now() + 86000);
}

#[test]
fn browser_login_fails_when_the_user_denies_it() {
    let harness = Harness::new();
    let mut child = harness
        .command()
        .args(["auth", "login", "--no-browser"])
        .stdin(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let mut stderr = BufReader::new(child.stderr.take().unwrap());
    let authorize = loop {
        let mut line = String::new();
        assert_ne!(stderr.read_line(&mut line).unwrap(), 0);
        if line.contains("/oauth/authorize?") {
            break Url::parse(line.trim()).unwrap();
        }
    };
    let param = |name: &str| {
        authorize
            .query_pairs()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.into_owned())
            .unwrap()
    };
    let callback = format!(
        "{}?error=access_denied&state={}",
        param("redirect_uri"),
        param("state")
    );
    reqwest::blocking::get(callback).unwrap();

    let status = child.wait().unwrap();
    let mut rest = String::new();
    std::io::Read::read_to_string(&mut stderr, &mut rest).unwrap();
    assert_eq!(status.code(), Some(3), "{rest}");
    assert!(rest.contains("denied"), "{rest}");
    assert!(!harness.credentials_path().exists());
}

#[test]
fn an_expired_access_token_is_refreshed_and_the_request_retried() {
    let mut harness = Harness::new();
    harness.store(&oauth("a1", "r1", now() + 3600));
    let rejected = harness.expired("a1");
    let refresh = harness.refresh(
        "r1",
        r#"{"access_token":"a2","refresh_token":"r2","expires_in":86400,"token_type":"bearer"}"#,
        200,
    );
    let retried = harness.me("a2");

    let output = harness.command().arg("me").output().unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("ana@example.com"));
    rejected.assert();
    refresh.assert();
    retried.assert();

    let stored = harness.stored();
    assert_eq!(stored["access_token"], "a2");
    assert_eq!(stored["refresh_token"], "r2");
    assert_eq!(stored["scope"], "pfm:read pfm:transactions:write");
}

#[test]
fn a_token_past_its_expiry_is_refreshed_before_the_request() {
    let mut harness = Harness::new();
    harness.store(&oauth("a1", "r1", now() - 5));
    let stale = harness.me("a1").expect(0);
    harness.refresh(
        "r1",
        r#"{"access_token":"a2","refresh_token":"r2","expires_in":86400,"token_type":"bearer"}"#,
        200,
    );
    let fresh = harness.me("a2");

    let output = harness.command().arg("me").output().unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    stale.assert();
    fresh.assert();
}

#[test]
fn a_rejected_refresh_token_asks_to_log_in_again() {
    let mut harness = Harness::new();
    harness.store(&oauth("a1", "r1", now() + 3600));
    harness.expired("a1");
    harness.refresh(
        "r1",
        r#"{"error":"invalid_grant","error_description":"Given refresh token is invalid."}"#,
        400,
    );

    let output = harness.command().arg("me").output().unwrap();

    assert_eq!(output.status.code(), Some(3));
    assert!(String::from_utf8_lossy(&output.stderr).contains("visor auth login"));
}

#[test]
fn a_revoked_login_is_not_refreshed() {
    let mut harness = Harness::new();
    harness.store(&oauth("a1", "r1", now() + 3600));
    harness
        .server
        .mock("GET", "/v1/me")
        .with_status(401)
        .with_body(r#"{"errors":[{"code":"TOKEN_REVOKED","message":"The token was revoked.","details":{}}]}"#)
        .create();
    let refresh = harness.refresh("r1", "{}", 200).expect(0);

    let output = harness.command().arg("me").output().unwrap();

    assert_eq!(output.status.code(), Some(3));
    assert!(String::from_utf8_lossy(&output.stderr).contains("TOKEN_REVOKED"));
    refresh.assert();
}

#[test]
fn status_describes_a_browser_login() {
    let mut harness = Harness::new();
    harness.store(&oauth("a1", "r1", now() + 3600));
    harness.me("a1");

    let output = harness.command().args(["auth", "status"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("browser (Visor CLI), renews itself"), "{stdout}");
    assert!(
        stdout.contains("Access     read and record transactions"),
        "{stdout}"
    );
    assert!(stdout.contains("Space      Pessoal · 1 space"), "{stdout}");
    assert!(!stdout.contains("a1"), "{stdout}");
}

#[test]
fn logout_of_a_browser_login_explains_how_to_revoke_it() {
    let harness = Harness::new();
    harness.store(&oauth("a1", "r1", now() + 3600));

    let output = harness.command().args(["auth", "logout"]).output().unwrap();

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("revoke Visor CLI"));
    assert!(!harness.credentials_path().exists());
}

#[test]
fn a_personal_token_from_before_the_browser_login_still_works() {
    let mut harness = Harness::new();
    harness.store(&json!({"kind": "personal_token", "token": "visor_pat_abcd2345_secret"}));
    let me_mock = harness.me("visor_pat_abcd2345_secret");

    let output = harness.command().arg("me").output().unwrap();

    assert!(output.status.success());
    me_mock.assert();
}
