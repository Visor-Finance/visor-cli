//! The new-version notice, like `gh`: at most one check a day, run beside
//! the command so it never slows it down, and shown only to a person.

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
pub const UPGRADE: &str = "brew upgrade visor-finance/tap/visor, or run the installer again";
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
            "{} visor {} is out (you have {current}). Upgrade: {UPGRADE}\n  What changed: https://github.com/Visor-Finance/visor-cli/releases/tag/v{latest}",
            paint.warn(),
            paint.bold(latest)
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
    fn notices_only_a_newer_version() {
        assert!(notice("999.0.0").unwrap().contains("visor"));
        assert!(notice(env!("CARGO_PKG_VERSION")).is_none());
        assert!(notice("0.0.1").is_none());
    }
}
