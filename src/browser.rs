//! Opening a URL in the default browser.

use std::process::{Command, Stdio};

/// Opens `url` in the default browser, or with the `BROWSER` command when
/// it is set, like `gh`. Returns false when no opener ran.
#[must_use]
pub fn open(url: &str) -> bool {
    let browser = std::env::var("BROWSER").unwrap_or_default();
    let mut words = browser.split_whitespace();
    let mut command = if let Some(program) = words.next() {
        let mut command = Command::new(program);
        command.args(words).arg(url);
        command
    } else if cfg!(target_os = "macos") {
        let mut command = Command::new("open");
        command.arg(url);
        command
    } else if cfg!(windows) {
        // `start` would split the URL at `&`; the URL handler takes it whole.
        let mut command = Command::new("rundll32");
        command.args(["url.dll,FileProtocolHandler", url]);
        command
    } else {
        let mut command = Command::new("xdg-open");
        command.arg(url);
        command
    };
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .is_ok()
}
