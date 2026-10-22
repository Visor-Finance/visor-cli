//! The account summary that `auth login`, `auth status`, and `whoami` show,
//! in the spirit of `gh auth status`: one check line, then the details.

use std::fmt::Write as _;

use serde_json::{Value, json};

use crate::api;
use crate::client::Client;
use crate::config::Config;
use crate::credentials::{self, Credential, Source, Store};
use crate::error::Result;
use crate::models::{Me, Space};
use crate::output::Paint;
use crate::resolve;

/// What the CLI knows about the login after one round trip to the API.
#[derive(Debug)]
pub struct Status {
    pub me: Me,
    pub credential: Credential,
    pub source: Source,
    /// The space commands act on, and how many spaces there are. `None`
    /// when the space list could not be read: the summary never fails on it.
    pub spaces: Option<(Option<Space>, usize)>,
}

impl Status {
    /// Reads the user and the spaces with `client`.
    pub fn fetch(client: &Client, credential: Credential, source: Source, config: &Config) -> Result<Self> {
        let me: Me = api::data(client.me()?)?;
        let spaces = client.spaces_typed().ok().map(|spaces| {
            let current = config
                .default_space
                .as_deref()
                .and_then(|id| spaces.iter().find(|space| space.id == id))
                .or_else(|| resolve::default_space(&spaces).ok())
                .cloned();
            (current, spaces.len())
        });
        Ok(Self {
            me,
            credential,
            source,
            spaces,
        })
    }

    /// The summary for people. `paint` styles the check mark.
    #[must_use]
    pub fn render(&self, paint: Paint) -> String {
        let me = &self.me;
        let mut out = format!(
            "{} Logged in to Visor as {} ({})\n\n",
            paint.ok(),
            paint.bold(&me.name),
            me.email
        );
        let mut rows = vec![
            ("Plan", capitalize(&me.plan)),
            ("Access", access(&self.credential)),
            ("Login", login(&self.credential)),
            ("Stored in", stored_in(&self.source)),
        ];
        if let Some((current, count)) = &self.spaces {
            let space = match current {
                Some(space) => format!("{} · {}", space.name, plural(*count, "space")),
                None => format!("none chosen · {}", plural(*count, "space")),
            };
            rows.push(("Space", space));
        }
        for (label, value) in rows {
            let _ = writeln!(out, "  {label:<11}{value}");
        }
        out
    }

    /// The summary for scripts: stable keys, no secrets.
    #[must_use]
    pub fn to_json(&self) -> Value {
        let me = &self.me;
        let login = match &self.credential {
            Credential::OAuth { scope, .. } => json!({
                "kind": "oauth",
                "scopes": scope.split_whitespace().collect::<Vec<_>>(),
            }),
            Credential::PersonalToken { token } => json!({
                "kind": "personal_token",
                "token": credentials::redact(token),
            }),
        };
        let source = match &self.source {
            Source::Env => "env",
            Source::Store(Store::Keyring) => "keychain",
            Source::Store(Store::File(_)) => "file",
        };
        let (space, spaces) = match &self.spaces {
            Some((Some(space), count)) => (
                json!({"id": space.id, "name": space.name, "type": space.kind}),
                json!(count),
            ),
            Some((None, count)) => (Value::Null, json!(count)),
            None => (Value::Null, Value::Null),
        };
        json!({
            "authenticated": true,
            "user": {"id": me.id, "name": me.name, "email": me.email, "plan": me.plan},
            "login": login,
            "credential_source": source,
            "space": space,
            "spaces": spaces,
        })
    }
}

/// What the credential lets the CLI do, as the consent screen names it.
#[must_use]
pub fn access(credential: &Credential) -> String {
    match credential {
        Credential::PersonalToken { .. } => "as set on the token in Settings".to_owned(),
        Credential::OAuth { scope, .. }
            if scope.split_whitespace().any(|s| s == "pfm:transactions:write") =>
        {
            "read and record transactions".to_owned()
        }
        Credential::OAuth { .. } => "read only".to_owned(),
    }
}

fn login(credential: &Credential) -> String {
    match credential {
        Credential::OAuth { .. } => "browser (Visor CLI), renews itself".to_owned(),
        Credential::PersonalToken { token } => format!("personal API token {}", credentials::redact(token)),
    }
}

fn stored_in(source: &Source) -> String {
    match source {
        Source::Env => "VISOR_TOKEN (environment)".to_owned(),
        Source::Store(store) => store.to_string(),
    }
}

fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

fn plural(count: usize, noun: &str) -> String {
    if count == 1 {
        format!("1 {noun}")
    } else {
        format!("{count} {noun}s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn me() -> Me {
        serde_json::from_value(json!({
            "id": "u1", "name": "Ana", "email": "ana@example.com", "locale": "pt-BR",
            "display_currency_code": "BRL", "plan": "premium", "active_space_id": null,
            "created_at": "2026-01-01T00:00:00Z"
        }))
        .unwrap()
    }

    fn space() -> Space {
        serde_json::from_value(
            json!({"id": "s1", "name": "Pessoal", "type": "personal", "default": true, "role": "owner"}),
        )
        .unwrap()
    }

    fn oauth() -> Credential {
        Credential::OAuth {
            access_token: "secret-access".into(),
            refresh_token: "secret-refresh".into(),
            expires_at: 0,
            scope: "pfm:read pfm:transactions:write".into(),
        }
    }

    #[test]
    fn renders_a_browser_login() {
        let status = Status {
            me: me(),
            credential: oauth(),
            source: Source::Store(Store::Keyring),
            spaces: Some((Some(space()), 3)),
        };

        let text = status.render(Paint::stdout());

        assert!(
            text.contains("Logged in to Visor as Ana (ana@example.com)"),
            "{text}"
        );
        assert!(text.contains("Plan       Premium"), "{text}");
        assert!(text.contains("Access     read and record transactions"), "{text}");
        assert!(text.contains("Space      Pessoal · 3 spaces"), "{text}");
        assert!(!text.contains("secret"), "{text}");
    }

    #[test]
    fn json_lists_scopes_and_hides_every_token() {
        let status = Status {
            me: me(),
            credential: Credential::PersonalToken {
                token: "visor_pat_abcd_secret".into(),
            },
            source: Source::Env,
            spaces: None,
        };

        let value = status.to_json();

        assert_eq!(value["login"]["token"], "visor_pat_abcd_…");
        assert_eq!(value["credential_source"], "env");
        assert_eq!(value["spaces"], Value::Null);
        assert!(!value.to_string().contains("secret"));

        let browser = Status {
            credential: oauth(),
            ..status
        }
        .to_json();
        assert_eq!(
            browser["login"]["scopes"],
            json!(["pfm:read", "pfm:transactions:write"])
        );
    }
}
