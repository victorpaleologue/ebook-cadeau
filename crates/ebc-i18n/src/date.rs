//! Long dates ("12 novembre 2026") in each locale.

use ebc_model::Locale;

/// Month names, January first, as used inside a date (lower case where the language
/// writes them so).
const fn months(locale: Locale) -> [&'static str; 12] {
    match locale {
        Locale::Fr => [
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
        ],
        Locale::En => [
            "January",
            "February",
            "March",
            "April",
            "May",
            "June",
            "July",
            "August",
            "September",
            "October",
            "November",
            "December",
        ],
        Locale::De => [
            "Januar",
            "Februar",
            "März",
            "April",
            "Mai",
            "Juni",
            "Juli",
            "August",
            "September",
            "Oktober",
            "November",
            "Dezember",
        ],
        Locale::Es => [
            "enero",
            "febrero",
            "marzo",
            "abril",
            "mayo",
            "junio",
            "julio",
            "agosto",
            "septiembre",
            "octubre",
            "noviembre",
            "diciembre",
        ],
        Locale::It => [
            "gennaio",
            "febbraio",
            "marzo",
            "aprile",
            "maggio",
            "giugno",
            "luglio",
            "agosto",
            "settembre",
            "ottobre",
            "novembre",
            "dicembre",
        ],
        Locale::Nl => [
            "januari",
            "februari",
            "maart",
            "april",
            "mei",
            "juni",
            "juli",
            "augustus",
            "september",
            "oktober",
            "november",
            "december",
        ],
        Locale::Pt => [
            "janeiro",
            "fevereiro",
            "março",
            "abril",
            "maio",
            "junho",
            "julho",
            "agosto",
            "setembro",
            "outubro",
            "novembro",
            "dezembro",
        ],
    }
}

