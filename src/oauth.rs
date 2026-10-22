//! Browser login for the Visor CLI: OAuth 2.0 authorization code with PKCE
//! on a loopback redirect (RFC 8252), and the refresh of its tokens.
//!
//! The access token lasts 24 hours and each refresh token works once, so a
//! refresh stores the new pair before the CLI sends anything with it.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use reqwest::Url;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::client::{self, TokenSource};
use crate::credentials::{Credential, Store};
use crate::error::{Error, Result};
use crate::output;

pub const DEFAULT_AUTH_URL: &str = "https://dashboard.visorfinance.app";

/// The first-party client ID that `Visor.OAuth.CliClient` registers.
pub const CLIENT_ID: &str = "908603ea-465f-4d4c-b5ac-30955885ee1d";

/// How long `visor auth login` waits for the browser.
const LOGIN_TIMEOUT: Duration = Duration::from_secs(300);

/// Refresh this long before the access token expires, so a request never
/// leaves with a token that dies on the way.
const EXPIRY_MARGIN_SECONDS: i64 = 60;

/// The PKCE pair of one login (RFC 7636).
#[derive(Debug)]
struct Pkce {
    verifier: String,
    challenge: String,
}

impl Pkce {
    fn new() -> Result<Self> {
        Ok(Self::from_verifier(random_token(32)?))
    }

    fn from_verifier(verifier: String) -> Self {
        let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
        Self { verifier, challenge }
    }
}

fn random_token(bytes: usize) -> Result<String> {
    let mut buffer = vec![0u8; bytes];
    getrandom::fill(&mut buffer)
        .map_err(|error| Error::Login(format!("cannot generate a random value: {error}")))?;
    Ok(URL_SAFE_NO_PAD.encode(buffer))
}

fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

/// Runs the browser login and returns the credential to store.
pub fn login(auth_url: &str, open_browser: bool) -> Result<Credential> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .map_err(|error| Error::io("cannot listen for the browser callback", error))?;
    let port = listener
        .local_addr()
        .map_err(|error| Error::io("cannot read the callback port", error))?
        .port();
    let redirect_uri = format!("http://127.0.0.1:{port}/callback");
    let pkce = Pkce::new()?;
    let state = random_token(16)?;
    let url = authorize_url(auth_url, &redirect_uri, &pkce.challenge, &state)?;

    // Like `fly auth login`: say what opens, and give the URL a line of its
    // own so it can be copied to a browser elsewhere on this machine.
    if open_browser && open_in_browser(url.as_str()) {
        output::note("Opening the Visor login page in your browser:\n");
    } else {
        output::note("Open this page in a browser on this machine and approve the login:\n");
    }
    output::note(&format!("    {url}\n"));

    let mut spinner = output::Spinner::start("Waiting for you to approve the login in the browser…");
    let code = wait_for_code(&listener, &state, LOGIN_TIMEOUT, &mut || spinner.tick())?;
    spinner.finish("Login approved in the browser");
    let tokens = exchange(
        auth_url,
        &[
            ("grant_type", "authorization_code"),
            ("client_id", CLIENT_ID),
            ("code", &code),
            ("redirect_uri", &redirect_uri),
            ("code_verifier", &pkce.verifier),
        ],
    )?;
    tokens.into_credential(None)
}

fn authorize_url(auth_url: &str, redirect_uri: &str, challenge: &str, state: &str) -> Result<Url> {
    let mut url = Url::parse(&format!("{}/oauth/authorize", auth_url.trim_end_matches('/')))
        .map_err(|error| Error::usage(format!("VISOR_AUTH_URL is not a URL: {error}")))?;
    url.query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", CLIENT_ID)
        .append_pair("redirect_uri", redirect_uri)
        .append_pair("code_challenge", challenge)
        .append_pair("code_challenge_method", "S256")
        .append_pair("state", state);
    Ok(url)
}

fn open_in_browser(url: &str) -> bool {
    let mut command = if cfg!(target_os = "macos") {
        let mut command = Command::new("open");
        command.arg(url);
        command
    } else if cfg!(windows) {
        // `start` would split the URL at `&`; the URL handler takes it whole.
        let mut command = Command::new("rundll32");
        command.args(["url.dll,FileProtocolHandler", url]);
        command
    } else {
        let mut command = Command::new("xdg-open");
        command.arg(url);
        command
    };
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .is_ok()
}

