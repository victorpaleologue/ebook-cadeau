//! Checks over the whole message catalogue: completeness, placeholders, terminology,
//! typography. These are the guard rails for anyone editing a translation.

use std::collections::HashSet;

use ebc_i18n::{
    Locale, Msg, catalog, format_template, native_name, placeholders_in, t, t_with, tf, tf_with,
    translation,
};

/// Every (locale, message, text) triple.
fn all_texts() -> impl Iterator<Item = (Locale, Msg, &'static str)> {
    Locale::ALL
        .into_iter()
        .flat_map(|locale| Msg::ALL.iter().map(move |msg| (locale, *msg, t(locale, *msg))))
}

/// The text with its `{placeholders}` removed, for typography checks.
fn without_placeholders(text: &str) -> String {
    let blanks: Vec<(&str, &str)> = placeholders_in(text).into_iter().map(|p| (p, "")).collect();
    // `{de:name}` renders as "de " + value: drop that too.
    format_template(text, &blanks).replace("de ", "")
}

#[test]
fn every_message_is_translated_in_every_locale() {
    assert!(Msg::ALL.len() >= 17, "the contract lists at least 17 messages");
    let unique: HashSet<Msg> = Msg::ALL.iter().copied().collect();
    assert_eq!(unique.len(), Msg::ALL.len(), "Msg::ALL lists a message twice");
    for (locale, msg, text) in all_texts() {
        assert!(!text.trim().is_empty(), "{locale}/{msg:?} is empty");
        // The locale's own table has it: the English/French fallback is never needed.
        assert_eq!(translation(locale, msg), Some(text), "{locale}/{msg:?} falls back");
    }
}

#[test]
fn placeholders_match_the_declaration_in_every_translation() {
    for (locale, msg, text) in all_texts() {
        let found: HashSet<&str> = placeholders_in(text).into_iter().collect();
        let declared: HashSet<&str> = msg.placeholders().iter().copied().collect();
        assert_eq!(found, declared, "{locale}/{msg:?}: {text:?}");
        // Braces only ever delimit a placeholder.
        let braces = text.matches(['{', '}']).count();
        let expected: usize =
            msg.placeholders().iter().map(|p| 2 * text.matches(&format!("{p}}}")).count()).sum();
        assert_eq!(braces, expected, "{locale}/{msg:?}: stray brace in {text:?}");
        for name in msg.placeholders() {
            assert!(
                name.bytes().all(|b| b.is_ascii_lowercase() || b == b'_') && !name.is_empty(),
                "placeholder name {name:?}"
            );
        }
    }
}

#[test]
fn tf_fills_every_placeholder() {
    for (locale, msg, text) in all_texts() {
        let values: Vec<String> = msg.placeholders().iter().map(|p| format!("<{p}>")).collect();
        let args: Vec<(&str, &str)> =
            msg.placeholders().iter().copied().zip(values.iter().map(String::as_str)).collect();
        let filled = tf(locale, msg, &args);
        assert!(!filled.contains(['{', '}']), "{locale}/{msg:?}: {filled:?}");
        for value in &values {
            assert!(
                filled.contains(value.as_str()),
                "{locale}/{msg:?}: {value} missing in {filled:?}"
            );
        }
        if msg.placeholders().is_empty() {
            assert_eq!(filled, text);
            // Arguments nobody asked for change nothing.
            assert_eq!(tf(locale, msg, &[("name", "x"), ("url", "y")]), text);
        } else {
            // Without arguments the template is returned untouched.
            assert_eq!(tf(locale, msg, &[]), text);
        }
    }
}

#[test]
fn hostile_values_are_inserted_verbatim_and_never_expanded() {
    for locale in Locale::ALL {
        let from = tf(locale, Msg::CardFrom, &[("name", "{url}<script>alert(1)</script>")]);
        assert!(from.ends_with("{url}<script>alert(1)</script>"), "{locale}: {from:?}");
        let hint =
            tf(locale, Msg::CardNoPhone, &[("url", "{name}"), ("name", "https://evil.example")]);
        assert!(hint.contains("{name}") && !hint.contains("evil.example"), "{locale}: {hint:?}");
        let huge = "x".repeat(100_000);
        assert!(tf(locale, Msg::CardTo, &[("name", &huge)]).len() > 100_000);
        assert_eq!(
            tf(locale, Msg::CardTo, &[("name", "")]),
            t(locale, Msg::CardTo).replace("{name}", "")
        );
    }
}

