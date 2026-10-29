//! Errors and the exit codes they map to.

use std::fmt;
use std::process::ExitCode;

use serde::Deserialize;
use serde_json::Value;

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Exit codes are part of the CLI contract; scripts branch on them.
pub mod exit {
    pub const FAILURE: u8 = 1;
    pub const USAGE: u8 = 2;
    pub const AUTH: u8 = 3;
    pub const NOT_FOUND: u8 = 4;
    pub const REJECTED: u8 = 5;
    pub const UNAVAILABLE: u8 = 6;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Api(ApiError),

    #[error("could not reach the Visor API: {0}")]
    Network(#[source] reqwest::Error),

    #[error("not logged in. Run `visor auth login` or set VISOR_TOKEN")]
    NotLoggedIn,

    #[error("the browser login expired or was revoked. Run `visor auth login` again")]
    LoginExpired,

    #[error("login failed: {0}")]
    Login(String),

    #[error("{0} check(s) failed")]
    ChecksFailed(usize),

    #[error("{0}")]
    Usage(String),

    #[error("{0}")]
    Credentials(String),

    #[error("{context}: {source}")]
    Io {
        context: String,
        #[source]
        source: std::io::Error,
    },

    #[error("unexpected response from the Visor API: {0}")]
    Decode(String),

    /// The user canceled a question with Esc or Ctrl+C.
    #[error("canceled")]
    Canceled,

    /// A Pix that ended without being paid: `failed`, `canceled`, or `expired`.
    #[error("{message}")]
    PixNotPaid { status: String, message: String },

    #[error("the Pix is still waiting after {minutes} minutes. Check it with `visor pix status {id}`")]
    PixWaitTimeout { id: String, minutes: u64 },

    #[error(
        "could not get the latest version from GitHub. Try again, or see https://github.com/Visor-Finance/visor-cli/releases"
    )]
    UpdateCheck,

    /// `visor update --check` found a newer release.
    #[error("visor {latest} is out. Run `visor update`")]
    UpdateAvailable { latest: String },

    /// The CLI cannot replace this install itself; the message says how.
    #[error("{0}")]
    CannotUpdate(String),

    #[error("{0}")]
    UpdateFailed(String),
}

impl Error {
    pub fn usage(message: impl Into<String>) -> Self {
        Self::Usage(message.into())
    }

    pub fn io(context: impl Into<String>, source: std::io::Error) -> Self {
        Self::Io {
            context: context.into(),
            source,
        }
    }

    #[must_use]
    pub fn exit_code(&self) -> ExitCode {
        ExitCode::from(self.exit_status())
    }

    /// The numeric exit status; see [`exit`].
    #[must_use]
    pub fn exit_status(&self) -> u8 {
        match self {
            Self::Api(api) => api.exit_code(),
            Self::Network(_) | Self::UpdateCheck => exit::UNAVAILABLE,
            Self::NotLoggedIn | Self::LoginExpired | Self::Login(_) => exit::AUTH,
            Self::Usage(_) => exit::USAGE,
            Self::PixNotPaid { .. } => exit::REJECTED,
            Self::Credentials(_)
            | Self::Io { .. }
            | Self::Decode(_)
            | Self::ChecksFailed(_)
            | Self::PixWaitTimeout { .. }
            | Self::Canceled
            | Self::UpdateAvailable { .. }
            | Self::CannotUpdate(_)
            | Self::UpdateFailed(_) => exit::FAILURE,
        }
    }