/// What the browser sent to the callback.
#[derive(Debug, PartialEq, Eq)]
enum Callback {
    Code(String),
    Denied(String),
    /// Anything else, such as `/favicon.ico`.
    Other,
}

/// Accepts browser requests until one brings the code, an error, or the
/// timeout passes.
fn wait_for_code(
    listener: &TcpListener,
    state: &str,
    timeout: Duration,
    on_tick: &mut dyn FnMut(),
) -> Result<String> {
    listener
        .set_nonblocking(true)
        .map_err(|error| Error::io("cannot wait for the browser callback", error))?;
    let deadline = Instant::now() + timeout;
    let mut next_tick = Instant::now();

    while Instant::now() < deadline {
        if Instant::now() >= next_tick {
            on_tick();
            next_tick += Duration::from_millis(80);
        }
        match listener.accept() {
            Ok((stream, _)) => match handle_callback(stream, state)? {
                Callback::Code(code) => return Ok(code),
                Callback::Denied(reason) => return Err(Error::Login(reason)),
                Callback::Other => {}
            },
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(50));
            }
            Err(error) => return Err(Error::io("cannot accept the browser callback", error)),
        }
    }
    Err(Error::Login(format!(
        "nobody approved the login within {} minutes. Run `visor auth login` again",
        timeout.as_secs() / 60
    )))
}

fn handle_callback(mut stream: TcpStream, state: &str) -> Result<Callback> {
    let io = |error| Error::io("cannot read the browser callback", error);
    stream.set_nonblocking(false).map_err(io)?;
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(io)?;

    // A broken request is not the callback, so a read error just leaves the
    // line empty. 8 KiB is far more than a callback needs.
    let mut request_line = String::new();
    let _ = BufReader::new(&stream).take(8192).read_line(&mut request_line);
    let callback = parse_callback(&request_line, state);

    let (status, title, message) = match &callback {
        Ok(Callback::Code(_)) => (
            "200 OK",
            "You are logged in",
            "You can close this tab and go back to the terminal.",
        ),
        Ok(Callback::Denied(_)) | Err(_) => (
            "400 Bad Request",
            "The login did not finish",
            "Go back to the terminal to see why.",
        ),
        Ok(Callback::Other) => ("404 Not Found", "Not found", ""),
    };
    let page = callback_page(title, message);
    // The browser may close the connection first; the result is known already.
    let _ = write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{page}",
        page.len()
    );
    callback
}

/// The page the browser shows after the redirect, in the Visor colors.
fn callback_page(title: &str, message: &str) -> String {
    const ICON: &str = include_str!("../assets/icon.svg");
    format!(
        "<!doctype html><html lang=\"en\"><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\
         <title>Visor CLI - {title}</title><style>\
         :root{{color-scheme:light dark;--bg:#f5f8ff;--card:#fff;--text:#0b1220;--muted:#5b6475}}\
         @media (prefers-color-scheme:dark){{:root{{--bg:#0b1220;--card:#121a2b;--text:#f5f8ff;--muted:#9aa4b8}}}}\
         html,body{{margin:0;height:100%;background:var(--bg);color:var(--text);\
         font-family:system-ui,-apple-system,sans-serif}}\
         main{{min-height:100%;display:grid;place-items:center;padding:16px;box-sizing:border-box}}\
         .card{{background:var(--card);border-radius:24px;padding:40px 32px;max-width:28rem;text-align:center;\
         box-shadow:0 12px 40px rgba(1,71,227,.12)}}\
         svg{{width:72px;height:72px}}h1{{font-size:1.4rem;margin:20px 0 8px}}p{{color:var(--muted);margin:0}}\
         </style><main><div class=\"card\">{ICON}<h1>{title}</h1><p>{message}</p></div></main></html>"
    )
}