#[test]
fn product_names_follow_the_brief() {
    let expected = [
        (Locale::Fr, "eBook cadeau"),
        (Locale::En, "eBook gift"),
        (Locale::De, "E-Book-Geschenk"),
        (Locale::Es, "eBook regalo"),
        (Locale::It, "eBook regalo"),
        (Locale::Nl, "e-book cadeau"),
        (Locale::Pt, "eBook presente"),
    ];
    assert_eq!(expected.len(), Locale::ALL.len());
    for (locale, name) in expected {
        assert_eq!(t(locale, Msg::ProductName), name);
        // The page title of the creator names the product.
        assert!(
            t(locale, Msg::CreateTitle).to_lowercase().contains(&name.to_lowercase()),
            "{locale}: {:?}",
            t(locale, Msg::CreateTitle)
        );
    }
}

#[test]
fn ebook_and_ereader_are_spelt_one_way_per_locale() {
    /// Every case-insensitive occurrence of `needle` (ASCII) in `text`, as written.
    fn occurrences<'a>(text: &'a str, needle: &str) -> Vec<&'a str> {
        let lower = text.to_ascii_lowercase();
        lower.match_indices(needle).filter_map(|(at, m)| text.get(at..at + m.len())).collect()
    }
    for locale in Locale::ALL {
        let ebook = t(locale, Msg::EbookWord);
        let ereader = t(locale, Msg::EreaderWord);
        assert!(t(locale, Msg::ProductName).contains(ebook), "{locale}: product name vs {ebook:?}");
        for msg in Msg::ALL {
            // The brand "eBook Cadeau" is a proper noun and keeps its spelling.
            let text = t(locale, *msg).replace("eBook Cadeau", "");
            for found in occurrences(&text, "ebook").into_iter().chain(occurrences(&text, "e-book"))
            {
                assert_eq!(found, ebook, "{locale}/{msg:?}: {text:?}");
            }
            for found in
                occurrences(&text, "ereader").into_iter().chain(occurrences(&text, "e-reader"))
            {
                assert_eq!(found, ereader, "{locale}/{msg:?}: {text:?}");
            }
        }
    }
    // French calls the device a "liseuse" everywhere.
    assert_eq!(t(Locale::Fr, Msg::EreaderWord), "liseuse");
    assert!(t(Locale::Fr, Msg::CardStep2).contains("liseuse"));
}

/// Words that name a stored-value instrument (a balance, a voucher) in each language.
/// The gift is one prepaid book; these words would promise something else, and some of
/// them carry statutory validity rules. German entries match inside compounds.
fn stored_value_terms(locale: Locale) -> &'static [&'static str] {
    match locale {
        Locale::Fr => &[
            "carte cadeau",
            "carte-cadeau",
            "cartes cadeaux",
            "chèque cadeau",
            "chèque-cadeau",
            "bon cadeau",
            "bon d’achat",
            "bon d'achat",
            "coffret cadeau",
            "chèque lire",
            "chèque culture",
        ],
        Locale::En => &["gift card", "gift voucher", "voucher", "token", "egift", "coupon"],
        Locale::De => &["gutschein", "geschenkkarte"],
        Locale::Es => {
            &["tarjeta regalo", "tarjeta de regalo", "cheque regalo", "cheque de regalo", "bono"]
        }
        Locale::It => &["buono regalo", "carta regalo", "gift card", "buono", "buoni"],
        Locale::Nl => &["cadeaubon", "cadeaukaart", "boekenbon", "tegoedbon"],
        Locale::Pt => &[
            "cartão oferta",
            "cartão-oferta",
            "cartão presente",
            "cheque-oferta",
            "cheque oferta",
            "vale",
            "vale-presente",
        ],
    }
}

/// Whether `term` occurs in `text` as a whole word or phrase (both lower case).
fn contains_term(text: &str, term: &str, inside_words: bool) -> bool {
    text.match_indices(term).any(|(at, m)| {
        let before = text.get(..at).and_then(|s| s.chars().next_back());
        let after = text.get(at + m.len()..).and_then(|s| s.chars().next());
        inside_words
            || (!before.is_some_and(char::is_alphanumeric)
                && !after.is_some_and(char::is_alphanumeric))
    })
}