    /// The error as `{"error": {...}}` for `--json`: a stable `code`, the
    /// message, and for an API error its HTTP status, details, and request ID.
    #[must_use]
    pub fn to_json(&self) -> serde_json::Value {
        let code = match self {
            Self::Api(api) => api.code.clone(),
            Self::Network(_) => "NETWORK_ERROR".to_owned(),
            Self::NotLoggedIn => "NOT_LOGGED_IN".to_owned(),
            Self::LoginExpired => "LOGIN_EXPIRED".to_owned(),
            Self::Login(_) => "LOGIN_FAILED".to_owned(),
            Self::ChecksFailed(_) => "CHECKS_FAILED".to_owned(),
            Self::Usage(_) => "USAGE".to_owned(),
            Self::Credentials(_) => "CREDENTIAL_STORE".to_owned(),
            Self::Io { .. } => "IO_ERROR".to_owned(),
            Self::Decode(_) => "UNEXPECTED_RESPONSE".to_owned(),
            Self::PixNotPaid { status, .. } => format!("PIX_{}", status.to_uppercase()),
            Self::PixWaitTimeout { .. } => "PIX_WAIT_TIMEOUT".to_owned(),
            Self::Canceled => "CANCELED".to_owned(),
            Self::UpdateCheck => "UPDATE_CHECK_FAILED".to_owned(),
            Self::UpdateAvailable { .. } => "UPDATE_AVAILABLE".to_owned(),
            Self::CannotUpdate(_) => "CANNOT_UPDATE".to_owned(),
            Self::UpdateFailed(_) => "UPDATE_FAILED".to_owned(),
        };
        let mut error = serde_json::json!({
            "code": code,
            "message": match self {
                Self::Api(api) => api.message.clone(),
                other => other.to_string(),
            },
            "exit_code": self.exit_status(),
        });
        if let Self::Api(api) = self {
            error["status"] = api.status.into();
            error["details"] = api.details.clone();
            error["request_id"] = api.request_id.clone().into();
        }
        serde_json::json!({ "error": error })
    }

    /// Returns the API error code, such as `NOT_FOUND`, when there is one.
    #[must_use]
    pub fn api_code(&self) -> Option<&str> {
        match self {
            Self::Api(api) => Some(&api.code),
            _ => None,
        }
    }
}

/// One error from the API envelope `{"errors": [...]}`.
#[derive(Debug, Clone, Deserialize)]
pub struct ApiError {
    #[serde(skip)]
    pub status: u16,
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub details: Value,
    #[serde(default)]
    pub request_id: Option<String>,
}

#[derive(Deserialize)]
struct Envelope {
    errors: Vec<ApiError>,
}

impl ApiError {
    /// Parses an error body. A body outside the envelope still yields an
    /// error that carries the HTTP status.
    #[must_use]
    pub fn from_body(status: u16, body: &str, request_id: Option<String>) -> Self {
        let parsed = serde_json::from_str::<Envelope>(body)
            .ok()
            .and_then(|envelope| envelope.errors.into_iter().next());

        match parsed {
            Some(mut error) => {
                error.status = status;
                error.request_id = error.request_id.or(request_id);
                error
            }
            None => Self {
                status,
                code: format!("HTTP_{status}"),
                message: format!("the API answered with HTTP {status}"),
                details: Value::Null,
                request_id,
            },
        }
    }

    fn exit_code(&self) -> u8 {
        match self.status {
            401 | 403 => exit::AUTH,
            404 => exit::NOT_FOUND,
            400 | 406 | 409 | 422 => exit::REJECTED,
            429 | 500.. => exit::UNAVAILABLE,
            _ => exit::FAILURE,
        }
    }

    /// The banks in `details.banks`, quoted for a `--bank` flag.
    fn banks(&self) -> Option<String> {
        let banks: Vec<String> = self.details["banks"]
            .as_array()?
            .iter()
            .filter_map(Value::as_str)
            .map(|bank| format!("\"{bank}\""))
            .collect();
        (!banks.is_empty()).then(|| banks.join(", "))
    }