/// Whether `year` is a leap year in the proleptic Gregorian calendar.
const fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Number of days in `month` (1..=12) of `year`; 0 for an invalid month.
const fn days_in_month(year: i64, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

/// Formats a calendar date in the long form of `locale`:
///
/// | locale | example |
/// |---|---|
/// | fr | `12 novembre 2026`, `1er novembre 2026` |
/// | en | `12 November 2026` |
/// | de | `12. November 2026` |
/// | es | `12 de noviembre de 2026` |
/// | it | `12 novembre 2026`, `1º novembre 2026` |
/// | nl | `12 november 2026` |
/// | pt | `12 de novembro de 2026` |
///
/// Returns `None` if the date does not exist (month 13, 30 February…).
///
/// ```
/// use ebc_i18n::{Locale, format_date};
/// assert_eq!(format_date(Locale::De, 2026, 11, 12).as_deref(), Some("12. November 2026"));
/// assert_eq!(format_date(Locale::Fr, 2027, 2, 29), None);
/// ```
#[must_use]
pub fn format_date(locale: Locale, year: i64, month: u32, day: u32) -> Option<String> {
    if day == 0 || day > days_in_month(year, month) {
        return None;
    }
    let name = months(locale).get(month.checked_sub(1)? as usize).copied()?;
    Some(match locale {
        Locale::Fr if day == 1 => format!("1er {name} {year}"),
        Locale::It if day == 1 => format!("1º {name} {year}"),
        Locale::Fr | Locale::En | Locale::It | Locale::Nl => format!("{day} {name} {year}"),
        Locale::De => format!("{day}. {name} {year}"),
        Locale::Es | Locale::Pt => format!("{day} de {name} de {year}"),
    })
}

/// Converts seconds since the Unix epoch to a `(year, month, day)` in UTC (proleptic
/// Gregorian calendar). Total: every `i64` maps to a date.
///
/// ```
/// assert_eq!(ebc_i18n::date_from_unix(0), (1970, 1, 1));
/// assert_eq!(ebc_i18n::date_from_unix(-1), (1969, 12, 31));
/// ```
#[must_use]
pub fn date_from_unix(unix_seconds: i64) -> (i64, u32, u32) {
    // Howard Hinnant's `civil_from_days`, in 64-bit arithmetic.
    let days = unix_seconds.div_euclid(86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 { month_index + 3 } else { month_index - 9 };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (year, month as u32, day as u32)
}

/// Formats the UTC date of a Unix timestamp in the long form of `locale` (see
/// [`format_date`]).
///
/// ```
/// use ebc_i18n::{Locale, format_unix_date};
/// assert_eq!(format_unix_date(Locale::Fr, 1_794_441_600), "12 novembre 2026");
/// ```
#[must_use]
pub fn format_unix_date(locale: Locale, unix_seconds: i64) -> String {
    let (year, month, day) = date_from_unix(unix_seconds);
    // `date_from_unix` only produces existing dates; the fallback is unreachable but
    // keeps this function total without a panic path.
    format_date(locale, year, month, day).unwrap_or_else(|| format!("{year}-{month:02}-{day:02}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn long_dates_per_locale() {
        let expected = [
            (Locale::Fr, "12 novembre 2026"),
            (Locale::En, "12 November 2026"),
            (Locale::De, "12. November 2026"),
            (Locale::Es, "12 de noviembre de 2026"),
            (Locale::It, "12 novembre 2026"),
            (Locale::Nl, "12 november 2026"),
            (Locale::Pt, "12 de novembro de 2026"),
        ];
        for (locale, text) in expected {
            assert_eq!(format_date(locale, 2026, 11, 12).as_deref(), Some(text));
            assert_eq!(format_unix_date(locale, 1_794_441_600), text);
        }
        assert_eq!(expected.len(), Locale::ALL.len());
    }

    #[test]
    fn first_of_the_month() {
        assert_eq!(format_date(Locale::Fr, 2027, 1, 1).as_deref(), Some("1er janvier 2027"));
        assert_eq!(format_date(Locale::It, 2027, 1, 1).as_deref(), Some("1º gennaio 2027"));
        assert_eq!(format_date(Locale::En, 2027, 1, 1).as_deref(), Some("1 January 2027"));
        assert_eq!(format_date(Locale::De, 2027, 3, 1).as_deref(), Some("1. März 2027"));
        assert_eq!(format_date(Locale::Es, 2027, 1, 1).as_deref(), Some("1 de enero de 2027"));
        assert_eq!(format_date(Locale::Pt, 2027, 3, 1).as_deref(), Some("1 de março de 2027"));
        assert_eq!(format_date(Locale::Nl, 2027, 8, 1).as_deref(), Some("1 augustus 2027"));
        assert_eq!(format_date(Locale::Fr, 2027, 8, 21).as_deref(), Some("21 août 2027"));
    }

    #[test]
    fn every_month_has_a_name_in_every_locale() {
        for locale in Locale::ALL {
            let names = months(locale);
            for (index, name) in names.iter().enumerate() {
                assert!(name.chars().count() >= 3, "{locale} month {}", index + 1);
                let formatted = format_date(locale, 2026, index as u32 + 1, 15).expect("valid");
                assert!(formatted.contains(name) && formatted.contains("2026"));
                assert!(formatted.contains("15"));
            }
            let mut unique = names.to_vec();
            unique.sort_unstable();
            unique.dedup();
            assert_eq!(unique.len(), 12, "{locale}");
        }
    }

    #[test]
    fn impossible_dates_are_rejected() {
        for (year, month, day) in [
            (2026, 0, 1),
            (2026, 13, 1),
            (2026, 1, 0),
            (2026, 1, 32),
            (2026, 4, 31),
            (2026, 2, 29),
            (2100, 2, 29),
            (2026, u32::MAX, 1),
            (2026, 1, u32::MAX),
        ] {
            for locale in Locale::ALL {
                assert_eq!(format_date(locale, year, month, day), None, "{year}-{month}-{day}");
            }
        }
        assert!(format_date(Locale::En, 2028, 2, 29).is_some());
        assert!(format_date(Locale::En, 2000, 2, 29).is_some());
        assert!(format_date(Locale::En, -4, 2, 29).is_some());
    }

    #[test]
    fn unix_conversion_matches_known_dates() {
        for (unix, date) in [
            (0, (1970, 1, 1)),
            (86_399, (1970, 1, 1)),
            (86_400, (1970, 1, 2)),
            (-1, (1969, 12, 31)),
            (-86_400, (1969, 12, 31)),
            (-86_401, (1969, 12, 30)),
            (951_782_400, (2000, 2, 29)),
            (951_868_800, (2000, 3, 1)),
            (1_709_164_800, (2024, 2, 29)),
            (1_794_441_600, (2026, 11, 12)),
            (1_798_761_599, (2026, 12, 31)),
            (1_798_761_600, (2027, 1, 1)),
            (4_102_444_800, (2100, 1, 1)),
            (-2_208_988_800, (1900, 1, 1)),
            (-62_135_596_800, (1, 1, 1)),
        ] {
            assert_eq!(date_from_unix(unix), date, "unix {unix}");
        }
    }

    #[test]
    fn unix_conversion_is_total_and_consistent() {
        // Extremes do not overflow or panic, and always give an existing date.
        for unix in [i64::MIN, i64::MIN + 1, -1, 0, 1, i64::MAX - 1, i64::MAX] {
            let (year, month, day) = date_from_unix(unix);
            assert!((1..=12).contains(&month), "unix {unix}");
            assert!(day >= 1 && day <= days_in_month(year, month), "unix {unix}");
            for locale in Locale::ALL {
                assert!(format_unix_date(locale, unix).contains(&year.to_string()));
            }
        }
        // Walking day by day never skips or repeats a date.
        let mut previous = date_from_unix(-800 * 86_400);
        for day_number in -799..800i64 {
            let current = date_from_unix(day_number * 86_400);
            let (year, month, day) = previous;
            let expected = if day < days_in_month(year, month) {
                (year, month, day + 1)
            } else if month < 12 {
                (year, month + 1, 1)
            } else {
                (year + 1, 1, 1)
            };
            assert_eq!(current, expected, "day {day_number}");
            previous = current;
        }
    }
}