#[test]
fn no_message_uses_a_stored_value_word() {
    for (locale, msg, text) in all_texts() {
        let lower = text.to_lowercase();
        for term in stored_value_terms(locale) {
            assert!(
                !contains_term(&lower, term, locale == Locale::De),
                "{locale}/{msg:?} uses the stored-value word {term:?}: {text:?}"
            );
        }
        // Whatever the language, the English terms must not leak in either.
        for term in ["gift card", "voucher"] {
            assert!(!contains_term(&lower, term, false), "{locale}/{msg:?}: {text:?}");
        }
    }
    // The checker itself: whole words only, except in German compounds.
    assert!(contains_term("une carte cadeau.", "carte cadeau", false));
    assert!(!contains_term("buona lettura", "buono", false));
    assert!(!contains_term("equivalente", "vale", false));
    assert!(contains_term("um vale de 10 €", "vale", false));
    assert!(contains_term("ein geschenkgutscheincode", "gutschein", true));
    assert!(!contains_term("ein geschenkgutscheincode", "gutschein", false));
}

#[test]
fn the_typed_fallback_is_called_a_code() {
    let code_word = |locale| match locale {
        Locale::Fr | Locale::En | Locale::De | Locale::Nl => "code",
        Locale::Es | Locale::Pt => "código",
        Locale::It => "codice",
    };
    for locale in Locale::ALL {
        let word = code_word(locale);
        assert_eq!(t(locale, Msg::CodeLabel).to_lowercase(), word);
        for msg in [
            Msg::CardNoPhone,
            Msg::RedeemIntro,
            Msg::ErrorCodeTypo,
            Msg::ErrorCodeUnknown,
            Msg::EreaderBrowserHint,
        ] {
            assert!(t(locale, msg).to_lowercase().contains(word), "{locale}/{msg:?}");
        }
    }
}

#[test]
fn typography_is_clean() {
    for (locale, msg, text) in all_texts() {
        let context = format!("{locale}/{msg:?}: {text:?}");
        assert_eq!(text.trim(), text, "{context}: surrounding whitespace");
        assert!(!text.contains("  "), "{context}: double space");
        assert!(!text.contains('\''), "{context}: use the typographic apostrophe ’");
        assert!(!text.contains('"'), "{context}: straight double quote");
        assert!(!text.contains("..."), "{context}: use the ellipsis character …");
        assert!(!text.chars().any(char::is_control), "{context}: control character");
        // Only characters any text font for Western European languages has.
        for c in text.chars() {
            let ok = matches!(c, ' '..='~' | '\u{a0}'..='\u{17f}' | '’' | '–' | '—' | '…');
            assert!(ok, "{context}: unexpected character {c:?} (U+{:04X})", c as u32);
        }
        let plain = without_placeholders(text);
        let chars: Vec<char> = plain.chars().collect();
        for (i, c) in chars.iter().enumerate() {
            if !matches!(c, '?' | '!' | ':' | ';') {
                continue;
            }
            let previous = i.checked_sub(1).and_then(|p| chars.get(p)).copied();
            if locale == Locale::Fr {
                assert_eq!(
                    previous,
                    Some('\u{a0}'),
                    "{context}: French needs a no-break space before {c:?}"
                );
            } else {
                assert!(
                    !previous.is_some_and(char::is_whitespace),
                    "{context}: no space before {c:?} outside French"
                );
            }
        }
        if locale != Locale::Fr {
            assert!(!text.contains('\u{a0}'), "{context}: unexpected no-break space");
        }
        if locale == Locale::Es {
            assert_eq!(text.matches('¿').count(), text.matches('?').count(), "{context}");
            assert_eq!(text.matches('¡').count(), text.matches('!').count(), "{context}");
        }
        if locale == Locale::De {
            assert!(!text.contains('ß'), "{context}: keep German strings free of ß (de-CH)");
        }
    }
}

