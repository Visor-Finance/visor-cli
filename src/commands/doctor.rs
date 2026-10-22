//! `visor doctor`: checks the CLI version, the login, the API, and the space,
//! like `resend doctor` and `fly doctor`. Fails when a check fails, so a
//! script or a support thread can run it.

use std::time::Duration;

use serde::Deserialize;
use serde_json::json;

use crate::cli::Global;
use crate::client::{self, Client};
use crate::commands::client_for;
use crate::config::{self, Config};
use crate::credentials::{self, Credential, Source, Store};
use crate::error::{Error, Result};
use crate::models::Me;
use crate::output::{self, Format, Paint};
use crate::{api, resolve};

const RELEASES_URL: &str = "https://api.github.com/repos/Visor-Finance/visor-cli/releases/latest";
const UPGRADE: &str = "brew upgrade visor-finance/tap/visor, or run the installer again";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outcome {
    Ok,
    Warn,
    Fail,
}

#[derive(Debug)]
struct Check {
    name: &'static str,
    outcome: Outcome,
    message: String,
}

impl Check {
    fn new(name: &'static str, outcome: Outcome, message: impl Into<String>) -> Self {
        Self {
            name,
            outcome,
            message: message.into(),
        }
    }
}

pub fn run(global: &Global, format: Format) -> Result<()> {
    let mut checks = vec![version()];
    let config_dir = config::config_dir()?;
    let config = Config::load(&config_dir)?;

    match login(&config_dir) {
        Ok((check, Some((credential, source)))) => {
            checks.push(check);
            let client = client_for(&global.api_url, &global.auth_url, credential, &source)?;
            let api_check = api(&client);
            let api_ok = api_check.outcome == Outcome::Ok;
            checks.push(api_check);
            if api_ok {
                checks.push(space(&client, &config));
            }
        }
        Ok((check, None)) | Err(check) => checks.push(check),
    }

    let failed = checks
        .iter()
        .filter(|check| check.outcome == Outcome::Fail)
        .count();
    if format == Format::Json {
        output::json(&json!({
            "ok": failed == 0,
            "checks": checks.iter().map(|check| json!({
                "name": check.name,
                "status": match check.outcome { Outcome::Ok => "ok", Outcome::Warn => "warn", Outcome::Fail => "fail" },
                "message": check.message,
            })).collect::<Vec<_>>(),
        }))?;
    } else {
        let paint = Paint::stdout();
        for check in &checks {
            let mark = match check.outcome {
                Outcome::Ok => paint.ok(),
                Outcome::Warn => paint.warn(),
                Outcome::Fail => paint.fail(),
            };
            output::line(&format!("  {mark} {:<13}{}", check.name, check.message))?;
        }
    }

    match failed {
        0 => Ok(()),
        count => Err(Error::ChecksFailed(count)),
    }
}

/// Compares this build with the latest GitHub release.
fn version() -> Check {
    let current = env!("CARGO_PKG_VERSION");
    match latest_release() {
        Some(latest) if newer(&latest, current) => Check::new(
            "CLI version",
            Outcome::Warn,
            format!("{current}; {latest} is out: {UPGRADE}"),
        ),
        Some(_) => Check::new("CLI version", Outcome::Ok, format!("{current} (latest)")),
        None => Check::new(
            "CLI version",
            Outcome::Warn,
            format!("{current} (could not check for updates)"),
        ),
    }
}

fn latest_release() -> Option<String> {
    #[derive(Deserialize)]
    struct Release {
        tag_name: String,
    }
    let url = std::env::var("VISOR_RELEASES_URL").unwrap_or_else(|_| RELEASES_URL.to_owned());
    let release: Release = client::http_with_timeout(Duration::from_secs(3))
        .ok()?
        .get(url)
        .header(reqwest::header::ACCEPT, "application/vnd.github+json")
        .send()
        .ok()?
        .error_for_status()
        .ok()?
        .json()
        .ok()?;
    Some(release.tag_name.trim_start_matches('v').to_owned())
}

/// True when the dotted version `a` is newer than `b`.
fn newer(a: &str, b: &str) -> bool {
    let parts =
        |version: &str| -> Vec<u64> { version.split('.').map(|part| part.parse().unwrap_or(0)).collect() };
    parts(a) > parts(b)
}

type Login = (Check, Option<(Credential, Source)>);

fn login(config_dir: &std::path::Path) -> Result<Login, Check> {
    let fail = |message: String| Check::new("Login", Outcome::Fail, message);
    let store = Store::from_env(config_dir).map_err(|error| fail(error.to_string()))?;
    let found = credentials::resolve(&store).map_err(|error| fail(error.to_string()))?;
    let Some((credential, source)) = found else {
        return Ok((fail("not logged in: run `visor auth login`".to_owned()), None));
    };
    let kind = match &credential {
        Credential::OAuth { .. } => "browser login".to_owned(),
        Credential::PersonalToken { token } => format!("personal API token {}", credentials::redact(token)),
    };
    let check = Check::new("Login", Outcome::Ok, format!("{kind}, from {source}"));
    Ok((check, Some((credential, source))))
}

fn api(client: &Client) -> Check {
    match client.me().and_then(api::data::<Me>) {
        Ok(me) => Check::new(
            "Visor API",
            Outcome::Ok,
            format!("{} · {} plan", me.email, me.plan),
        ),
        Err(error) => {
            let first_line = error.to_string().lines().next().unwrap_or_default().to_owned();
            Check::new("Visor API", Outcome::Fail, first_line)
        }
    }
}

fn space(client: &Client, config: &Config) -> Check {
    let Ok(spaces) = client.spaces_typed() else {
        return Check::new("Space", Outcome::Warn, "could not list your spaces");
    };
    match &config.default_space {
        Some(id) => match spaces.iter().find(|space| &space.id == id) {
            Some(space) => Check::new(
                "Space",
                Outcome::Ok,
                format!("{} (set with `visor spaces use`)", space.name),
            ),
            None => Check::new(
                "Space",
                Outcome::Warn,
                "the space set with `visor spaces use` is gone; run `visor spaces use` again",
            ),
        },
        None => match resolve::default_space(&spaces) {
            Ok(space) => Check::new(
                "Space",
                Outcome::Ok,
                format!("{} (your default space)", space.name),
            ),
            Err(_) => Check::new(
                "Space",
                Outcome::Warn,
                "no space chosen: run `visor spaces use <name>`",
            ),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::status;

    #[test]
    fn compares_versions_by_number() {
        assert!(newer("0.10.0", "0.9.1"));
        assert!(newer("1.0.0", "0.99.99"));
        assert!(!newer("0.2.0", "0.2.0"));
        assert!(!newer("0.1.9", "0.2.0"));
    }

    #[test]
    fn status_access_matches_the_consent_screen() {
        let read = Credential::OAuth {
            access_token: String::new(),
            refresh_token: String::new(),
            expires_at: 0,
            scope: "pfm:read".into(),
        };
        assert_eq!(status::access(&read), "read only");
    }
}
