//! `visor update`: replaces the CLI through whatever installed it, never
//! behind its back. A Homebrew binary overwritten in place would leave brew
//! believing the old version is installed, like `gh` and `fly` avoid.

use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

use serde_json::json;

use crate::cli::UpdateArgs;
use crate::client;
use crate::error::{Error, Result};
use crate::output::{self, Format};
use crate::update::{self, CARGO_INSTALL, FORMULA, Install};

pub fn run(args: &UpdateArgs, format: Format) -> Result<()> {
    let current = env!("CARGO_PKG_VERSION");
    crate::activity::start("Checking for a new version…");
    let latest = update::latest_release(Duration::from_secs(10)).ok_or(Error::UpdateCheck)?;
    let install = Install::detect();
    let available = update::newer(&latest, current);

    if args.check || !available {
        report(format, current, &latest, available, &install, false)?;
        return if args.check && available {
            Err(Error::UpdateAvailable { latest })
        } else {
            Ok(())
        };
    }

    if install.can_update() {
        let with = if matches!(install, Install::Homebrew { .. }) {
            "Homebrew"
        } else {
            "the installer"
        };
        output::note(&format!("Updating visor {current} → {latest} with {with}…"));
    }
    let output = if format == Format::Json {
        UpdateOutput::Stderr
    } else {
        UpdateOutput::Terminal
    };
    install_update(&install, &latest, output)?;
    report(format, current, &latest, available, &install, true)
}

/// Where the output of Homebrew or the installer goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateOutput {
    Terminal,
    /// Standard error, so standard output stays JSON.
    Stderr,
    /// Nowhere: an automatic update under `--json`.
    Quiet,
}

/// Installs release `latest` through whatever installed this binary, then
/// checks that the installed binary is that version.
pub fn install_update(install: &Install, latest: &str, output: UpdateOutput) -> Result<()> {
    match install {
        Install::Homebrew { brew } => {
            let mut command = Command::new(brew);
            command.args(["upgrade", FORMULA]);
            run_step(command, &format!("brew upgrade {FORMULA}"), output)?;
        }
        Install::Installer { dir } => run_installer(latest, dir, output)?,
        Install::Cargo => {
            return Err(Error::CannotUpdate(format!(
                "visor {latest} is out, and this visor was installed with cargo. Update it with:\n  {CARGO_INSTALL}"
            )));
        }
        Install::Unknown { path } => {
            return Err(Error::CannotUpdate(format!(
                "visor {latest} is out, but {} was not installed by Homebrew or the installer, so `visor update` cannot replace it. Reinstall it: https://github.com/Visor-Finance/visor-cli#install",
                path.display()
            )));
        }
    }
    verify(install, latest)
}

/// Fails unless the installed binary reports `latest`. The Homebrew tap is
/// published minutes after the GitHub release, and `brew upgrade` then
/// succeeds without installing anything.
fn verify(install: &Install, latest: &str) -> Result<()> {
    let Some(binary) = install.binary() else {
        return Ok(());
    };
    let found = Command::new(&binary)
        .arg("--version")
        .env("VISOR_NO_UPDATE_NOTIFIER", "1")
        .env_remove("VISOR_UPDATE_CHECK")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|text| text.trim().trim_start_matches("visor ").to_owned());
    match found {
        Some(version) if version == latest => Ok(()),
        found => {
            let found = found.map_or_else(
                || "cannot be run".to_owned(),
                |version| format!("is visor {version}"),
            );
            let hint = if matches!(install, Install::Homebrew { .. }) {
                " Homebrew may not have it yet; try again in a few minutes."
            } else {
                ""
            };
            Err(Error::UpdateFailed(format!(
                "visor {latest} was not installed: {} {found}.{hint}",
                binary.display()
            )))
        }
    }
}

fn report(
    format: Format,
    current: &str,
    latest: &str,
    available: bool,
    install: &Install,
    updated: bool,
) -> Result<()> {
    if format == Format::Json {
        return output::json(&json!({
            "current": current,
            "latest": latest,
            "update_available": available && !updated,
            "updated": updated,
            "installed_with": install.name(),
        }));
    }
    let paint = output::Paint::stdout();
    if updated {
        output::line(&format!(
            "{} Updated visor to {}. What changed: {}",
            paint.ok(),
            paint.bold(latest),
            update::release_page(latest)
        ))
    } else if available {
        output::print_fields(&[
            ("Installed", current.to_owned()),
            ("Latest", latest.to_owned()),
            ("Installed with", install.name().to_owned()),
            ("What changed", update::release_page(latest)),
        ])
    } else {
        output::line(&format!("{} visor {current} is the latest version.", paint.ok()))
    }
}

/// Downloads the installer of release `version` and runs it into `dir`, the
/// directory of this binary, without touching PATH again.
fn run_installer(version: &str, dir: &Path, output: UpdateOutput) -> Result<()> {
    if output != UpdateOutput::Quiet {
        crate::activity::start("Downloading the installer…");
    }
    let url = update::installer_url(version);
    let script = client::http_with_timeout(Duration::from_secs(60))?
        .get(&url)
        .send()
        .and_then(reqwest::blocking::Response::error_for_status)
        .and_then(reqwest::blocking::Response::bytes)
        .map_err(Error::Network)?;
    let extension = if cfg!(windows) { "ps1" } else { "sh" };
    let path = std::env::temp_dir().join(format!("visor-installer-{}.{extension}", uuid::Uuid::new_v4()));
    std::fs::write(&path, &script).map_err(|error| Error::io("cannot save the installer", error))?;

    let mut command = if cfg!(windows) {
        let mut command = Command::new("powershell");
        command.args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"]);
        command
    } else {
        Command::new("sh")
    };
    command
        .arg(&path)
        .env("VISOR_UNMANAGED_INSTALL", dir)
        .env("VISOR_NO_MODIFY_PATH", "1");

    // Windows cannot overwrite a running .exe but can rename it: move it
    // aside for the installer and put it back if the installer fails.
    let aside = cfg!(windows).then(|| dir.join("visor.exe.old"));
    if let Some(aside) = &aside {
        let _ = std::fs::remove_file(aside);
        std::fs::rename(dir.join("visor.exe"), aside)
            .map_err(|error| Error::io("cannot move the running visor.exe aside", error))?;
    }
    let result = run_step(command, &format!("the installer from {url}"), output);
    if let (Err(_), Some(aside)) = (&result, &aside) {
        let _ = std::fs::rename(aside, dir.join("visor.exe"));
    }
    let _ = std::fs::remove_file(&path);
    result
}

/// Runs an update step with its output where `output` says.
fn run_step(mut command: Command, name: &str, output: UpdateOutput) -> Result<()> {
    crate::activity::stop();
    match output {
        UpdateOutput::Terminal => {}
        UpdateOutput::Stderr => {
            command.stdout(Stdio::from(std::io::stderr()));
        }
        UpdateOutput::Quiet => {
            command
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null());
        }
    }
    let status = command
        .status()
        .map_err(|error| Error::UpdateFailed(format!("cannot run {name}: {error}")))?;
    if status.success() {
        Ok(())
    } else if output == UpdateOutput::Quiet {
        Err(Error::UpdateFailed(format!(
            "{name} failed ({status}). visor was not updated"
        )))
    } else {
        Err(Error::UpdateFailed(format!(
            "{name} failed ({status}); its output above says why. visor was not updated"
        )))
    }
}