/// Reads the request line, such as `GET /callback?code=…&state=… HTTP/1.1`.
fn parse_callback(request_line: &str, state: &str) -> Result<Callback> {
    let mut parts = request_line.split_whitespace();
    let (Some("GET"), Some(target)) = (parts.next(), parts.next()) else {
        return Ok(Callback::Other);
    };
    let Ok(url) = Url::parse(&format!("http://127.0.0.1{target}")) else {
        return Ok(Callback::Other);
    };
    if url.path() != "/callback" {
        return Ok(Callback::Other);
    }

    let param = |name: &str| {
        url.query_pairs()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.into_owned())
    };
    if param("state").as_deref() != Some(state) {
        return Err(Error::Login(
            "the browser callback did not match this login. Run `visor auth login` again".to_owned(),
        ));
    }
    if let Some(error) = param("error") {
        let reason = match error.as_str() {
            "access_denied" => "the login was denied in the browser".to_owned(),
            _ => format!(
                "Visor refused the login: {}",
                param("error_description").unwrap_or(error)
            ),
        };
        return Ok(Callback::Denied(reason));
    }
    match param("code") {
        Some(code) if !code.is_empty() => Ok(Callback::Code(code)),
        _ => Err(Error::Login("the browser callback had no code".to_owned())),
    }
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: i64,
    #[serde(default)]
    scope: Option<String>,
}

impl TokenResponse {
    /// `previous` fills what a refresh response leaves out.
    fn into_credential(self, previous: Option<&Credential>) -> Result<Credential> {
        let (old_refresh, old_scope) = match previous {
            Some(Credential::OAuth {
                refresh_token, scope, ..
            }) => (Some(refresh_token.clone()), scope.clone()),
            _ => (None, String::new()),
        };
        let refresh_token = self
            .refresh_token
            .or(old_refresh)
            .ok_or_else(|| Error::Login("Visor sent no refresh token".to_owned()))?;
        Ok(Credential::OAuth {
            access_token: self.access_token,
            refresh_token,
            expires_at: now() + self.expires_in,
            scope: self.scope.unwrap_or(old_scope),
        })
    }
}

#[derive(Debug, Deserialize)]
struct TokenError {
    error: String,
    #[serde(default)]
    error_description: Option<String>,
}

/// Why a token request failed.
#[derive(Debug)]
enum ExchangeError {
    /// The code or refresh token is no longer valid (`invalid_grant`).
    InvalidGrant,
    Other(Error),
}

fn exchange(auth_url: &str, form: &[(&str, &str)]) -> Result<TokenResponse> {
    exchange_with(auth_url, form).map_err(|error| match error {
        ExchangeError::InvalidGrant => {
            Error::Login("Visor did not accept the login code. Run `visor auth login` again".to_owned())
        }
        ExchangeError::Other(error) => error,
    })
}

fn exchange_with(auth_url: &str, form: &[(&str, &str)]) -> Result<TokenResponse, ExchangeError> {
    let url = format!("{}/oauth/token", auth_url.trim_end_matches('/'));
    let response = client::http()
        .map_err(ExchangeError::Other)?
        .post(url)
        .header(reqwest::header::ACCEPT, "application/json")
        .form(form)
        .send()
        .map_err(|error| ExchangeError::Other(Error::Network(error)))?;

    let status = response.status();
    let text = response
        .text()
        .map_err(|error| ExchangeError::Other(Error::Network(error)))?;
    if status.is_success() {
        return serde_json::from_str(&text)
            .map_err(|error| ExchangeError::Other(Error::Decode(error.to_string())));
    }
    match serde_json::from_str::<TokenError>(&text) {
        Ok(error) if error.error == "invalid_grant" => Err(ExchangeError::InvalidGrant),
        Ok(error) => Err(ExchangeError::Other(Error::Login(format!(
            "Visor refused the token request: {}",
            error.error_description.unwrap_or(error.error)
        )))),
        Err(_) => Err(ExchangeError::Other(Error::Login(format!(
            "Visor answered the token request with HTTP {}",
            status.as_u16()
        )))),
    }
}

/// The tokens of a browser login, refreshed as they expire and saved after
/// every refresh.
#[derive(Debug)]
pub struct Session {
    auth_url: String,
    store: Store,
    credential: Mutex<Credential>,
}