    fn hint(&self) -> Option<String> {
        let hint = match self.code.as_str() {
            "UNAUTHENTICATED" | "INVALID_TOKEN" => {
                "Check the token, then run `visor auth login` again.".to_owned()
            }
            "TOKEN_EXPIRED" | "TOKEN_REVOKED" => {
                "Run `visor auth login` again. With VISOR_TOKEN, create a new token in Settings.".to_owned()
            }
            "PLAN_REQUIRED" => "The personal API needs an active Pro or Premium plan.".to_owned(),
            "INSUFFICIENT_SCOPE" if self.details["required_scope"] == PIX_SCOPE => {
                "Starting a Pix needs the Visor CLI browser login with Pix access. Run `visor pix` \
                 in a terminal to allow it in the browser. A personal API token or VISOR_TOKEN \
                 cannot start a Pix."
                    .to_owned()
            }
            "INSUFFICIENT_SCOPE" => {
                let scope = self.details["required_scope"].as_str().unwrap_or("another scope");
                format!(
                    "This login lacks `{scope}`. Run `visor auth login` and choose Read and record \
                     transactions, or create a token with that access at {TOKENS_URL}."
                )
            }
            "BANK_REQUIRED" => match self.banks() {
                Some(banks) => format!("Choose the bank the Pix comes from with --bank, such as {banks}."),
                None => "Choose the bank the Pix comes from with --bank.".to_owned(),
            },
            "BANK_NOT_FOUND" => match self.banks() {
                Some(banks) => format!("Banks you can pay from with --bank: {banks}."),
                None => "Choose another bank with --bank.".to_owned(),
            },
            "AMBIGUOUS_PIX_KEY" => {
                "Run the command again with --key-type cpf or --key-type phone.".to_owned()
            }
            "CPF_REQUIRED" => format!("Add your CPF to your profile at {SETTINGS_URL}, then try again."),
            "PREMIUM_REQUIRED" => format!(
                "Pix needs the Pro or Premium plan. See the plans at {}.",
                self.details["url"].as_str().unwrap_or(PLAN_URL)
            ),
            "PIX_MONTHLY_LIMIT_REACHED" => {
                let limit = self.details["limit"]
                    .as_u64()
                    .map_or_else(String::new, |limit| format!(" of {limit} Pix"));
                match self.details["renews_on"].as_str() {
                    Some(renews_on) => {
                        format!("You used this month's limit{limit}. It renews on {renews_on}.")
                    }
                    None => format!("You used this month's limit{limit}."),
                }
            }
            "PIX_UNAVAILABLE" => "Pix through Visor is not available for this account yet.".to_owned(),
            "PIX_START_FAILED" => {
                "Run the command again; it sends a new Idempotency-Key unless you pass --idempotency-key."
                    .to_owned()
            }
            "RATE_LIMITED" => match self.details["retry_after_seconds"].as_u64() {
                Some(seconds) => format!("Try again in {seconds} seconds."),
                None => "Try again in a minute.".to_owned(),
            },
            _ => return None,
        };
        Some(hint)
    }
}

pub const TOKENS_URL: &str = "https://dashboard.visorfinance.app/settings/connected_apps";
const SETTINGS_URL: &str = "https://dashboard.visorfinance.app/settings";
const PLAN_URL: &str = "https://dashboard.visorfinance.app/settings/plan";