#[test]
fn card_steps_are_short_single_instructions() {
    for locale in Locale::ALL {
        for msg in [Msg::CardStep1, Msg::CardStep2, Msg::CardStep3] {
            let text = t(locale, msg);
            let length = text.chars().count();
            assert!(
                (15..=60).contains(&length),
                "{locale}/{msg:?} has {length} characters: {text:?}"
            );
            assert!(text.ends_with(['.', '!']), "{locale}/{msg:?}: {text:?}");
            assert!(msg.placeholders().is_empty());
            assert!(
                text.chars().next().is_some_and(|c| c.is_uppercase() || c == '¡'),
                "{locale}/{msg:?}"
            );
        }
        for msg in [Msg::CardStep2Reseller, Msg::CardStep3Reseller] {
            let text = t(locale, msg);
            let length = text.chars().count();
            assert!((15..=90).contains(&length), "{locale}/{msg:?} has {length} characters");
            assert!(text.ends_with('.'), "{locale}/{msg:?}: {text:?}");
        }
        // Step 1 is about the QR code; the fallback line carries the address to type.
        assert!(t(locale, Msg::CardStep1).contains("QR"), "{locale}");
        assert!(t(locale, Msg::CardNoPhone).contains("{url}"), "{locale}");
        // The three steps are three different sentences.
        let steps: HashSet<&str> = [Msg::CardStep1, Msg::CardStep2, Msg::CardStep3]
            .into_iter()
            .map(|m| t(locale, m))
            .collect();
        assert_eq!(steps.len(), 3, "{locale}");
    }
}

#[test]
fn french_addresses_the_reader_with_vous() {
    // Imperatives in -ez, possessives votre/vos; never tu/ton/ta/tes/toi.
    for msg in [
        Msg::CardStep1,
        Msg::CardStep2,
        Msg::CardStep3,
        Msg::CardStep2Reseller,
        Msg::CardStep3Reseller,
    ] {
        let text = t(Locale::Fr, msg);
        let first_word = text.split_whitespace().next().unwrap_or_default();
        assert!(first_word.trim_end_matches("-vous").ends_with("ez"), "{msg:?}: {text:?}");
    }
    for msg in Msg::ALL {
        let lower = t(Locale::Fr, *msg).to_lowercase();
        for informal in ["tu", "ton", "ta", "tes", "toi", "t’"] {
            assert!(
                !contains_term(&lower, informal, false),
                "{msg:?} uses {informal:?}: {lower:?}"
            );
        }
    }
    assert!(t(Locale::Fr, Msg::CardStep1).contains("votre"));
}

#[test]
fn sentences_are_actually_translated() {
    // A long English sentence appearing verbatim in another locale is a copy-paste slip.
    for msg in Msg::ALL {
        let english = t(Locale::En, *msg);
        if english.split_whitespace().count() < 4 {
            continue;
        }
        for locale in Locale::ALL {
            if locale != Locale::En {
                assert_ne!(t(locale, *msg), english, "{locale}/{msg:?} is still English");
            }
        }
    }
    // And no two locales share a long sentence, except Spanish/Italian/Portuguese
    // cognates, which never coincide over a whole sentence in practice.
    for msg in Msg::ALL {
        let mut seen: HashSet<&str> = HashSet::new();
        for locale in Locale::ALL {
            let text = t(locale, *msg);
            if text.split_whitespace().count() >= 5 {
                assert!(seen.insert(text), "{locale}/{msg:?} duplicates another locale: {text:?}");
            }
        }
    }
}

#[test]
fn french_card_from_elides_before_a_vowel() {
    assert_eq!(
        tf(Locale::Fr, Msg::CardFrom, &[("name", "Mamie Jeanne")]),
        "De la part de Mamie Jeanne"
    );
    assert_eq!(tf(Locale::Fr, Msg::CardFrom, &[("name", "Élise")]), "De la part d’Élise");
    assert_eq!(tf(Locale::Fr, Msg::CardFrom, &[("name", "Hugo")]), "De la part d’Hugo");
    assert_eq!(tf(Locale::Fr, Msg::CardFrom, &[("name", "Yann")]), "De la part de Yann");
    assert_eq!(tf(Locale::Fr, Msg::CardTo, &[("name", "Élise")]), "Pour Élise");
    assert_eq!(tf(Locale::En, Msg::CardFrom, &[("name", "Élise")]), "From Élise");
    assert_eq!(tf(Locale::De, Msg::CardFrom, &[("name", "Anna")]), "Von Anna");
    assert_eq!(tf(Locale::Es, Msg::CardFrom, &[("name", "Ana")]), "De parte de Ana");
    assert_eq!(tf(Locale::It, Msg::CardFrom, &[("name", "Anna")]), "Da parte di Anna");
    assert_eq!(tf(Locale::Nl, Msg::CardFrom, &[("name", "Anna")]), "Van Anna");
    assert_eq!(tf(Locale::Pt, Msg::CardFrom, &[("name", "Ana")]), "De Ana");
}

