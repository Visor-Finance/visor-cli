//! Non-secret settings in `config.toml` (the token never goes here).
//!
//! The directory is `$VISOR_CONFIG_DIR`, else `$XDG_CONFIG_HOME/visor`, else
//! `~/.config/visor` (`%APPDATA%\visor` on Windows).

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

const FILE_NAME: &str = "config.toml";

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    /// The space commands act on when `--space` is not given.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_space: Option<String>,

    /// Automatic updates, on unless set to `false` (`visor settings autoupdate`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_update: Option<bool>,
}

pub fn config_dir() -> Result<PathBuf> {
    if let Some(dir) = non_empty_env("VISOR_CONFIG_DIR") {
        return Ok(PathBuf::from(dir));
    }
    if let Some(dir) = non_empty_env("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(dir).join("visor"));
    }
    let base = if cfg!(windows) {
        dirs::config_dir()
    } else {
        dirs::home_dir().map(|home| home.join(".config"))
    };
    base.map(|dir| dir.join("visor"))
        .ok_or_else(|| Error::usage("cannot find a config directory; set VISOR_CONFIG_DIR"))
}

fn non_empty_env(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|value| !value.is_empty())
}

impl Config {
    /// True unless automatic updates were turned off, like flyctl's default.
    #[must_use]
    pub fn auto_update(&self) -> bool {
        self.auto_update.unwrap_or(true)
    }

    pub fn load(dir: &Path) -> Result<Self> {
        let path = dir.join(FILE_NAME);
        match fs::read_to_string(&path) {
            Ok(text) => toml::from_str(&text)
                .map_err(|error| Error::usage(format!("{} is not valid: {error}", path.display()))),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(error) => Err(Error::io(format!("cannot read {}", path.display()), error)),
        }
    }

    pub fn save(&self, dir: &Path) -> Result<()> {
        let text = toml::to_string_pretty(self).expect("config serializes");
        write_atomically(&dir.join(FILE_NAME), text.as_bytes(), false)
    }
}

/// Writes through a temporary file and a rename so a crash never leaves a
/// half-written file. `private` restricts the file to its owner on Unix.
pub fn write_atomically(path: &Path, contents: &[u8], private: bool) -> Result<()> {
    let dir = path.parent().expect("file path has a parent");
    fs::create_dir_all(dir).map_err(|error| Error::io(format!("cannot create {}", dir.display()), error))?;

    let temp = path.with_extension("tmp");
    let write = || -> std::io::Result<()> {
        let mut options = fs::OpenOptions::new();
        options.write(true).create(true).truncate(true);
        #[cfg(unix)]
        if private {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        #[cfg(not(unix))]
        let _ = private;
        std::io::Write::write_all(&mut options.open(&temp)?, contents)?;
        fs::rename(&temp, path)
    };
    write().map_err(|error| Error::io(format!("cannot write {}", path.display()), error))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_is_the_default_config() {
        let dir = tempfile::tempdir().unwrap();

        assert_eq!(Config::load(dir.path()).unwrap(), Config::default());
    }

    #[test]
    fn round_trips_through_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config {
            default_space: Some("b5e2d0de-0000-4000-8000-000000000001".into()),
            auto_update: Some(false),
        };

        config.save(&dir.path().join("nested")).unwrap();

        assert_eq!(Config::load(&dir.path().join("nested")).unwrap(), config);
    }

    #[test]
    fn automatic_updates_are_on_by_default() {
        assert!(Config::default().auto_update());
        assert!(
            !Config {
                auto_update: Some(false),
                ..Config::default()
            }
            .auto_update()
        );
    }

    #[test]
    fn rejects_an_invalid_file() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(FILE_NAME), "default_space = [").unwrap();

        assert!(matches!(Config::load(dir.path()), Err(Error::Usage(_))));
    }
}