impl Session {
    #[must_use]
    pub fn new(auth_url: &str, store: Store, credential: Credential) -> Self {
        Self {
            auth_url: auth_url.to_owned(),
            store,
            credential: Mutex::new(credential),
        }
    }

    fn refresh_locked(&self, credential: &mut Credential) -> Result<()> {
        let Credential::OAuth { refresh_token, .. } = &*credential else {
            return Err(Error::LoginExpired);
        };
        let form = [
            ("grant_type", "refresh_token"),
            ("client_id", CLIENT_ID),
            ("refresh_token", refresh_token.as_str()),
        ];
        match exchange_with(&self.auth_url, &form) {
            Ok(tokens) => {
                let refreshed = tokens.into_credential(Some(credential))?;
                // The old refresh token is spent: store the new pair first.
                self.store.save(&refreshed)?;
                *credential = refreshed;
                Ok(())
            }
            Err(ExchangeError::InvalidGrant) => self.adopt_stored(credential),
            Err(ExchangeError::Other(error)) => Err(error),
        }
    }

    /// Another `visor` process may have refreshed first and spent the token
    /// this one holds. Its result is in the store; use it when it is newer.
    fn adopt_stored(&self, credential: &mut Credential) -> Result<()> {
        match self.store.load()? {
            Some(stored @ Credential::OAuth { .. })
                if stored.refresh_token() != credential.refresh_token() =>
            {
                *credential = stored;
                Ok(())
            }
            _ => Err(Error::LoginExpired),
        }
    }
}

impl TokenSource for Session {
    fn bearer(&self) -> Result<String> {
        let mut credential = self.credential.lock().expect("credential lock");
        if credential
            .expires_at()
            .is_some_and(|at| at - now() <= EXPIRY_MARGIN_SECONDS)
        {
            self.refresh_locked(&mut credential)?;
        }
        Ok(credential.bearer().to_owned())
    }