#[test]
fn dates_fit_the_expiry_sentence() {
    use ebc_i18n::format_date;
    let expected = [
        (Locale::Fr, "Disponible jusqu’au 1er mars 2027."),
        (Locale::En, "Available until 1 March 2027."),
        (Locale::De, "Verfügbar bis zum 1. März 2027."),
        (Locale::Es, "Disponible hasta el 1 de marzo de 2027."),
        (Locale::It, "Disponibile fino al giorno 1º marzo 2027."),
        (Locale::Nl, "Beschikbaar tot en met 1 maart 2027."),
        (Locale::Pt, "Disponível até ao dia 1 de março de 2027."),
    ];
    for (locale, sentence) in expected {
        let date = format_date(locale, 2027, 3, 1).expect("valid date");
        assert_eq!(tf(locale, Msg::CardValidUntil, &[("date", &date)]), sentence);
    }
}

#[test]
fn overrides_replace_single_messages() {
    let overrides = [
        (Msg::EbookWord, "ebook"),
        (Msg::CheckoutOption, "C’est pour offrir"),
        (Msg::CardTo, "À {name}"),
        (Msg::CodeLabel, "   "),
    ];
    assert_eq!(t_with(Locale::Fr, Msg::EbookWord, &overrides), "ebook");
    assert_eq!(t_with(Locale::Fr, Msg::CheckoutOption, &overrides), "C’est pour offrir");
    assert_eq!(tf_with(Locale::Fr, Msg::CardTo, &[("name", "Léo")], &overrides), "À Léo");
    // Blank overrides are ignored; untouched messages keep the built-in text.
    assert_eq!(t_with(Locale::Fr, Msg::CodeLabel, &overrides), "Code");
    assert_eq!(t_with(Locale::De, Msg::CardHeadline, &overrides), t(Locale::De, Msg::CardHeadline));
    assert_eq!(t_with(Locale::De, Msg::CardHeadline, &[]), t(Locale::De, Msg::CardHeadline));
    // The first matching override wins.
    let twice = [(Msg::SendCta, "A"), (Msg::SendCta, "B")];
    assert_eq!(t_with(Locale::En, Msg::SendCta, &twice), "A");
}

#[test]
fn catalogue_is_complete_and_serialisable() {
    for locale in Locale::ALL {
        let entries = catalog(locale);
        assert_eq!(entries.len(), Msg::ALL.len());
        let keys: HashSet<&str> = entries.iter().map(|(key, _)| *key).collect();
        assert_eq!(keys.len(), entries.len(), "{locale}: duplicate keys");
        for ((key, text), msg) in entries.iter().zip(Msg::ALL) {
            assert_eq!(*key, msg.key());
            assert_eq!(*text, t(locale, *msg));
            assert_eq!(Msg::from_key(key), Some(*msg));
        }
        let object: serde_json::Map<String, serde_json::Value> =
            entries.iter().map(|(key, text)| ((*key).to_owned(), (*text).into())).collect();
        let json = serde_json::to_string(&object).expect("serialises");
        let back: serde_json::Value = serde_json::from_str(&json).expect("parses");
        assert_eq!(back["product_name"], t(locale, Msg::ProductName));
        assert_eq!(back.as_object().map(serde_json::Map::len), Some(Msg::ALL.len()));
    }
}

#[test]
fn native_language_names() {
    let names: Vec<&str> = Locale::ALL.into_iter().map(native_name).collect();
    assert_eq!(
        names,
        ["Français", "English", "Deutsch", "Español", "Italiano", "Nederlands", "Português"]
    );
}

#[test]
fn error_format_message_takes_the_code_length() {
    use ebc_model::GiftCode;
    let count = GiftCode::LEN.to_string();
    for locale in Locale::ALL {
        let text = tf(locale, Msg::ErrorCodeFormat, &[("count", &count)]);
        assert!(text.contains("10"), "{locale}: {text:?}");
    }
}
