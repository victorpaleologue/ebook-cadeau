//! Injected time and randomness: nothing in the request path reads the wall clock or the
//! OS random source directly, so tests control both.

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

/// A clock defined by a function (to share another component's clock).
pub struct FnClock<F>(pub F);

impl<F: Fn() -> i64 + Send + Sync + 'static> Clock for FnClock<F> {
    fn now_unix(&self) -> i64 {
        (self.0)()
    }
}

/// A source of cryptographically secure random bytes.
pub trait Entropy: Send + Sync + 'static {
    /// Fills `buf` with random bytes.
    ///
    /// # Errors
    /// When the generator fails; callers answer `500` rather than use weaker randomness.
    fn fill(&self, buf: &mut [u8]) -> Result<(), EntropyError>;
}

/// The random source failed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("the random source failed: {0}")]
pub struct EntropyError(pub String);

/// The operating system's CSPRNG.
#[derive(Debug, Clone, Copy, Default)]
pub struct OsEntropy;

impl Entropy for OsEntropy {
    fn fill(&self, buf: &mut [u8]) -> Result<(), EntropyError> {
        use rand::TryRng;
        rand::rngs::SysRng.try_fill_bytes(buf).map_err(|e| EntropyError(e.to_string()))
    }
}

/// `(year, month, day)` of a Unix time (UTC), proleptic Gregorian calendar.
#[must_use]
pub fn civil_date(unix: i64) -> (i64, u32, u32) {
    // Howard Hinnant's days-from-civil inverse.
    let days = unix.div_euclid(86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    (year, u32::try_from(month).unwrap_or(1), u32::try_from(day).unwrap_or(1))
}

/// Unix time of `year-month-day` 00:00 UTC; `None` for an impossible date.
#[must_use]
pub fn unix_of_date(year: i64, month: u32, day: u32) -> Option<i64> {
    if !(1..=12).contains(&month) || day == 0 || !(1900..=9999).contains(&year) {
        return None;
    }
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    let days_in_month = match month {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    if day > days_in_month {
        return None;
    }
    let y = if month <= 2 { year - 1 } else { year };
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let m = i64::from(month);
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + i64::from(day) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some((era * 146_097 + doe - 719_468) * 86_400)
}

/// `YYYY-MM-DD` of a Unix time (UTC).
#[must_use]
pub fn iso_date(unix: i64) -> String {
    let (y, m, d) = civil_date(unix);
    format!("{y:04}-{m:02}-{d:02}")
}

/// `YYYY-MM-DD HH:MM UTC` of a Unix time.
#[must_use]
pub fn iso_minute(unix: i64) -> String {
    let seconds = unix.rem_euclid(86_400);
    format!("{} {:02}:{:02} UTC", iso_date(unix), seconds / 3_600, seconds % 3_600 / 60)
}

/// Parses `YYYY-MM-DD` (as an HTML date input sends it) into 00:00 UTC of that day.
#[must_use]
pub fn parse_iso_date(text: &str) -> Option<i64> {
    let text = text.trim();
    let mut parts = text.splitn(3, '-');
    let (y, m, d) = (parts.next()?, parts.next()?, parts.next()?);
    if y.len() != 4 || m.len() != 2 || d.len() != 2 {
        return None;
    }
    if ![y, m, d].iter().all(|p| p.bytes().all(|b| b.is_ascii_digit())) {
        return None;
    }
    unix_of_date(y.parse().ok()?, m.parse().ok()?, d.parse().ok()?)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn dates_round_trip() {
        assert_eq!(civil_date(0), (1970, 1, 1));
        assert_eq!(iso_date(1_794_441_600), "2026-11-12");
        assert_eq!(iso_minute(1_794_441_600 + 3_600 * 13 + 60 * 7 + 5), "2026-11-12 13:07 UTC");
        assert_eq!(parse_iso_date("2026-11-12"), Some(1_794_441_600));
        for day in (0..30_000).step_by(7) {
            let unix = i64::from(day) * 86_400 + 1_000_000_000;
            let midnight = unix - unix.rem_euclid(86_400);
            assert_eq!(parse_iso_date(&iso_date(unix)), Some(midnight));
        }
        assert_eq!(parse_iso_date("2024-02-29").map(iso_date).as_deref(), Some("2024-02-29"));
        for bad in ["2025-02-29", "2026-13-01", "2026-00-10", "26-11-12", "2026-1-12", "", "x"] {
            assert_eq!(parse_iso_date(bad), None, "{bad}");
        }
    }

    #[test]
    fn manual_clock_moves_only_when_told() {
        let clock = ManualClock::new(10);
        clock.advance(5);
        assert_eq!(clock.now_unix(), 15);
        assert_eq!(FnClock(|| 42).now_unix(), 42);
    }

    #[test]
    fn os_entropy_differs() {
        let (mut a, mut b) = ([0u8; 16], [0u8; 16]);
        OsEntropy.fill(&mut a).unwrap();
        OsEntropy.fill(&mut b).unwrap();
        assert_ne!(a, b);
    }
}
