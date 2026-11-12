//! Printed strings that `ebc-i18n` does not have yet (a gap of that crate, reported):
//! the instructions in the margin of a home sheet and the cover credit line.
//!
//! Drafted to the wording rules of `ebc-i18n` (infinitive of instructions in French,
//! German, Spanish, Italian and Portuguese; imperative in English and Dutch). Like the
//! strings of `ebc-i18n`, none was reviewed by a native speaker.

use ebc_layout::DuplexFlip;
use ebc_model::Locale;

/// "Print at 100 %, on both sides, flip on the short/long edge."
pub(crate) fn duplex_hint(locale: Locale, flip: DuplexFlip) -> &'static str {
    let short = flip == DuplexFlip::ShortEdge;
    match (locale, short) {
        (Locale::Fr, true) => {
            "Imprimer à 100\u{a0}% (taille réelle), recto verso, retourner sur le bord court."
        }
        (Locale::Fr, false) => {
            "Imprimer à 100\u{a0}% (taille réelle), recto verso, retourner sur le bord long."
        }
        (Locale::En, true) => "Print at 100% (actual size), on both sides, flip on the short edge.",
        (Locale::En, false) => "Print at 100% (actual size), on both sides, flip on the long edge.",
        (Locale::De, true) => {
            "In 100\u{a0}% (Originalgröße) beidseitig drucken, an der kurzen Kante spiegeln."
        }
        (Locale::De, false) => {
            "In 100\u{a0}% (Originalgröße) beidseitig drucken, an der langen Kante spiegeln."
        }
        (Locale::Es, true) => {
            "Imprimir al 100\u{a0}% (tamaño real), a doble cara, voltear por el borde corto."
        }
        (Locale::Es, false) => {
            "Imprimir al 100\u{a0}% (tamaño real), a doble cara, voltear por el borde largo."
        }
        (Locale::It, true) => {
            "Stampare al 100% (dimensioni reali), fronte-retro, capovolgere sul lato corto."
        }
        (Locale::It, false) => {
            "Stampare al 100% (dimensioni reali), fronte-retro, capovolgere sul lato lungo."
        }
        (Locale::Nl, true) => {
            "Print op 100% (ware grootte), dubbelzijdig, omslaan langs de korte zijde."
        }
        (Locale::Nl, false) => {
            "Print op 100% (ware grootte), dubbelzijdig, omslaan langs de lange zijde."
        }
        (Locale::Pt, true) => {
            "Imprimir a 100\u{a0}% (tamanho real), frente e verso, virar pela margem curta."
        }
        (Locale::Pt, false) => {
            "Imprimir a 100\u{a0}% (tamanho real), frente e verso, virar pela margem longa."
        }
    }
}

/// "Cut on the solid marks, fold on the dashed ones."
pub(crate) fn cut_fold_hint(locale: Locale) -> &'static str {
    match locale {
        Locale::Fr => "Couper sur les traits pleins, plier sur les pointillés.",
        Locale::En => "Cut along the solid marks, fold along the dashed ones.",
        Locale::De => "An den durchgezogenen Marken schneiden, an den gestrichelten falzen.",
        Locale::Es => "Cortar por las marcas continuas, doblar por las discontinuas.",
        Locale::It => "Tagliare lungo i segni continui, piegare lungo quelli tratteggiati.",
        Locale::Nl => "Snijd langs de doorgetrokken streepjes, vouw langs de gestippelde.",
        Locale::Pt => "Cortar pelas marcas contínuas, dobrar pelas tracejadas.",
    }
}

/// The word for the cover in a credit line, with its colon: "Couverture :".
fn cover_label(locale: Locale) -> &'static str {
    match locale {
        Locale::Fr => "Couverture\u{a0}:",
        Locale::En => "Cover:",
        Locale::De => "Umschlag:",
        Locale::Es => "Cubierta:",
        Locale::It => "Copertina:",
        Locale::Nl => "Omslag:",
        Locale::Pt => "Capa:",
    }
}

/// "Couverture : {credit}" — `credit` is the caller's credit text, used as given.
pub(crate) fn cover_credit(locale: Locale, credit: &str) -> String {
    format!("{} {credit}", cover_label(locale))
}

/// "Couverture : © {publisher}".
pub(crate) fn cover_credit_publisher(locale: Locale, publisher: &str) -> String {
    format!("{} ©\u{a0}{publisher}", cover_label(locale))
}

/// "Couverture : domaine public".
pub(crate) fn cover_credit_public_domain(locale: Locale) -> String {
    let words = match locale {
        Locale::Fr => "domaine public",
        Locale::En => "public domain",
        Locale::De => "gemeinfrei",
        Locale::Es => "dominio público",
        Locale::It => "pubblico dominio",
        Locale::Nl => "publiek domein",
        Locale::Pt => "domínio público",
    };
    format!("{} {words}", cover_label(locale))
}

/// "Couverture : tous droits réservés" — when neither a credit nor a publisher is known.
pub(crate) fn cover_credit_unknown(locale: Locale) -> String {
    let words = match locale {
        Locale::Fr => "tous droits réservés",
        Locale::En => "all rights reserved",
        Locale::De => "alle Rechte vorbehalten",
        Locale::Es => "todos los derechos reservados",
        Locale::It => "tutti i diritti riservati",
        Locale::Nl => "alle rechten voorbehouden",
        Locale::Pt => "todos os direitos reservados",
    };
    format!("{} {words}", cover_label(locale))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_locale_has_every_string() {
        for locale in Locale::ALL {
            for flip in [DuplexFlip::ShortEdge, DuplexFlip::LongEdge] {
                let hint = duplex_hint(locale, flip);
                assert!(hint.contains("100"), "{locale}: {hint}");
            }
            assert_ne!(
                duplex_hint(locale, DuplexFlip::ShortEdge),
                duplex_hint(locale, DuplexFlip::LongEdge)
            );
            assert!(!cut_fold_hint(locale).is_empty());
            assert!(cover_credit_publisher(locale, "Éditions du Lac").ends_with("Éditions du Lac"));
            assert!(cover_credit_publisher(locale, "X").contains('©'));
            assert!(!cover_credit_public_domain(locale).contains('©'));
            assert!(!cover_credit_unknown(locale).contains('©'));
            assert!(cover_credit(locale, "© Jeanne Roy").ends_with("© Jeanne Roy"));
        }
    }
}
