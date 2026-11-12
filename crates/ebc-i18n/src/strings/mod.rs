//! The message tables, one module per locale.
//!
//! Each module has a single exhaustive `match`, so adding a [`Msg`] variant does not
//! compile until every locale says something about it. A locale may return `""` to mean
//! "not translated yet": [`crate::t`] then falls back to English, then French.

use crate::Msg;
use ebc_model::Locale;

mod de;
mod en;
mod es;
mod fr;
mod it;
mod nl;
mod pt;

/// The text `locale` itself provides for `msg` (`""` if none).
pub(crate) fn text(locale: Locale, msg: Msg) -> &'static str {
    match locale {
        Locale::Fr => fr::text(msg),
        Locale::En => en::text(msg),
        Locale::De => de::text(msg),
        Locale::Es => es::text(msg),
        Locale::It => it::text(msg),
        Locale::Nl => nl::text(msg),
        Locale::Pt => pt::text(msg),
    }
}
