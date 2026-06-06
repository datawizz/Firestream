//! Bounded retry loop used to drive a probe until success-or-deadline.
//!
//! Lifted verbatim from `firestream/tests/e2e/harness.rs` (Phase 1 of the
//! k8s-e2e plan). Both harnesses share the same poll cadence so observed
//! timing characteristics stay consistent across backends.

use std::thread;
use std::time::{Duration, Instant};

use anyhow::Result;

/// Inter-attempt sleep. Public so callers (and tests) can derive sensible
/// deadlines from it; the loop itself doesn't honor any cancellation
/// signal beyond the wall-clock deadline.
pub const POLL_PROBE: Duration = Duration::from_millis(500);

/// Run `f` until it returns `Ok(())` or `*deadline` elapses. Sleeps
/// [`POLL_PROBE`] between attempts. On deadline returns the most recent
/// error; if no attempt ever ran before the deadline returns a synthetic
/// `anyhow` error.
pub fn retry_until_sync<F>(deadline: &Instant, mut f: F) -> Result<()>
where
    F: FnMut() -> Result<()>,
{
    let mut last_err: Option<anyhow::Error> = None;
    while Instant::now() < *deadline {
        match f() {
            Ok(()) => return Ok(()),
            Err(e) => last_err = Some(e),
        }
        thread::sleep(POLL_PROBE);
    }
    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("probe never ran before deadline")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[test]
    fn returns_ok_immediately() {
        let calls = AtomicU32::new(0);
        let deadline = Instant::now() + Duration::from_secs(5);
        retry_until_sync(&deadline, || {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
        .expect("immediate success path");
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn returns_last_err_on_deadline() {
        let deadline = Instant::now() + Duration::from_millis(50);
        let err = retry_until_sync(&deadline, || Err::<(), _>(anyhow::anyhow!("nope"))).unwrap_err();
        assert!(err.to_string().contains("nope"));
    }
}
