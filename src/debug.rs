//! `--debug` / `VISOR_DEBUG`: one line per HTTP request on standard error,
//! for support. It never prints a token or a request body.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

static ENABLED: AtomicBool = AtomicBool::new(false);

pub fn enable() {
    ENABLED.store(true, Ordering::Relaxed);
}

#[must_use]
pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

/// Logs a finished request: `[debug] GET https://… → 200 in 84 ms (request id …)`.
pub fn request(method: &str, url: &str, outcome: &str, elapsed: Duration, request_id: Option<&str>) {
    if enabled() {
        let id = request_id
            .map(|id| format!(" (request id {id})"))
            .unwrap_or_default();
        eprintln!(
            "[debug] {method} {url} → {outcome} in {} ms{id}",
            elapsed.as_millis()
        );
    }
}

/// Logs any other event, such as a retry or a token refresh.
pub fn note(message: &str) {
    if enabled() {
        eprintln!("[debug] {message}");
    }
}
