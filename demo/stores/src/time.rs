//! Calendar helpers on Unix timestamps (UTC, proleptic Gregorian). No clock access here.

/// Civil date `(year, month 1..=12, day 1..=31)` of a Unix timestamp, in UTC.
#[must_use]
pub fn civil_from_unix(unix_seconds: i64) -> (i64, u32, u32) {
    // Howard Hinnant's `civil_from_days`, valid far beyond any date this demo will see.
    let days = unix_seconds.div_euclid(86_400);
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

/// The UTC year of a Unix timestamp.
#[must_use]
pub fn year(unix_seconds: i64) -> i64 {
    civil_from_unix(unix_seconds).0
}

/// `2026-11-12T08:30:00Z`.
#[must_use]
pub fn iso8601(unix_seconds: i64) -> String {
    let (y, m, d) = civil_from_unix(unix_seconds);
    let secs = unix_seconds.rem_euclid(86_400);
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z", secs / 3600, secs % 3600 / 60, secs % 60)
}

/// `12 novembre 2026` (French long date, `1er` for the first of the month).
#[must_use]
pub fn french_date(unix_seconds: i64) -> String {
    const MONTHS: [&str; 12] = [
        "janvier",
        "février",
        "mars",
        "avril",
        "mai",
        "juin",
        "juillet",
        "août",
        "septembre",
        "octobre",
        "novembre",
        "décembre",
    ];
    let (y, m, d) = civil_from_unix(unix_seconds);
    let month = MONTHS.get(m.saturating_sub(1) as usize).copied().unwrap_or("?");
    if d == 1 { format!("1er {month} {y}") } else { format!("{d} {month} {y}") }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_dates() {
        assert_eq!(civil_from_unix(0), (1970, 1, 1));
        assert_eq!(civil_from_unix(86_399), (1970, 1, 1));
        assert_eq!(civil_from_unix(86_400), (1970, 1, 2));
        assert_eq!(civil_from_unix(951_782_400), (2000, 2, 29)); // leap day
        assert_eq!(civil_from_unix(1_794_441_600), (2026, 11, 12));
        assert_eq!(civil_from_unix(-1), (1969, 12, 31));
        assert_eq!(iso8601(1_794_441_600 + 8 * 3600 + 30 * 60 + 5), "2026-11-12T08:30:05Z");
        assert_eq!(iso8601(-1), "1969-12-31T23:59:59Z");
        assert_eq!(year(1_794_441_600), 2026);
        assert_eq!(french_date(1_794_441_600), "12 novembre 2026");
        assert_eq!(french_date(1_796_083_200), "1er décembre 2026");
    }

    #[test]
    fn extreme_values_do_not_panic() {
        for t in [i64::MIN / 2, i64::MAX / 2, i64::MIN + 1, i64::MAX] {
            let _ = iso8601(t);
            let _ = french_date(t);
        }
    }
}
