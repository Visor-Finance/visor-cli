//! A picker for one choice on the terminal: each option with a line on what
//! it does, the highlighted one marked `❯`, ↑ ↓ to move, Enter to confirm.
//! Once picked, the menu collapses to one `✓ question  answer` line.
//!
//! Without a real terminal on standard input and standard error (a wrapper
//! that sets `VISOR_FORCE_TTY`, or a test), it asks for the number on a line
//! instead, so the answer can be typed or piped.

use std::io::{self, IsTerminal, Write};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal;

use crate::error::{Error, Result};
use crate::output::Paint;

/// One option: what it is and, optionally, what picking it does.
#[derive(Debug, Clone)]
pub struct Choice {
    pub title: String,
    pub detail: Option<String>,
}

impl Choice {
    #[must_use]
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            detail: None,
        }
    }

    #[must_use]
    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

const FOOTER: &str = "↑ ↓ choose · Enter confirm · Esc cancel";

/// Asks `question` and returns the index of the picked choice; the first
/// one is highlighted. Call only when someone can answer
/// (`output::interactive()`).
pub fn choose(question: &str, choices: &[Choice]) -> Result<usize> {
    assert!(!choices.is_empty(), "a picker needs choices");
    crate::activity::stop();
    let picked = if io::stdin().is_terminal() && io::stderr().is_terminal() {
        choose_with_keys(question, choices)?
    } else {
        choose_with_number(question, choices)?
    };
    let paint = Paint::stderr();
    eprintln!(
        "{} {}  {}",
        paint.ok(),
        paint.bold(question),
        paint.accent(&choices[picked].title)
    );
    Ok(picked)
}

/// The menu as lines, `picked` highlighted, each cut to `width` columns so
/// none wraps (a wrapped line would break the redraw).
fn render(question: &str, choices: &[Choice], picked: usize, width: usize, paint: Paint) -> Vec<String> {
    let fit = |text: &str, indent: usize| -> String {
        let room = width.saturating_sub(indent).max(1);
        if text.chars().count() <= room {
            text.to_owned()
        } else {
            let mut cut: String = text.chars().take(room.saturating_sub(1)).collect();
            cut.push('…');
            cut
        }
    };
    let mut lines = vec![paint.bold(&fit(question, 0)), String::new()];
    for (index, choice) in choices.iter().enumerate() {
        if index == picked {
            lines.push(paint.highlight(&format!("❯ {}", fit(&choice.title, 2))));
            if let Some(detail) = &choice.detail {
                lines.push(format!("    {}", fit(detail, 4)));
            }
        } else {
            lines.push(format!("  {}", fit(&choice.title, 2)));
            if let Some(detail) = &choice.detail {
                lines.push(format!("    {}", paint.dim(&fit(detail, 4))));
            }
        }
    }
    lines.push(String::new());
    lines.push(paint.dim(&fit(FOOTER, 0)));
    lines
}

/// Restores the terminal however the picker ends, a panic included.
struct RawMode;

impl RawMode {
    fn on() -> Result<Self> {
        terminal::enable_raw_mode()
            .map_err(|error| Error::io("cannot read keys from the terminal", error))?;
        eprint!("\x1b[?25l");
        Ok(Self)
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        eprint!("\x1b[?25h");
        let _ = terminal::disable_raw_mode();
    }
}

fn choose_with_keys(question: &str, choices: &[Choice]) -> Result<usize> {
    let paint = Paint::stderr();
    // Some terminals (and a fresh pty) report 0 columns; assume 80 then.
    let width = terminal::size()
        .ok()
        .map(|(columns, _)| usize::from(columns))
        .filter(|columns| *columns > 0)
        .unwrap_or(80);
    let mut picked = 0;
    let mut drawn = 0;
    let _raw = RawMode::on()?;

    let outcome = loop {
        let lines = render(question, choices, picked, width, paint);
        redraw(&lines, drawn);
        drawn = lines.len();

        let Event::Key(KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press | KeyEventKind::Repeat,
            ..
        }) = event::read().map_err(|error| Error::io("cannot read keys from the terminal", error))?
        else {
            continue;
        };
        match code {
            KeyCode::Up | KeyCode::Char('k') | KeyCode::BackTab => {
                picked = (picked + choices.len() - 1) % choices.len();
            }
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => picked = (picked + 1) % choices.len(),
            KeyCode::Char(digit @ '1'..='9') => {
                let index = digit as usize - '1' as usize;
                if index < choices.len() {
                    picked = index;
                }
            }
            KeyCode::Enter => break Ok(picked),
            KeyCode::Esc => break Err(Error::Canceled),
            KeyCode::Char('c' | 'd') if modifiers.contains(KeyModifiers::CONTROL) => {
                break Err(Error::Canceled);
            }
            _ => {}
        }
    };
    redraw(&[], drawn);
    outcome
}

/// Replaces the `drawn` lines printed last with `lines`. In raw mode a line
/// break needs its own carriage return.
fn redraw(lines: &[String], drawn: usize) {
    let mut stderr = io::stderr().lock();
    if drawn > 1 {
        let _ = write!(stderr, "\x1b[{}A", drawn - 1);
    }
    let _ = write!(stderr, "\r\x1b[J{}", lines.join("\r\n"));
    let _ = stderr.flush();
}

fn choose_with_number(question: &str, choices: &[Choice]) -> Result<usize> {
    let paint = Paint::stderr();
    eprintln!("\n{}", paint.bold(question));
    for (index, choice) in choices.iter().enumerate() {
        eprintln!("  {} {}", paint.bold(&(index + 1).to_string()), choice.title);
        if let Some(detail) = &choice.detail {
            eprintln!("    {}", paint.dim(detail));
        }
    }
    loop {
        eprint!("Choose [1]: ");
        let mut answer = String::new();
        let read = io::stdin()
            .read_line(&mut answer)
            .map_err(|error| Error::io("cannot read the answer", error))?;
        let answer = answer.trim();
        if read == 0 || answer.is_empty() {
            return Ok(0);
        }
        match answer.parse::<usize>() {
            Ok(number) if (1..=choices.len()).contains(&number) => return Ok(number - 1),
            _ => eprintln!("Type a number from 1 to {}.", choices.len()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn choices() -> Vec<Choice> {
        vec![
            Choice::new("On your phone").detail("Shows a QR code to scan with the camera."),
            Choice::new("On this computer").detail("Opens the page in your browser."),
        ]
    }

    #[test]
    fn highlights_the_picked_choice_and_dims_the_rest() {
        let lines = render("Where?", &choices(), 1, 80, Paint::plain());

        assert_eq!(
            lines,
            [
                "Where?",
                "",
                "  On your phone",
                "    Shows a QR code to scan with the camera.",
                "❯ On this computer",
                "    Opens the page in your browser.",
                "",
                FOOTER,
            ]
        );
    }

    #[test]
    fn cuts_lines_to_the_terminal_width() {
        let lines = render("Where do you want to confirm?", &choices(), 0, 20, Paint::plain());

        assert!(lines.iter().all(|line| line.chars().count() <= 20), "{lines:#?}");
        assert_eq!(lines[0], "Where do you want t…");
    }
}
