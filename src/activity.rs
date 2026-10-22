//! The loader: a spinner on standard error while a command waits for Visor.
//!
//! One spinner runs at a time, on its own thread, so it animates while a
//! request blocks. It appears only after a short delay, so a fast command
//! never flickers, and only on a terminal. Anything that prints stops it
//! first (`output::line`, `output::note`), so it never mixes with output.

use std::io::IsTerminal;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

const FRAMES: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
const DELAY: Duration = Duration::from_millis(150);
const FRAME: Duration = Duration::from_millis(80);

struct Running {
    message: Arc<Mutex<String>>,
    stop: Arc<AtomicBool>,
    thread: JoinHandle<()>,
}

static CURRENT: Mutex<Option<Running>> = Mutex::new(None);

/// Starts the loader with `message`, unless standard error is not a
/// terminal or `--debug` prints its own lines.
pub fn start(message: &str) {
    if !std::io::stderr().is_terminal() || crate::debug::enabled() {
        return;
    }
    stop();
    let message = Arc::new(Mutex::new(message.to_owned()));
    let stop_flag = Arc::new(AtomicBool::new(false));
    let thread = {
        let message = Arc::clone(&message);
        let stop_flag = Arc::clone(&stop_flag);
        thread::spawn(move || {
            thread::sleep(DELAY);
            let mut drawn = false;
            let mut frame = 0;
            while !stop_flag.load(Ordering::Relaxed) {
                let text = message.lock().map(|text| text.clone()).unwrap_or_default();
                eprint!("\r\x1b[K{} {text}", FRAMES[frame % FRAMES.len()]);
                drawn = true;
                frame += 1;
                thread::sleep(FRAME);
            }
            if drawn {
                eprint!("\r\x1b[K");
            }
        })
    };
    *CURRENT.lock().expect("loader lock") = Some(Running {
        message,
        stop: stop_flag,
        thread,
    });
}

/// Changes what the loader says, such as `1200 transactions so far…`.
pub fn set(text: &str) {
    if let Some(running) = CURRENT.lock().expect("loader lock").as_ref()
        && let Ok(mut message) = running.message.lock()
    {
        text.clone_into(&mut message);
    }
}

/// Stops the loader and clears its line. Safe to call when none runs.
pub fn stop() {
    let running = CURRENT.lock().expect("loader lock").take();
    if let Some(running) = running {
        running.stop.store(true, Ordering::Relaxed);
        let _ = running.thread.join();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn without_a_terminal_nothing_runs() {
        // `cargo test` captures standard error, so it is not a terminal.
        start("Loading…");
        set("still loading…");

        assert!(CURRENT.lock().unwrap().is_none());
        stop();
    }
}
