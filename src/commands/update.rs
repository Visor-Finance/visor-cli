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

    match &install {
        Install::Homebrew { brew } => {
            output::note(&format!("Updating visor {current} → {latest} with Homebrew…"));
            let mut command = Command::new(brew);
            command.args(["upgrade", FORMULA]);
            run_step(command, &format!("brew upgrade {FORMULA}"), format)?;
        }
        Install::Installer { dir } => {
            output::note(&format!(
                "Updating visor {current} → {latest} with the installer…"
            ));
            run_installer(&latest, dir, format)?;
        }
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
    report(format, current, &latest, available, &install, true)
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
fn run_installer(version: &str, dir: &Path, format: Format) -> Result<()> {
    crate::activity::start("Downloading the installer…");
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
    let result = run_step(command, &format!("the installer from {url}"), format);
    if let (Err(_), Some(aside)) = (&result, &aside) {
        let _ = std::fs::rename(aside, dir.join("visor.exe"));
    }
    let _ = std::fs::remove_file(&path);
    result
}

/// Runs an update step with its output on the terminal. With `--json` its
/// output goes to standard error, so standard output stays JSON.
fn run_step(mut command: Command, name: &str, format: Format) -> Result<()> {
    crate::activity::stop();
    if format == Format::Json {
        command.stdout(Stdio::from(std::io::stderr()));
    }
    let status = command
        .status()
        .map_err(|error| Error::UpdateFailed(format!("cannot run {name}: {error}")))?;
    if status.success() {
        Ok(())
    } else {
        Err(Error::UpdateFailed(format!(
            "{name} failed ({status}); its output above says why. visor was not updated"
        )))
    }
}
