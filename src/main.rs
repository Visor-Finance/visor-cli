use std::process::ExitCode;

use clap::Parser;
use visor::cli::{Cli, Command};

fn main() -> ExitCode {
    let cli = Cli::parse();
    let json = cli.global.json;

    // `doctor` and `update` check the version themselves, and completions feed a shell.
    let notify = matches!(&cli.command, Some(command) if !matches!(command, Command::Completions { .. } | Command::Doctor | Command::Update(_)));
    let update = notify
        .then(|| visor::config::config_dir().ok())
        .flatten()
        .and_then(|dir| visor::update::start(&dir));

    let code = match visor::commands::run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            visor::activity::stop();
            // With --json, the error is JSON too, on standard error, so a
            // script reads one format; like `resend --json`.
            if json {
                eprintln!(
                    "{}",
                    serde_json::to_string_pretty(&error.to_json()).expect("error serializes")
                );
            } else {
                eprintln!("error: {error}");
            }
            error.exit_code()
        }
    };
    visor::activity::stop();
    visor::update::finish(update);
    code
}
