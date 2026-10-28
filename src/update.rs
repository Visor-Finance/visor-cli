//! The new-version notice, like `gh`: at most one check a day, run beside
//! the command so it never slows it down, and shown only to a person. Also
//! how this binary was installed, which `visor update` follows.

use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::client;
use crate::config::write_atomically;
use crate::output::Paint;

const RELEASES_URL: &str = "https://api.github.com/repos/Visor-Finance/visor-cli/releases/latest";
const DOWNLOAD_URL: &str = "https://github.com/Visor-Finance/visor-cli/releases/download";
pub const UPGRADE: &str = "run `visor update`";
pub const FORMULA: &str = "visor-finance/tap/visor";
pub const CARGO_INSTALL: &str =
    "cargo install --git https://github.com/Visor-Finance/visor-cli --locked --force";
const CACHE_FILE: &str = "update-check.json";
const CHECK_EVERY_SECONDS: i64 = 24 * 60 * 60;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Cache {
    checked_at: i64,
    latest: String,
}

/// The latest release version, such as `0.3.0`, or `None` when GitHub does
/// not answer within `timeout`.
#[must_use]
pub fn latest_release(timeout: Duration) -> Option<String> {
    #[derive(Deserialize)]
    struct Release {
        tag_name: String,
    }
    let url = std::env::var("VISOR_RELEASES_URL").unwrap_or_else(|_| RELEASES_URL.to_owned());
    let release: Release = client::http_with_timeout(timeout)
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

/// The URL of the installer script of release `version`, for this platform.
#[must_use]
pub fn installer_url(version: &str) -> String {
    let base = std::env::var("VISOR_RELEASE_DOWNLOAD_URL").unwrap_or_else(|_| DOWNLOAD_URL.to_owned());
    let extension = if cfg!(windows) { "ps1" } else { "sh" };
    format!("{base}/v{version}/visor-installer.{extension}")
}

/// The page of release `version`.
#[must_use]
pub fn release_page(version: &str) -> String {
    format!("https://github.com/Visor-Finance/visor-cli/releases/tag/v{version}")
}

/// How this binary was installed, which decides how `visor update`
/// replaces it. Without an install receipt (dist writes one only with its
/// updater), the binary's location tells.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Install {
    /// `<prefix>/Cellar/visor/<version>/bin/visor`: the `brew` of that prefix upgrades it.
    Homebrew { brew: PathBuf },
    /// The shell or PowerShell installer put it in `dir` (`$CARGO_HOME/bin`).
    Installer { dir: PathBuf },
    /// `cargo install`, which records it in `$CARGO_HOME/.crates2.json`.
    Cargo,
    /// A build or a package the CLI does not know how to replace.
    Unknown { path: PathBuf },
}

impl Install {
    /// Where the running binary came from.
    #[must_use]
    pub fn detect() -> Self {
        let Ok(exe) = std::env::current_exe() else {
            return Self::Unknown {
                path: PathBuf::from("visor"),
            };
        };
        let exe = exe.canonicalize().unwrap_or(exe);
        let cargo_home = std::env::var_os("CARGO_HOME")
            .filter(|home| !home.is_empty())
            .map(PathBuf::from)
            .or_else(|| dirs::home_dir().map(|home| home.join(".cargo")));
        detect_at(&exe, cargo_home.as_deref())
    }

    /// A stable name for `--json` and the messages.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Homebrew { .. } => "homebrew",
            Self::Installer { .. } => "installer",
            Self::Cargo => "cargo",
            Self::Unknown { .. } => "unknown",
        }
    }
}

fn detect_at(exe: &Path, cargo_home: Option<&Path>) -> Install {
    let is = |path: &Path, name: &str| path.file_name().is_some_and(|file| file == name);
    for dir in exe.ancestors() {
        if is(dir, "visor")
            && let Some(cellar) = dir.parent().filter(|cellar| is(cellar, "Cellar"))
            && let Some(prefix) = cellar.parent()
        {
            return Install::Homebrew {
                brew: prefix.join("bin").join("brew"),
            };
        }
    }
    if let (Some(dir), Some(home)) = (exe.parent(), cargo_home) {
        let bin = home.join("bin");
        if dir == bin || bin.canonicalize().is_ok_and(|bin| dir == bin) {
            return if installed_by_cargo(home) {
                Install::Cargo
            } else {
                Install::Installer { dir: dir.to_owned() }
            };
        }
    }
    Install::Unknown { path: exe.to_owned() }
}

/// True when cargo's install list has the `visor` package.
fn installed_by_cargo(cargo_home: &Path) -> bool {
    let Ok(text) = std::fs::read_to_string(cargo_home.join(".crates2.json")) else {
        return false;
    };
    serde_json::from_str::<serde_json::Value>(&text).is_ok_and(|list| {
        list["installs"]
            .as_object()
            .is_some_and(|installs| installs.keys().any(|key| key.starts_with("visor ")))
    })
}

