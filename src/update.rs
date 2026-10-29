//! The new-version notice, like `gh`: at most one check a day, run beside
//! the command so it never slows it down, and shown only to a person. Also
//! how this binary was installed, which `visor update` follows, and the
//! automatic updates that run it, like flyctl's (see [`auto`]).

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
const ATTEMPT_FILE: &str = "auto-update.json";
const LOG_FILE: &str = "update.log";
/// Like flyctl: at most one release check an hour.
const CHECK_EVERY_SECONDS: i64 = 60 * 60;
/// At most one automatic update attempt an hour, so a broken Homebrew does
/// not run on every command.
const ATTEMPT_EVERY_SECONDS: i64 = 60 * 60;
/// How long a release check may take. The command waits for it at the end
/// (like flyctl's tasks), at most once an hour: a thread still running when
/// the command exits dies with it, and the cache would never fill.
const CHECK_TIMEOUT: Duration = Duration::from_secs(2);

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

/// True when `latest` is far enough ahead to update before the command,
/// like flyctl's `SignificantlyBehind`: a new major or minor version, or
/// more than five patches.
#[must_use]
pub fn far_behind(latest: &str, current: &str) -> bool {
    let parts = |version: &str| -> [u64; 3] {
        let mut parts = version.split('.').map(|part| part.parse().unwrap_or(0));
        [(); 3].map(|()| parts.next().unwrap_or(0))
    };
    let ([major, minor, patch], [current_major, current_minor, current_patch]) =
        (parts(latest), parts(current));
    (major, minor) > (current_major, current_minor)
        || ((major, minor) == (current_major, current_minor) && patch > current_patch + 5)
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

    /// True for the installs `visor update` replaces itself.
    #[must_use]
    pub fn can_update(&self) -> bool {
        matches!(self, Self::Homebrew { .. } | Self::Installer { .. })
    }

    /// The path that runs this install after an update. Homebrew links
    /// `<prefix>/bin/visor` to the new version; the old one may be gone.
    #[must_use]
    pub fn binary(&self) -> Option<PathBuf> {
        let name = if cfg!(windows) { "visor.exe" } else { "visor" };
        match self {
            Self::Homebrew { brew } => Some(brew.with_file_name(name)),
            Self::Installer { dir } => Some(dir.join(name)),
            Self::Cargo | Self::Unknown { .. } => None,
        }
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
/// `VISOR_UPDATE_CHECK=1` forces it, like `FLY_UPDATE_CHECK`.
fn wanted() -> bool {
    let set = |name| std::env::var_os(name).is_some_and(|value| !value.is_empty());
    set("VISOR_UPDATE_CHECK")
        || (std::io::stderr().is_terminal() && !set("CI") && !set("VISOR_NO_UPDATE_NOTIFIER"))
}

/// The latest version from the last check, without a request.
fn cached_latest(config_dir: &Path) -> Option<String> {
    let text = std::fs::read_to_string(config_dir.join(CACHE_FILE)).ok()?;
    serde_json::from_str::<Cache>(&text)
        .ok()
        .map(|cache| cache.latest)
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
                if let Some(latest) = latest_release(CHECK_TIMEOUT) {
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

/// After the command: updates in the background when automatic updates
/// may, else shows the notice, if the check finished and found a newer
/// version. A check in flight gets up to [`CHECK_TIMEOUT`] to finish.
pub fn finish(receiver: Option<Receiver<String>>, config_dir: Option<&Path>, json: bool) {
    let Some(latest) = receiver.and_then(|receiver| receiver.recv_timeout(CHECK_TIMEOUT).ok()) else {
        return;
    };
    if config_dir.is_some_and(|dir| auto::in_background(dir, &latest)) || json {
        return;
    }
    if let Some(message) = notice(&latest) {
        eprintln!("\n{message}");
    }
}

/// Automatic updates, like flyctl's `promptAndAutoUpdate`: on by default,
/// only for a person at a terminal, and only for an install `visor update`
/// can replace. A version far ahead is installed before the command, which
/// then runs again in the new binary; a newer patch is installed in the
/// background after it (not on Windows, where flyctl only does the first).
/// Unlike flyctl, a failed update never stops the command.
pub mod auto {
    use std::io::IsTerminal;
    use std::path::Path;
    use std::process::{Command, ExitCode, Stdio};

    use serde::{Deserialize, Serialize};

    use super::{ATTEMPT_EVERY_SECONDS, ATTEMPT_FILE, Install, LOG_FILE, cached_latest, far_behind, newer};
    use crate::config::{Config, write_atomically};
    use crate::output::Paint;

    #[derive(Serialize, Deserialize)]
    struct Attempt {
        attempted_at: i64,
    }

    /// True when automatic updates may run: the notice may, standard output
    /// is a terminal too (a pipe never sees the binary change), and the
    /// config does not turn them off.
    fn enabled(config_dir: &Path) -> bool {
        let forced = std::env::var_os("VISOR_UPDATE_CHECK").is_some_and(|value| !value.is_empty());
        super::wanted()
            && (forced || std::io::stdout().is_terminal())
            && Config::load(config_dir).map_or(true, |config| config.auto_update())
    }

    /// Records an attempt unless one ran within the hour.
    fn claim_attempt(config_dir: &Path) -> bool {
        let path = config_dir.join(ATTEMPT_FILE);
        let now = chrono::Utc::now().timestamp();
        let last = std::fs::read_to_string(&path)
            .ok()
            .and_then(|text| serde_json::from_str::<Attempt>(&text).ok());
        if last.is_some_and(|last| now - last.attempted_at < ATTEMPT_EVERY_SECONDS) {
            return false;
        }
        let text = serde_json::to_vec(&Attempt { attempted_at: now }).expect("attempt serializes");
        write_atomically(&path, &text, false).is_ok()
    }

    /// Before the command: when the last check found a version far ahead,
    /// installs it and runs the command again in the new binary, returning
    /// that run's exit code. `None` means: run the command here.
    #[must_use]
    pub fn before(config_dir: &Path, json: bool) -> Option<ExitCode> {
        let current = env!("CARGO_PKG_VERSION");
        if !enabled(config_dir) {
            return None;
        }
        let latest = cached_latest(config_dir)
            .filter(|latest| newer(latest, current) && far_behind(latest, current))?;
        let install = Install::detect();
        if !install.can_update() || !claim_attempt(config_dir) {
            return None;
        }

        let paint = Paint::stderr();
        if !json {
            eprintln!(
                "{} Automatically updating visor {current} → {}.",
                paint.warn(),
                paint.bold(&latest)
            );
        }
        let output = if json {
            crate::commands::UpdateOutput::Quiet
        } else {
            crate::commands::UpdateOutput::Stderr
        };
        if let Err(error) = crate::commands::install_update(&install, &latest, output) {
            if !json {
                eprintln!(
                    "{} The automatic update failed: {error}\n  Running this command with visor {current}. Run `visor update` to try again.\n",
                    paint.warn()
                );
            }
            return None;
        }

        let binary = install.binary()?;
        // Like flyctl's `Relaunch`: the same arguments, without a second check.
        match Command::new(&binary)
            .args(std::env::args_os().skip(1))
            .env("VISOR_NO_UPDATE_NOTIFIER", "1")
            .env_remove("VISOR_UPDATE_CHECK")
            .status()
        {
            Ok(status) => Some(ExitCode::from(
                status
                    .code()
                    .and_then(|code| u8::try_from(code).ok())
                    .unwrap_or(1),
            )),
            Err(error) => {
                if !json {
                    eprintln!(
                        "{} Updated, but cannot run {}: {error}",
                        paint.warn(),
                        binary.display()
                    );
                }
                None
            }
        }
    }

    /// After the command: starts `visor update` in the background for a
    /// newer version, with its output in `update.log`. True when the update
    /// is handled (started now or within the hour), so no notice is needed.
    #[must_use]
    pub fn in_background(config_dir: &Path, latest: &str) -> bool {
        if cfg!(windows) || !enabled(config_dir) || !newer(latest, env!("CARGO_PKG_VERSION")) {
            return false;
        }
        if !Install::detect().can_update() {
            return false;
        }
        if !claim_attempt(config_dir) {
            return true;
        }
        let Ok(exe) = std::env::current_exe() else {
            return false;
        };
        let Ok(log) = std::fs::File::create(config_dir.join(LOG_FILE)) else {
            return false;
        };
        let Ok(log_err) = log.try_clone() else {
            return false;
        };
        let mut command = Command::new(exe);
        command
            .arg("update")
            .stdin(Stdio::null())
            .stdout(log)
            .stderr(log_err)
            .env("VISOR_NO_UPDATE_NOTIFIER", "1")
            .env_remove("VISOR_UPDATE_CHECK");
        // Its own process group, so Ctrl+C in the shell does not stop it.
        #[cfg(unix)]
        std::os::unix::process::CommandExt::process_group(&mut command, 0);
        command.spawn().is_ok()
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
    fn far_behind_is_a_new_minor_or_more_than_five_patches() {
        assert!(far_behind("0.7.0", "0.6.0"));
        assert!(far_behind("1.0.0", "0.9.9"));
        assert!(far_behind("0.6.6", "0.6.0"));
        assert!(!far_behind("0.6.5", "0.6.0"));
        assert!(!far_behind("0.6.1", "0.6.0"));
        assert!(!far_behind("0.6.0", "0.6.0"));
    }

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
