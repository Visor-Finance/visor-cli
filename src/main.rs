use std::process::ExitCode;

use clap::Parser;

fn main() -> ExitCode {
    let cli = visor::cli::Cli::parse();
    match visor::commands::run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            error.exit_code()
        }
    }
}
