//! Injected time and randomness, so that nothing in the request path reads the wall clock
//! or the OS random source directly and tests can control both.

use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// A source of the current time.
pub trait Clock: Send + Sync + 'static {
    /// Seconds since the Unix epoch (UTC).
    fn now_unix(&self) -> i64;
}

/// The system clock.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_unix(&self) -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
    }
}

/// A clock that only moves when told to; for tests and demos.
#[derive(Debug)]
pub struct ManualClock(AtomicI64);

impl ManualClock {
    /// A clock stopped at `now_unix`.
    #[must_use]
    pub fn new(now_unix: i64) -> Self {
        Self(AtomicI64::new(now_unix))
    }

    /// Jumps to `now_unix`.
    pub fn set(&self, now_unix: i64) {
        self.0.store(now_unix, Ordering::SeqCst);
    }

    /// Moves forward (or backward, for negative values) by `seconds`.
    pub fn advance(&self, seconds: i64) {
        self.0.fetch_add(seconds, Ordering::SeqCst);
    }
}

impl Clock for ManualClock {
    fn now_unix(&self) -> i64 {
        self.0.load(Ordering::SeqCst)
    }
}

/// A source of cryptographically secure random bytes.
pub trait Entropy: Send + Sync + 'static {
    /// Fills `buf` with random bytes.
    ///
    /// # Errors
    /// When the underlying generator fails; callers answer `500` rather than fall back to
    /// weaker randomness.
    fn fill(&self, buf: &mut [u8]) -> Result<(), EntropyError>;
}

/// The random source failed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("the random source failed: {0}")]
pub struct EntropyError(pub String);

/// The operating system's CSPRNG (through `rand::rngs::SysRng`).
#[derive(Debug, Clone, Copy, Default)]
pub struct OsEntropy;

impl Entropy for OsEntropy {
    fn fill(&self, buf: &mut [u8]) -> Result<(), EntropyError> {
        use rand::TryRng;
        rand::rngs::SysRng.try_fill_bytes(buf).map_err(|e| EntropyError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manual_clock_moves_only_when_told() {
        let clock = ManualClock::new(1_000);
        assert_eq!(clock.now_unix(), 1_000);
        clock.advance(15);
        assert_eq!(clock.now_unix(), 1_015);
        clock.advance(-20);
        assert_eq!(clock.now_unix(), 995);
        clock.set(42);
        assert_eq!(clock.now_unix(), 42);
    }

    #[test]
    fn system_clock_is_after_2025() {
        assert!(SystemClock.now_unix() > 1_735_689_600);
    }

    #[test]
    fn os_entropy_fills_and_differs() {
        let (mut a, mut b) = ([0u8; 32], [0u8; 32]);
        OsEntropy.fill(&mut a).expect("os rng");
        OsEntropy.fill(&mut b).expect("os rng");
        assert_ne!(a, b);
        assert_ne!(a, [0u8; 32]);
    }
}
