//! Rendering: tables for people, the unchanged API body for `--json`.
//!
//! Data goes to stdout; notes such as "No accounts found" go to stderr, so a
//! pipe only ever sees data.

use std::io::{self, IsTerminal, Write};

use chrono::{DateTime, Local};
use comfy_table::{ContentArrangement, Table, presets};
use serde_json::Value;

use crate::error::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Table,
    Json,
}

pub fn json(value: &Value) -> Result<()> {
    let text = serde_json::to_string_pretty(value).expect("JSON value serializes");
    line(&text)
}

/// Writes a line to stdout. A closed pipe (`visor tx list | head`) is not an
/// error worth reporting, so it ends the process quietly.
pub fn line(text: &str) -> Result<()> {
    let mut stdout = io::stdout().lock();
    match writeln!(stdout, "{text}") {
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => std::process::exit(0),
        result => result.map_err(|error| Error::io("cannot write to stdout", error)),
    }
}

pub fn note(text: &str) {
    eprintln!("{text}");
}

/// ANSI styling for one stream, off for a pipe or under `NO_COLOR`.
#[derive(Debug, Clone, Copy)]
pub struct Paint {
    on: bool,
}

impl Paint {
    #[must_use]
    pub fn stdout() -> Self {
        Self::when(io::stdout().is_terminal())
    }

    #[must_use]
    pub fn stderr() -> Self {
        Self::when(io::stderr().is_terminal())
    }

    fn when(is_terminal: bool) -> Self {
        Self {
            on: crate::banner::Palette::detect(is_terminal) != crate::banner::Palette::None,
        }
    }

    fn wrap(self, code: &str, text: &str) -> String {
        if self.on {
            format!("\x1b[{code}m{text}\x1b[0m")
        } else {
            text.to_owned()
        }
    }

    #[must_use]
    pub fn bold(self, text: &str) -> String {
        self.wrap("1", text)
    }

    #[must_use]
    pub fn dim(self, text: &str) -> String {
        self.wrap("2", text)
    }

    #[must_use]
    pub fn ok(self) -> String {
        self.wrap("32", "✓")
    }

    #[must_use]
    pub fn fail(self) -> String {
        self.wrap("31", "✗")
    }

    #[must_use]
    pub fn warn(self) -> String {
        self.wrap("33", "!")
    }
}

/// A one-line spinner on standard error. Without a terminal it prints the
/// message once, so logs still say what the CLI waits for.
#[derive(Debug)]
pub struct Spinner {
    message: String,
    frame: usize,
    live: bool,
}

impl Spinner {
    const FRAMES: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

    #[must_use]
    pub fn start(message: &str) -> Self {
        let live = io::stderr().is_terminal();
        let mut spinner = Self {
            message: message.to_owned(),
            frame: 0,
            live,
        };
        if live {
            spinner.tick();
        } else {
            note(message);
        }
        spinner
    }

    pub fn tick(&mut self) {
        if self.live {
            eprint!(
                "\r\x1b[K{} {}",
                Self::FRAMES[self.frame % Self::FRAMES.len()],
                self.message
            );
            self.frame += 1;
        }
    }

    /// Replaces the spinner with `✓ done`.
    pub fn finish(mut self, done: &str) {
        self.clear();
        note(&format!("{} {done}", Paint::stderr().ok()));
    }

    fn clear(&mut self) {
        if self.live {
            eprint!("\r\x1b[K");
            self.live = false;
        }
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        self.clear();
    }
}

#[must_use]
pub fn table(header: &[&str]) -> Table {
    let mut table = Table::new();
    table
        .load_style(presets::NOTHING)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(header.iter().map(|title| title.to_uppercase()));
    table
}

/// Prints a table, or `empty` on stderr when it has no rows.
pub fn print_table(table: &Table, empty: &str) -> Result<()> {
    if table.row_count() == 0 {
        note(empty);
        Ok(())
    } else {
        line(&table.to_string())
    }
}

/// Prints `label  value` pairs for a single record.
pub fn print_fields(fields: &[(&str, String)]) -> Result<()> {
    let mut table = Table::new();
    table.load_style(presets::NOTHING);
    for (label, value) in fields {
        table.add_row(vec![(*label).to_owned(), value.clone()]);
    }
    line(&table.to_string())
}

#[must_use]
pub fn money(amount: &str, currency: &str) -> String {
    format!("{amount} {currency}")
}

#[must_use]
pub fn opt_money(amount: Option<&str>, currency: &str) -> String {
    amount.map_or_else(|| "-".to_owned(), |amount| money(amount, currency))
}

/// A debit shows as negative so a column of amounts reads as a ledger.
#[must_use]
pub fn signed(amount: &str, kind: &str, currency: &str) -> String {
    let sign = if kind == "debit" { "-" } else { "" };
    format!("{sign}{amount} {currency}")
}

#[must_use]
pub fn opt(value: Option<&str>) -> String {
    value.unwrap_or("-").to_owned()
}

#[must_use]
pub fn yes_no(value: bool) -> String {
    if value { "yes" } else { "no" }.to_owned()
}

/// Shows an RFC 3339 time as a local date.
#[must_use]
pub fn local_date(time: &str) -> String {
    DateTime::parse_from_rfc3339(time).map_or_else(
        |_| time.to_owned(),
        |time| time.with_timezone(&Local).format("%Y-%m-%d").to_string(),
    )
}

/// Shows an RFC 3339 time as local date and minutes.
#[must_use]
pub fn local_time(time: Option<&str>) -> String {
    let Some(time) = time else {
        return "-".to_owned();
    };
    DateTime::parse_from_rfc3339(time).map_or_else(
        |_| time.to_owned(),
        |time| time.with_timezone(&Local).format("%Y-%m-%d %H:%M").to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signs_debits() {
        assert_eq!(signed("12.50", "debit", "BRL"), "-12.50 BRL");
        assert_eq!(signed("12.50", "credit", "BRL"), "12.50 BRL");
    }

    #[test]
    fn keeps_unparseable_times() {
        assert_eq!(local_date("soon"), "soon");
        assert_eq!(local_time(None), "-");
    }
}