/// True when the dotted version `a` is newer than `b`.
#[must_use]
pub fn newer(a: &str, b: &str) -> bool {
    let parts =
        |version: &str| -> Vec<u64> { version.split('.').map(|part| part.parse().unwrap_or(0)).collect() };
    parts(a) > parts(b)
}

/// True when the notice may run: a person at a terminal, outside CI.
fn wanted() -> bool {
    let set = |name| std::env::var_os(name).is_some_and(|value| !value.is_empty());
    std::io::stderr().is_terminal() && !set("CI") && !set("VISOR_NO_UPDATE_NOTIFIER")
}

/// Starts the check. The receiver yields the latest version once known.
#[must_use]
pub fn start(config_dir: &Path) -> Option<Receiver<String>> {
    if !wanted() {
        return None;
    }
    Some(start_at(
        config_dir.join(CACHE_FILE),
        chrono::Utc::now().timestamp(),
    ))
}

fn start_at(cache_path: PathBuf, now: i64) -> Receiver<String> {
    let (sender, receiver) = mpsc::channel();
    let cached = std::fs::read_to_string(&cache_path)
        .ok()
        .and_then(|text| serde_json::from_str::<Cache>(&text).ok());
    match cached {
        Some(cache) if now - cache.checked_at < CHECK_EVERY_SECONDS => {
            let _ = sender.send(cache.latest);
        }
        _ => {
            thread::spawn(move || {
                if let Some(latest) = latest_release(Duration::from_secs(3)) {
                    let cache = Cache {
                        checked_at: now,
                        latest: latest.clone(),
                    };
                    let text = serde_json::to_vec(&cache).expect("cache serializes");
                    let _ = write_atomically(&cache_path, &text, false);
                    let _ = sender.send(latest);
                }
            });
        }
    }
    receiver
}

/// Shows the notice if the check finished and found a newer version. It
/// never waits: a slow check is shown on a later run, from the cache.
pub fn finish(receiver: Option<Receiver<String>>) {
    if let Some(message) = receiver
        .and_then(|receiver| receiver.try_recv().ok())
        .and_then(|latest| notice(&latest))
    {
        eprintln!("\n{message}");
    }
}

fn notice(latest: &str) -> Option<String> {
    let current = env!("CARGO_PKG_VERSION");
    newer(latest, current).then(|| {
        let paint = Paint::stderr();
        format!(
            "{} visor {} is out (you have {current}). Upgrade: {UPGRADE}\n  What changed: {}",
            paint.warn(),
            paint.bold(latest),
            release_page(latest)
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_versions_by_number() {
        assert!(newer("0.10.0", "0.9.1"));
        assert!(newer("1.0.0", "0.99.99"));
        assert!(!newer("0.2.0", "0.2.0"));
        assert!(!newer("0.1.9", "0.2.0"));
    }

    #[test]
    fn a_fresh_cache_answers_without_a_request() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(CACHE_FILE);
        std::fs::write(&path, r#"{"checked_at":1000,"latest":"9.9.9"}"#).unwrap();

        let receiver = start_at(path, 1000 + 60);

        assert_eq!(receiver.try_recv().unwrap(), "9.9.9");
    }

    #[test]
    fn a_cellar_binary_is_homebrew_with_the_brew_of_its_prefix() {
        let install = detect_at(Path::new("/opt/homebrew/Cellar/visor/0.5.0/bin/visor"), None);

        assert_eq!(
            install,
            Install::Homebrew {
                brew: PathBuf::from("/opt/homebrew/bin/brew")
            }
        );
    }

    #[test]
    fn a_binary_in_cargo_bin_is_the_installer_unless_cargo_lists_it() {
        let home = tempfile::tempdir().unwrap();
        let bin = home.path().join("bin");
        std::fs::create_dir(&bin).unwrap();
        let exe = bin.join("visor");

        assert_eq!(
            detect_at(&exe, Some(home.path())),
            Install::Installer { dir: bin.clone() }
        );

        std::fs::write(
            home.path().join(".crates2.json"),
            r#"{"installs":{"visor 0.5.0 (git+https://github.com/Visor-Finance/visor-cli#677cfbb)":{}}}"#,
        )
        .unwrap();
        assert_eq!(detect_at(&exe, Some(home.path())), Install::Cargo);
    }

    #[test]
    fn any_other_binary_is_unknown() {
        let exe = Path::new("/home/ana/visor-cli/target/release/visor");

        assert_eq!(
            detect_at(exe, Some(Path::new("/home/ana/.cargo"))),
            Install::Unknown { path: exe.to_owned() }
        );
    }

    #[test]
    fn notices_only_a_newer_version() {
        assert!(notice("999.0.0").unwrap().contains("visor"));
        assert!(notice(env!("CARGO_PKG_VERSION")).is_none());
        assert!(notice("0.0.1").is_none());
    }
}