    fn refresh(&self, rejected: &str) -> Result<Option<String>> {
        let mut credential = self.credential.lock().expect("credential lock");
        if credential.bearer() == rejected {
            self.refresh_locked(&mut credential)?;
        }
        Ok(Some(credential.bearer().to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkce_matches_the_rfc_7636_example() {
        let pkce = Pkce::from_verifier("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk".into());

        assert_eq!(pkce.challenge, "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM");
    }

    #[test]
    fn random_tokens_are_url_safe_and_unique() {
        let a = random_token(32).unwrap();
        let b = random_token(32).unwrap();

        assert_ne!(a, b);
        assert_eq!(a.len(), 43);
        assert!(
            a.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        );
    }

    #[test]
    fn builds_the_authorize_url() {
        let url = authorize_url(
            "https://dash.example/",
            "http://127.0.0.1:5000/callback",
            "ch",
            "st",
        )
        .unwrap();

        assert_eq!(url.path(), "/oauth/authorize");
        let query: Vec<(String, String)> = url.query_pairs().into_owned().collect();
        assert!(query.contains(&("client_id".into(), CLIENT_ID.into())));
        assert!(query.contains(&("redirect_uri".into(), "http://127.0.0.1:5000/callback".into())));
        assert!(query.contains(&("code_challenge_method".into(), "S256".into())));
        assert!(query.contains(&("state".into(), "st".into())));
    }

    #[test]
    fn parses_the_callback() {
        let ok = parse_callback("GET /callback?code=abc&state=s1 HTTP/1.1\r\n", "s1").unwrap();
        assert_eq!(ok, Callback::Code("abc".into()));

        let denied = parse_callback("GET /callback?error=access_denied&state=s1 HTTP/1.1", "s1").unwrap();
        assert!(matches!(denied, Callback::Denied(reason) if reason.contains("denied")));

        assert_eq!(
            parse_callback("GET /favicon.ico HTTP/1.1", "s1").unwrap(),
            Callback::Other
        );
        assert_eq!(parse_callback("", "s1").unwrap(), Callback::Other);
    }

    #[test]
    fn rejects_a_callback_with_another_state() {
        let error = parse_callback("GET /callback?code=abc&state=evil HTTP/1.1", "s1").unwrap_err();

        assert!(error.to_string().contains("did not match"), "{error}");
    }

    #[test]
    fn waits_for_the_code_and_answers_the_browser() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let browser = thread::spawn(move || {
            let get = |path: &str| {
                reqwest::blocking::get(format!("http://127.0.0.1:{port}{path}"))
                    .unwrap()
                    .status()
            };
            (get("/favicon.ico"), get("/callback?code=xyz&state=s1"))
        });

        let code = wait_for_code(&listener, "s1", Duration::from_secs(10), &mut || {}).unwrap();
        let (favicon, callback) = browser.join().unwrap();

        assert_eq!(code, "xyz");
        assert_eq!(favicon.as_u16(), 404);
        assert_eq!(callback.as_u16(), 200);
    }

    #[test]
    fn times_out_without_a_callback() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();

        let error = wait_for_code(&listener, "s1", Duration::from_millis(100), &mut || {}).unwrap_err();

        assert!(error.to_string().contains("nobody approved the login"), "{error}");
    }

    fn oauth(access: &str, refresh: &str, expires_at: i64) -> Credential {
        Credential::OAuth {
            access_token: access.into(),
            refresh_token: refresh.into(),
            expires_at,
            scope: "pfm:read".into(),
        }
    }

    #[test]
    fn refreshes_an_expiring_token_and_stores_the_new_pair() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/oauth/token")
            .match_body(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("grant_type".into(), "refresh_token".into()),
                mockito::Matcher::UrlEncoded("refresh_token".into(), "r1".into()),
                mockito::Matcher::UrlEncoded("client_id".into(), CLIENT_ID.into()),
            ]))
            .with_body(
                r#"{"access_token":"a2","refresh_token":"r2","expires_in":86400,"token_type":"bearer"}"#,
            )
            .create();
        let dir = tempfile::tempdir().unwrap();
        let store = Store::File(dir.path().join("credentials.json"));
        let session = Session::new(&server.url(), store.clone(), oauth("a1", "r1", now() - 10));

        assert_eq!(session.bearer().unwrap(), "a2");
        mock.assert();
        let stored = store.load().unwrap().unwrap();
        assert_eq!(stored.refresh_token(), Some("r2"));
        assert_eq!(stored.bearer(), "a2");
        assert!(matches!(stored, Credential::OAuth { ref scope, .. } if scope == "pfm:read"));
    }

    #[test]
    fn adopts_the_pair_another_process_stored() {
        let mut server = mockito::Server::new();
        server
            .mock("POST", "/oauth/token")
            .with_status(400)
            .with_body(r#"{"error":"invalid_grant","error_description":"Given refresh token is invalid."}"#)
            .create();
        let dir = tempfile::tempdir().unwrap();
        let store = Store::File(dir.path().join("credentials.json"));
        store.save(&oauth("a2", "r2", now() + 86400)).unwrap();
        let session = Session::new(&server.url(), store, oauth("a1", "r1", now() + 86400));

        assert_eq!(session.refresh("a1").unwrap().as_deref(), Some("a2"));
    }

    #[test]
    fn a_spent_refresh_token_means_logging_in_again() {
        let mut server = mockito::Server::new();
        server
            .mock("POST", "/oauth/token")
            .with_status(400)
            .with_body(r#"{"error":"invalid_grant"}"#)
            .create();
        let dir = tempfile::tempdir().unwrap();
        let store = Store::File(dir.path().join("credentials.json"));
        let credential = oauth("a1", "r1", now() + 86400);
        store.save(&credential).unwrap();
        let session = Session::new(&server.url(), store, credential);

        assert!(matches!(session.refresh("a1"), Err(Error::LoginExpired)));
    }

    #[test]
    fn does_not_refresh_twice_for_the_same_rejection() {
        let server = mockito::Server::new();
        let dir = tempfile::tempdir().unwrap();
        let store = Store::File(dir.path().join("credentials.json"));
        let session = Session::new(&server.url(), store, oauth("a2", "r2", now() + 86400));

        // Another thread already replaced `a1`; no token request is made.
        assert_eq!(session.refresh("a1").unwrap().as_deref(), Some("a2"));
    }
}
