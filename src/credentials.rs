//! Where the API credential lives.
//!
//! `VISOR_TOKEN` wins over a stored credential, so CI and one-off scripts
//! never touch the store. The store is the OS keychain (Keychain, Credential
//! Manager, or Secret Service). `VISOR_CREDENTIAL_STORE=file` keeps it in
//! `credentials.json` in the config directory, readable only by the owner,
//! for machines without a keychain.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::write_atomically;
use crate::error::{Error, Result};

const KEYRING_SERVICE: &str = "visor-cli";
const KEYRING_USER: &str = "default";
const FILE_NAME: &str = "credentials.json";

/// A stored credential. Tagged by kind so OAuth tokens (ENG-2592) can join
/// personal tokens without breaking existing logins.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Credential {
    PersonalToken { token: String },
}

impl Credential {
    #[must_use]
    pub fn bearer(&self) -> &str {
        match self {
            Self::PersonalToken { token } => token,
        }
    }
}

impl fmt::Debug for Credential {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PersonalToken { token } => f
                .debug_struct("PersonalToken")
                .field("token", &redact(token))
                .finish(),
        }
    }
}

/// Shows the safe prefix that Settings lists, such as `visor_pat_abcd2345`.
#[must_use]
pub fn redact(token: &str) -> String {
    match token.strip_prefix("visor_pat_") {
        Some(rest) => {
            let public_id = rest.split('_').next().unwrap_or_default();
            format!("visor_pat_{public_id}_…")
        }
        None => "…".to_owned(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Store {
    Keyring,
    File(PathBuf),
}

impl fmt::Display for Store {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Keyring => f.write_str("the system keychain"),
            Self::File(path) => write!(f, "{}", path.display()),
        }
    }
}

impl Store {
    pub fn from_env(config_dir: &Path) -> Result<Self> {
        match std::env::var("VISOR_CREDENTIAL_STORE").as_deref() {
            Err(_) | Ok("" | "keyring") => Ok(Self::Keyring),
            Ok("file") => Ok(Self::File(config_dir.join(FILE_NAME))),
            Ok(other) => Err(Error::usage(format!(
                "VISOR_CREDENTIAL_STORE must be `keyring` or `file`, not `{other}`"
            ))),
        }
    }

    pub fn load(&self) -> Result<Option<Credential>> {
        let text = match self {
            Self::Keyring => match entry()?.get_password() {
                Ok(text) => text,
                Err(keyring::Error::NoEntry) => return Ok(None),
                Err(error) => return Err(keyring_error(&error)),
            },
            Self::File(path) => match fs::read_to_string(path) {
                Ok(text) => text,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
                Err(error) => return Err(Error::io(format!("cannot read {}", path.display()), error)),
            },
        };
        serde_json::from_str(&text).map(Some).map_err(|_| {
            Error::Credentials(format!(
                "the credential in {self} is not valid; run `visor auth login` again"
            ))
        })
    }

    pub fn save(&self, credential: &Credential) -> Result<()> {
        let text = serde_json::to_string(credential).expect("credential serializes");
        match self {
            Self::Keyring => entry()?
                .set_password(&text)
                .map_err(|error| keyring_error(&error)),
            Self::File(path) => write_atomically(path, text.as_bytes(), true),
        }
    }

    /// Removes the credential. Returns `false` when there was none.
    pub fn delete(&self) -> Result<bool> {
        match self {
            Self::Keyring => match entry()?.delete_credential() {
                Ok(()) => Ok(true),
                Err(keyring::Error::NoEntry) => Ok(false),
                Err(error) => Err(keyring_error(&error)),
            },
            Self::File(path) => match fs::remove_file(path) {
                Ok(()) => Ok(true),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
                Err(error) => Err(Error::io(format!("cannot remove {}", path.display()), error)),
            },
        }
    }
}

fn entry() -> Result<keyring::Entry> {
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER).map_err(|error| keyring_error(&error))
}

fn keyring_error(error: &keyring::Error) -> Error {
    Error::Credentials(format!(
        "the system keychain is not available ({error}). \
         Set VISOR_CREDENTIAL_STORE=file to keep the token in a file, or set VISOR_TOKEN"
    ))
}

/// Where the token of this run came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    Env,
    Store(Store),
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Env => f.write_str("the VISOR_TOKEN environment variable"),
            Self::Store(store) => store.fmt(f),
        }
    }
}

/// Finds the token: `VISOR_TOKEN` first, then the store.
pub fn resolve(store: &Store) -> Result<Option<(Credential, Source)>> {
    if let Some(token) = std::env::var("VISOR_TOKEN")
        .ok()
        .filter(|token| !token.trim().is_empty())
    {
        let credential = Credential::PersonalToken {
            token: token.trim().to_owned(),
        };
        return Ok(Some((credential, Source::Env)));
    }
    Ok(store
        .load()?
        .map(|credential| (credential, Source::Store(store.clone()))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_store_round_trips_and_is_private() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::File(dir.path().join(FILE_NAME));
        let credential = Credential::PersonalToken {
            token: "visor_pat_abcd_secret".into(),
        };

        assert_eq!(store.load().unwrap(), None);
        store.save(&credential).unwrap();
        assert_eq!(store.load().unwrap(), Some(credential));

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(dir.path().join(FILE_NAME))
                .unwrap()
                .permissions()
                .mode();
            assert_eq!(mode & 0o777, 0o600);
        }

        assert!(store.delete().unwrap());
        assert!(!store.delete().unwrap());
    }

    #[test]
    fn stores_the_credential_kind() {
        let credential = Credential::PersonalToken { token: "t".into() };

        assert_eq!(
            serde_json::to_string(&credential).unwrap(),
            r#"{"kind":"personal_token","token":"t"}"#
        );
    }

    #[test]
    fn debug_output_never_shows_the_secret() {
        let credential = Credential::PersonalToken {
            token: "visor_pat_abcd2345_supersecret".into(),
        };

        let debug = format!("{credential:?}");
        assert!(!debug.contains("supersecret"), "{debug}");
        assert!(debug.contains("visor_pat_abcd2345_…"), "{debug}");
    }

    #[test]
    fn redacts_unknown_token_shapes_entirely() {
        assert_eq!(redact("something-else"), "…");
    }
}
