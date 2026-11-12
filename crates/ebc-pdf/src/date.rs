//! Conversion of Unix timestamps to UTC calendar dates, without touching the system clock.

/// A UTC calendar date and time of day.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DateTime {
    pub(crate) year: u16,
    pub(crate) month: u8,
    pub(crate) day: u8,
    pub(crate) hour: u8,
    pub(crate) minute: u8,
    pub(crate) second: u8,
}

/// Seconds between 0001-01-01T00:00:00Z and the Unix epoch.
const MIN_UNIX: i64 = -62_135_596_800;
/// 9999-12-31T23:59:59Z as a Unix timestamp.
const MAX_UNIX: i64 = 253_402_300_799;

impl DateTime {
    /// Converts seconds since the Unix epoch (UTC) to a calendar date, or `None` when the
    /// instant is outside the years 0001–9999 that PDF and XMP dates can express.
    pub(crate) fn from_unix(seconds: i64) -> Option<Self> {
        if !(MIN_UNIX..=MAX_UNIX).contains(&seconds) {
            return None;
        }
        let days = seconds.div_euclid(86_400);
        let secs_of_day = seconds.rem_euclid(86_400);

        // Howard Hinnant's `civil_from_days`, proleptic Gregorian calendar.
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let day_of_era = z.rem_euclid(146_097);
        let year_of_era =
            (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
        let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
        let mp = (5 * day_of_year + 2) / 153;
        let day = day_of_year - (153 * mp + 2) / 5 + 1;
        let month = if mp < 10 { mp + 3 } else { mp - 9 };
        let year = year_of_era + era * 400 + i64::from(month <= 2);

        Some(Self {
            year: u16::try_from(year).ok()?,
            month: u8::try_from(month).ok()?,
            day: u8::try_from(day).ok()?,
            hour: u8::try_from(secs_of_day / 3_600).ok()?,
            minute: u8::try_from(secs_of_day % 3_600 / 60).ok()?,
            second: u8::try_from(secs_of_day % 60).ok()?,
        })
    }

    /// The date as a `pdf-writer` date in UTC, written as `D:YYYYMMDDHHmmSSZ`.
    pub(crate) fn to_pdf(self) -> pdf_writer::Date {
        pdf_writer::Date::new(self.year)
            .month(self.month)
            .day(self.day)
            .hour(self.hour)
            .minute(self.minute)
            .second(self.second)
            .utc_offset_hour(0)
            .utc_offset_minute(0)
    }

    /// The date in the XMP (ISO 8601) form `YYYY-MM-DDThh:mm:ssZ`.
    pub(crate) fn to_xmp(self) -> String {
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
            self.year, self.month, self.day, self.hour, self.minute, self.second
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn xmp(seconds: i64) -> Option<String> {
        DateTime::from_unix(seconds).map(DateTime::to_xmp)
    }

    #[test]
    fn known_instants() {
        assert_eq!(xmp(0).as_deref(), Some("1970-01-01T00:00:00Z"));
        assert_eq!(xmp(951_782_400).as_deref(), Some("2000-02-29T00:00:00Z"));
        assert_eq!(xmp(1_700_000_000).as_deref(), Some("2023-11-14T22:13:20Z"));
        assert_eq!(xmp(1_794_355_199).as_deref(), Some("2026-11-10T23:59:59Z"));
        assert_eq!(xmp(-1).as_deref(), Some("1969-12-31T23:59:59Z"));
        // 1900 was not a leap year, 2100 will not be one either.
        assert_eq!(xmp(-2_203_891_200).as_deref(), Some("1900-03-01T00:00:00Z"));
        assert_eq!(xmp(4_107_542_400).as_deref(), Some("2100-03-01T00:00:00Z"));
    }

    #[test]
    fn range_limits() {
        assert_eq!(xmp(MIN_UNIX).as_deref(), Some("0001-01-01T00:00:00Z"));
        assert_eq!(xmp(MAX_UNIX).as_deref(), Some("9999-12-31T23:59:59Z"));
        assert_eq!(xmp(MIN_UNIX - 1), None);
        assert_eq!(xmp(MAX_UNIX + 1), None);
        assert_eq!(xmp(i64::MIN), None);
        assert_eq!(xmp(i64::MAX), None);
    }

    #[test]
    fn every_day_of_four_centuries_round_trips_in_order() {
        // Walk day by day and check that the calendar advances by exactly one day each step.
        let days_in = |y: u16, m: u8| -> u8 {
            match m {
                1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
                4 | 6 | 9 | 11 => 30,
                _ if (y.is_multiple_of(4) && !y.is_multiple_of(100)) || y.is_multiple_of(400) => 29,
                _ => 28,
            }
        };
        let start = -11_644_473_600; // 1601-01-01
        let mut prev = DateTime::from_unix(start).expect("in range");
        assert_eq!((prev.year, prev.month, prev.day), (1601, 1, 1));
        for i in 1..146_097i64 {
            let cur = DateTime::from_unix(start + i * 86_400).expect("in range");
            let expected = if prev.day < days_in(prev.year, prev.month) {
                (prev.year, prev.month, prev.day + 1)
            } else if prev.month < 12 {
                (prev.year, prev.month + 1, 1)
            } else {
                (prev.year + 1, 1, 1)
            };
            assert_eq!((cur.year, cur.month, cur.day), expected);
            prev = cur;
        }
        assert_eq!((prev.year, prev.month, prev.day), (2000, 12, 31));
    }
}