/// The scope that starting a Pix needs.
pub const PIX_SCOPE: &str = "pix:payments:write";

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.message, self.code)?;

        if let Some(errors) = self.details["errors"].as_array() {
            for error in errors {
                let field = error["field"].as_str().unwrap_or("body");
                let message = error["message"].as_str().unwrap_or("is invalid");
                write!(f, "\n  {field}: {message}")?;
            }
        }
        for key in ["field", "parameter", "reason"] {
            if let Some(value) = self.details[key].as_str() {
                write!(f, "\n  {key}: {value}")?;
            }
        }
        if let Some(hint) = self.hint() {
            write!(f, "\n{hint}")?;
        }
        if let Some(request_id) = &self.request_id {
            write!(f, "\nRequest ID: {request_id}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_error_envelope() {
        let body = r#"{"errors":[{"code":"INSUFFICIENT_SCOPE","message":"Missing scope.","details":{"required_scope":"pfm:transactions:write"},"request_id":"abc"}]}"#;
        let error = ApiError::from_body(403, body, None);

        assert_eq!(error.code, "INSUFFICIENT_SCOPE");
        assert_eq!(error.status, 403);
        let text = error.to_string();
        assert!(text.contains("pfm:transactions:write"), "{text}");
        assert!(text.contains("Request ID: abc"), "{text}");
    }

    #[test]
    fn falls_back_to_the_status_when_the_body_is_not_an_envelope() {
        let error = ApiError::from_body(502, "<html>bad gateway</html>", Some("req".into()));

        assert_eq!(error.code, "HTTP_502");
        assert_eq!(error.request_id.as_deref(), Some("req"));
        assert_eq!(Error::Api(error).exit_status(), exit::UNAVAILABLE);
    }

    #[test]
    fn lists_validation_errors() {
        let body = r#"{"errors":[{"code":"VALIDATION_FAILED","message":"Invalid body.","details":{"errors":[{"field":"amount","message":"must be positive"}]}}]}"#;
        let text = ApiError::from_body(422, body, None).to_string();

        assert!(text.contains("amount: must be positive"), "{text}");
    }

    #[test]
    fn json_errors_carry_a_stable_code() {
        let body =
            r#"{"errors":[{"code":"NOT_FOUND","message":"Not found.","details":{},"request_id":"r1"}]}"#;
        let api = Error::Api(ApiError::from_body(404, body, None)).to_json();

        assert_eq!(api["error"]["code"], "NOT_FOUND");
        assert_eq!(api["error"]["status"], 404);
        assert_eq!(api["error"]["request_id"], "r1");
        assert_eq!(api["error"]["exit_code"], exit::NOT_FOUND);

        let local = Error::NotLoggedIn.to_json();
        assert_eq!(local["error"]["code"], "NOT_LOGGED_IN");
        assert_eq!(local["error"]["exit_code"], exit::AUTH);
        assert!(local["error"].get("status").is_none());
    }

    #[test]
    fn pix_errors_say_what_to_do_next() {
        let hint = |code: &str, details: &str| {
            let body = format!(r#"{{"errors":[{{"code":"{code}","message":"m","details":{details}}}]}}"#);
            ApiError::from_body(422, &body, None).to_string()
        };

        let text = hint("BANK_REQUIRED", r#"{"banks":["Nubank","Itaú"]}"#);
        assert!(text.contains(r#"--bank, such as "Nubank", "Itaú""#), "{text}");
        let text = hint("AMBIGUOUS_PIX_KEY", "{}");
        assert!(text.contains("--key-type cpf or --key-type phone"), "{text}");
        let text = hint(
            "PIX_MONTHLY_LIMIT_REACHED",
            r#"{"limit":30,"renews_on":"2026-11-01"}"#,
        );
        assert!(
            text.contains("limit of 30 Pix. It renews on 2026-11-01"),
            "{text}"
        );
        let text = hint("INSUFFICIENT_SCOPE", r#"{"required_scope":"pix:payments:write"}"#);
        assert!(text.contains("Run `visor pix` in a terminal"), "{text}");
    }

    #[test]
    fn an_unpaid_pix_is_a_rejected_request() {
        let error = Error::PixNotPaid {
            status: "failed".into(),
            message: "Saldo insuficiente".into(),
        };

        assert_eq!(error.exit_status(), exit::REJECTED);
        assert_eq!(error.to_json()["error"]["code"], "PIX_FAILED");
    }

    #[test]
    fn maps_statuses_to_exit_codes() {
        let code = |status| Error::Api(ApiError::from_body(status, "", None)).exit_status();

        assert_eq!(code(401), exit::AUTH);
        assert_eq!(code(403), exit::AUTH);
        assert_eq!(code(404), exit::NOT_FOUND);
        assert_eq!(code(409), exit::REJECTED);
        assert_eq!(code(422), exit::REJECTED);
        assert_eq!(code(429), exit::UNAVAILABLE);
        assert_eq!(code(500), exit::UNAVAILABLE);
    }
}
