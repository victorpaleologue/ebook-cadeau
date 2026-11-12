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
fn product_names_follow_the_terminology_research() {
    // docs/research/06-terminology.md §1.2 and §1.5.
    let expected = [
        (Locale::Fr, "eBook Cadeau"),
        (Locale::En, "eBook Gift"),
        (Locale::De, "eBook-Geschenk"),
        (Locale::Es, "eBook Regalo"),
        (Locale::It, "eBook Regalo"),
        (Locale::Nl, "eBook Cadeau"),
        (Locale::Pt, "eBook para Oferecer"),
    ];
    assert_eq!(expected.len(), Locale::ALL.len());
    for (locale, name) in expected {
        assert_eq!(t(locale, Msg::ProductName), name);
        // The page title of the creator names the product.
        assert!(
            t(locale, Msg::CreateTitle).contains(name),
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
    // (locale, the stand-alone noun, the spelling wherever "ebook" is written, the device)
    // — docs/research/06-terminology.md §1.2 and §1.4. French running text says « livre
    // numérique »; only the checkout option uses the trade form « ebook ».
    let expected = [
        (Locale::Fr, "livre numérique", "ebook", "liseuse"),
        (Locale::En, "eBook", "eBook", "e-reader"),
        (Locale::De, "eBook", "eBook", "eReader"),
        (Locale::Es, "eBook", "eBook", "eReader"),
        (Locale::It, "eBook", "eBook", "eReader"),
        (Locale::Nl, "e-book", "e-book", "e-reader"),
        (Locale::Pt, "eBook", "eBook", "eReader"),
    ];
    assert_eq!(expected.len(), Locale::ALL.len());
    for (locale, noun, ebook, ereader) in expected {
        assert_eq!(t(locale, Msg::EbookWord), noun, "{locale}");
        assert_eq!(t(locale, Msg::EreaderWord), ereader, "{locale}");
        let mut ebook_seen = 0usize;
        let mut ereader_seen = 0usize;
        for msg in Msg::ALL {
            // The product name is a proper noun and keeps its own spelling.
            let text = t(locale, *msg).replace(t(locale, Msg::ProductName), "");
            let text = text.replace("eBook Cadeau", "");
            for found in occurrences(&text, "ebook").into_iter().chain(occurrences(&text, "e-book"))
            {
                assert_eq!(found, ebook, "{locale}/{msg:?}: {text:?}");
                ebook_seen += 1;
            }
            for found in
                occurrences(&text, "ereader").into_iter().chain(occurrences(&text, "e-reader"))
            {
                assert_eq!(found, ereader, "{locale}/{msg:?}: {text:?}");
                ereader_seen += 1;
            }
        }
        assert!(ebook_seen > 0, "{locale}: the checker found no {ebook:?} at all");
        if locale != Locale::Fr {
            assert!(ereader_seen > 3, "{locale}: the checker found too few {ereader:?}");
        }
    }
    // French calls the device a "liseuse" everywhere, and never an "e-reader".
    assert!(t(Locale::Fr, Msg::CardStep2).contains("liseuse"));
    assert!(t(Locale::Fr, Msg::CheckoutOption).contains("ebook"));
    for msg in Msg::ALL {
        assert!(!t(Locale::Fr, *msg).to_lowercase().contains("reader"), "{msg:?}");
    }
}

/// Words that name a stored-value instrument (a balance, a voucher) in each language,
/// plus the scheme names, the print-category names ("gift book") and the phrases that
/// read as "free eBook" — the `stored_value`, `scheme_names`, `print_category` and
/// `reads_as_free` lists of `docs/research/06-terminology.md` §5.3, with a few more.
/// The gift is one prepaid book; these words would promise something else, and some of
/// them carry statutory validity rules. German entries match inside compounds.
fn stored_value_terms(locale: Locale) -> &'static [&'static str] {
    match locale {
        Locale::Fr => &[
            "carte cadeau",
            "carte-cadeau",
            "cartes cadeaux",
            "e-carte cadeau",
            "chèque cadeau",
            "chèque-cadeau",
            "bon cadeau",
            "bon d’achat",
            "bon d'achat",
            "bon pour",
            "certificat cadeau",
            "coffret cadeau",
            "chèque lire",
            "chèque culture",
            "kadéos",
            "cadhoc",
            "tir groupé",
            "illicado",
            "livre cadeau",
            "ebook offert",
            "valable jusqu",
        ],
        Locale::En => &[
            "gift card",
            "gift voucher",
            "voucher",
            "token",
            "egift",
            "coupon",
            "gift book",
            "valid until",
        ],
        Locale::De => {
            &["gutschein", "geschenkkarte", "geschenkbuch", "geschenk-ebook", "ebook geschenkt"]
        }
        Locale::Es => &[
            "tarjeta regalo",
            "tarjeta de regalo",
            "cheque regalo",
            "cheque de regalo",
            "bono",
            "cupón",
            "libro regalo",
            "ebook de regalo",
        ],
        Locale::It => &[
            "buono regalo",
            "carta regalo",
            "gift card",
            "buono",
            "buoni",
            "18app",
            "carta della cultura",
            "carta del docente",
            "libro regalo",
            "ebook in regalo",
            "ebook in omaggio",
        ],
        Locale::Nl => &["cadeaubon", "cadeaukaart", "boekenbon", "tegoedbon", "cadeauboek"],
        Locale::Pt => &[
            "cartão oferta",
            "cartão-oferta",
            "cartão de oferta",
            "cartão presente",
            "cartão de presente",
            "cheque-oferta",
            "cheque oferta",
            "cheque prenda",
            "vale",
            "vale-presente",
            "vale presente",
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

/// Every message that can be printed on the card.
const CARD_MESSAGES: [Msg; 22] = [
    Msg::CardHeadline,
    Msg::CardFrom,
    Msg::CardTo,
    Msg::CardHowTo,
    Msg::CardStep1,
    Msg::CardStep2,
    Msg::CardStep3,
    Msg::CardStep2Reseller,
    Msg::CardStep3Reseller,
    Msg::CardNoPhone,
    Msg::CodeLabel,
    Msg::CardAlreadyPaid,
    Msg::CardValidUntil,
    Msg::CardNoExpiry,
    Msg::CardHelp,
    Msg::CardBackDefault,
    Msg::CardLegalLine,
    Msg::CardLegalLineNoAuthor,
    Msg::CardSingleUse,
    Msg::FoldHere,
    Msg::CutHere,
    Msg::PrintedWith,
];

#[test]
fn card_steps_are_short_single_instructions() {
    for locale in Locale::ALL {
        for msg in [Msg::CardStep1, Msg::CardStep2, Msg::CardStep3] {
            let text = t(locale, msg);
            let length = text.chars().count();
            // 60 characters is the length budget of a printed step (06-terminology,
            // "Implications", item 3).
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
        assert_eq!(Msg::CardStep2Reseller.placeholders(), ["store"]);
        // The three steps are three different sentences, and so are the reseller ones.
        let steps: HashSet<&str> = [
            Msg::CardStep1,
            Msg::CardStep2,
            Msg::CardStep3,
            Msg::CardStep2Reseller,
            Msg::CardStep3Reseller,
        ]
        .into_iter()
        .map(|m| t(locale, m))
        .collect();
        assert_eq!(steps.len(), 5, "{locale}");
    }
}

#[test]
fn the_printed_card_never_says_qr() {
    // Printed steps say "this code"; "QR" is a screen word (06-terminology,
    // "Implications", item 12).
    for locale in Locale::ALL {
        for msg in CARD_MESSAGES {
            assert!(!t(locale, msg).contains("QR"), "{locale}/{msg:?}: {:?}", t(locale, msg));
        }
        assert!(t(locale, Msg::QrWord).contains("QR"), "{locale}");
        assert!(!t(locale, Msg::CheckoutOptionHelp).contains("QR"), "{locale}");
    }
}

#[test]
fn the_no_phone_line_names_the_browser_the_address_and_the_code() {
    // ereader-ux.ux.md §12: « Sans téléphone ? Sur la liseuse ou un ordinateur, ouvrir le
    // navigateur, aller sur {domaine} et saisir le code… »
    let browser = |locale| match locale {
        Locale::Fr => "navigateur",
        Locale::En | Locale::Nl | Locale::It => "browser",
        Locale::De => "Browser",
        Locale::Es | Locale::Pt => "navegador",
    };
    let computer = |locale| match locale {
        Locale::Fr => "ordinateur",
        Locale::En | Locale::It | Locale::Nl => "computer",
        Locale::De => "Computer",
        Locale::Es => "ordenador",
        Locale::Pt => "computador",
    };
    for locale in Locale::ALL {
        let text = t(locale, Msg::CardNoPhone);
        assert_eq!(Msg::CardNoPhone.placeholders(), ["url"]);
        assert_eq!(text.matches("{url}").count(), 1, "{locale}: {text:?}");
        assert!(text.contains(browser(locale)), "{locale}: {text:?}");
        assert!(text.contains(computer(locale)), "{locale}: {text:?}");
        assert!(text.contains(t(locale, Msg::EreaderWord)), "{locale}: {text:?}");
        // German capitalises the noun: compare without case.
        let code_word = t(locale, Msg::CodeLabel).to_lowercase();
        assert!(text.to_lowercase().contains(&code_word), "{locale}: {text:?}");
        // A question, then one sentence.
        assert_eq!(text.matches('?').count(), 1, "{locale}: {text:?}");
        assert!(text.ends_with('.'), "{locale}: {text:?}");
        let filled = tf(locale, Msg::CardNoPhone, &[("url", "ebk.example")]);
        assert!(filled.contains("ebk.example") && !filled.contains('{'), "{locale}: {filled:?}");
        assert!(filled.chars().count() <= 135, "{locale}: {} characters", filled.chars().count());
    }
    assert_eq!(
        tf(Locale::Fr, Msg::CardNoPhone, &[("url", "ebk.example")]),
        "Sans téléphone\u{a0}? Sur la liseuse ou un ordinateur, ouvrir le navigateur, aller sur \
         ebk.example et saisir le code de cette carte."
    );
}

#[test]
fn card_strings_are_the_recommended_ones() {
    // The canonical block of docs/research/06-terminology.md §5.1, key by key. (In the
    // French strings, U+00A0 stands before `:` and `!`.)
    type Row = (Locale, Msg, &'static str);
    let fr: [Row; 9] = [
        (Locale::Fr, Msg::CheckoutOption, "Offrir cet ebook"),
        (Locale::Fr, Msg::CardHeadline, "Ce livre est un cadeau"),
        (Locale::Fr, Msg::CardFrom, "De la part {de:name}"),
        (Locale::Fr, Msg::CardStep1, "Scanner ce code avec l’appareil photo d’un téléphone."),
        (Locale::Fr, Msg::CardStep2, "Choisir où lire\u{a0}: liseuse, tablette ou téléphone."),
        (Locale::Fr, Msg::CardStep3, "Le livre arrive. Bonne lecture\u{a0}!"),
        (Locale::Fr, Msg::RedeemCta, "Récupérer mon livre"),
        (Locale::Fr, Msg::CodeLabel, "Code"),
        (Locale::Fr, Msg::CardHowTo, "Pour le lire"),
    ];
    let en: [Row; 8] = [
        (Locale::En, Msg::CheckoutOption, "Give as a gift"),
        (Locale::En, Msg::CardHeadline, "This book is a gift"),
        (Locale::En, Msg::CardFrom, "From {name}"),
        (Locale::En, Msg::CardStep1, "Scan this code with a phone camera."),
        (Locale::En, Msg::CardStep2, "Choose where to read: e-reader, tablet or phone."),
        (Locale::En, Msg::CardStep3, "The book arrives. Happy reading!"),
        (Locale::En, Msg::RedeemCta, "Get my book"),
        (Locale::En, Msg::CodeLabel, "Code"),
    ];
    let de: [Row; 8] = [
        (Locale::De, Msg::CheckoutOption, "eBook verschenken"),
        (Locale::De, Msg::CardHeadline, "Dieses Buch ist ein Geschenk"),
        (Locale::De, Msg::CardFrom, "Von {name}"),
        (Locale::De, Msg::CardStep1, "Diesen Code mit der Kamera eines Smartphones scannen."),
        (
            Locale::De,
            Msg::CardStep2,
            "Auswählen, wo gelesen wird: eReader, Tablet oder Smartphone.",
        ),
        (Locale::De, Msg::CardStep3, "Das Buch ist da. Viel Freude beim Lesen!"),
        (Locale::De, Msg::RedeemCta, "Mein Buch abholen"),
        (Locale::De, Msg::CodeLabel, "Code"),
    ];
    let es: [Row; 8] = [
        (Locale::Es, Msg::CheckoutOption, "Regalar este eBook"),
        (Locale::Es, Msg::CardHeadline, "Este libro es un regalo"),
        (Locale::Es, Msg::CardFrom, "De parte de {name}"),
        (Locale::Es, Msg::CardStep1, "Escanear este código con la cámara de un móvil."),
        (Locale::Es, Msg::CardStep2, "Elegir dónde leer: eReader, tableta o móvil."),
        (Locale::Es, Msg::CardStep3, "El libro llega. ¡Feliz lectura!"),
        (Locale::Es, Msg::RedeemCta, "Conseguir mi libro"),
        (Locale::Es, Msg::CodeLabel, "Código"),
    ];
    let it: [Row; 8] = [
        (Locale::It, Msg::CheckoutOption, "Regala questo eBook"),
        (Locale::It, Msg::CardHeadline, "Questo libro è un regalo"),
        (Locale::It, Msg::CardFrom, "Da parte di {name}"),
        (Locale::It, Msg::CardStep1, "Inquadrare questo codice con la fotocamera di un telefono."),
        (Locale::It, Msg::CardStep2, "Scegliere dove leggere: eReader, tablet o smartphone."),
        (Locale::It, Msg::CardStep3, "Il libro arriva. Buona lettura!"),
        (Locale::It, Msg::RedeemCta, "Ricevi il libro"),
        (Locale::It, Msg::CodeLabel, "Codice"),
    ];
    let nl: [Row; 8] = [
        (Locale::Nl, Msg::CheckoutOption, "Cadeau doen"),
        (Locale::Nl, Msg::CardHeadline, "Dit boek is een cadeau"),
        (Locale::Nl, Msg::CardFrom, "Van {name}"),
        (Locale::Nl, Msg::CardStep1, "Scan deze code met de camera van een telefoon."),
        (Locale::Nl, Msg::CardStep2, "Kies waar je wilt lezen: e-reader, tablet of telefoon."),
        (Locale::Nl, Msg::CardStep3, "Het boek staat klaar. Veel leesplezier!"),
        (Locale::Nl, Msg::RedeemCta, "Mijn boek ophalen"),
        (Locale::Nl, Msg::CodeLabel, "Code"),
    ];
    let pt: [Row; 8] = [
        (Locale::Pt, Msg::CheckoutOption, "Oferecer este eBook"),
        (Locale::Pt, Msg::CardHeadline, "Este livro é um presente"),
        (Locale::Pt, Msg::CardFrom, "De {name}"),
        (Locale::Pt, Msg::CardStep1, "Apontar a câmara de um telemóvel para este código."),
        (Locale::Pt, Msg::CardStep2, "Escolher onde ler: eReader, tablet ou telemóvel."),
        (Locale::Pt, Msg::CardStep3, "O livro chega. Boa leitura!"),
        (Locale::Pt, Msg::RedeemCta, "Receber o meu livro"),
        (Locale::Pt, Msg::CodeLabel, "Código"),
    ];
    let rows = fr.iter().chain(&en).chain(&de).chain(&es).chain(&it).chain(&nl).chain(&pt);
    let mut locales = HashSet::new();
    for (locale, msg, text) in rows {
        assert_eq!(t(*locale, *msg), *text, "{locale}/{msg:?}");
        locales.insert(*locale);
    }
    assert_eq!(locales.len(), Locale::ALL.len());
    // The reseller variant of steps 2 and 3 and the legal line, in French (UX note §12).
    assert_eq!(
        tf(Locale::Fr, Msg::CardStep2Reseller, &[("store", "Septante")]),
        "Se connecter à son compte Septante, ou en créer un."
    );
    assert_eq!(
        t(Locale::Fr, Msg::CardStep3Reseller),
        "Le livre apparaît dans la bibliothèque, sur la liseuse et l’application."
    );
    assert_eq!(
        tf(Locale::Fr, Msg::CardLegalLine, &[("title", "Germinal"), ("author", "Émile Zola")]),
        "Germinal, Émile Zola. Un exemplaire numérique, déjà payé\u{a0}: rien à régler."
    );
    assert_eq!(
        tf(Locale::Fr, Msg::CardLegalLineNoAuthor, &[("title", "Germinal")]),
        "Germinal. Un exemplaire numérique, déjà payé\u{a0}: rien à régler."
    );
    assert_eq!(t(Locale::Fr, Msg::CardSingleUse), "Usage unique.");
    assert_eq!(t(Locale::Fr, Msg::CardNoExpiry), "Sans date limite.");
}

#[test]
fn legal_line_without_author_is_the_legal_line_minus_the_author() {
    for locale in Locale::ALL {
        let with = t(locale, Msg::CardLegalLine);
        let without = t(locale, Msg::CardLegalLineNoAuthor);
        assert_eq!(with.replace("{title}, {author}.", "{title}."), without, "{locale}");
        assert!(with.starts_with("{title}, {author}. "), "{locale}: {with:?}");
    }
}

#[test]
fn french_card_uses_the_infinitive_and_screens_say_vous() {
    // The printed card does not choose between tu and vous on behalf of the giver
    // (ereader-ux.ux.md §12): instructions are infinitives, and no card line holds a
    // second-person pronoun or possessive.
    for msg in [Msg::CardStep1, Msg::CardStep2, Msg::CardStep2Reseller] {
        let text = t(Locale::Fr, msg);
        let mut words = text.split_whitespace();
        let first = words.next().unwrap_or_default();
        let verb = if first == "Se" { words.next().unwrap_or_default() } else { first };
        assert!(
            ["er", "ir", "re"].iter().any(|ending| verb.ends_with(ending)),
            "{msg:?} does not start with an infinitive: {text:?}"
        );
    }
    for msg in CARD_MESSAGES {
        let lower = t(Locale::Fr, msg).to_lowercase();
        for pronoun in ["vous", "votre", "vos", "tu", "ton", "ta", "tes", "toi", "t’"] {
            assert!(!contains_term(&lower, pronoun, false), "{msg:?} uses {pronoun:?}: {lower:?}");
        }
        for word in lower.split(|c: char| !c.is_alphabetic()) {
            assert!(
                !(word.len() > 3 && word.ends_with("ez")),
                "{msg:?} holds the imperative {word:?}: {lower:?}"
            );
        }
    }
    // Screens address the reader with vous; nothing anywhere says tu.
    assert!(t(Locale::Fr, Msg::RedeemIntro).contains("votre"));
    assert!(t(Locale::Fr, Msg::ChooseDevice).contains("vous"));
    for msg in Msg::ALL {
        let lower = t(Locale::Fr, *msg).to_lowercase();
        for informal in ["tu", "ton", "ta", "tes", "toi", "t’"] {
            assert!(
                !contains_term(&lower, informal, false),
                "{msg:?} uses {informal:?}: {lower:?}"
            );
        }
    }
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
fn dates_fit_the_collect_before_sentence() {
    use ebc_i18n::format_date;
    // "To be collected before", never "valid until" (06-terminology, "Implications",
    // item 13).
    let expected = [
        (Locale::Fr, "À récupérer avant le 1er mars 2027."),
        (Locale::En, "To be collected before 1 March 2027."),
        (Locale::De, "Abholen bis 1. März 2027."),
        (Locale::Es, "Recoger antes del 1 de marzo de 2027."),
        (Locale::It, "Da ritirare entro il giorno 1º marzo 2027."),
        (Locale::Nl, "Ophalen vóór 1 maart 2027."),
        (Locale::Pt, "Receber até 1 de março de 2027."),
    ];
    for (locale, sentence) in expected {
        let date = format_date(locale, 2027, 3, 1).expect("valid date");
        assert_eq!(tf(locale, Msg::CardValidUntil, &[("date", &date)]), sentence);
    }
    // Days whose number starts with a vowel sound must not need an elided article
    // (Italian « entro l’8 », « entro l’11 »): the sentence is built so that none is needed.
    for day in [8, 11] {
        let date = format_date(Locale::It, 2027, 3, day).expect("valid date");
        let sentence = tf(Locale::It, Msg::CardValidUntil, &[("date", &date)]);
        assert!(sentence.starts_with(&format!("Da ritirare entro il giorno {day} ")), "{sentence}");
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
    use ebc_model::{GiftCode, PairCode};
    // The message is told how long the code is; it must not assume a length itself.
    assert_eq!(Msg::ErrorCodeFormat.placeholders(), ["count"]);
    let letters = |locale| match locale {
        Locale::Fr => "lettres",
        Locale::En | Locale::Nl => "letters",
        Locale::De => "Buchstaben",
        Locale::Es | Locale::Pt => "letras",
        Locale::It => "lettere",
    };
    for len in [GiftCode::LEN, PairCode::LEN] {
        let count = len.to_string();
        for locale in Locale::ALL {
            let text = tf(locale, Msg::ErrorCodeFormat, &[("count", &count)]);
            assert!(!text.contains(['{', '}']), "{locale}: {text:?}");
            // A code is consonants only (`GiftCode::ALPHABET`): the message says "N
            // letters", not "N characters: letters and digits".
            let expected = format!("{count} {}", letters(locale));
            assert!(text.contains(&expected), "{locale}: {text:?} lacks {expected:?}");
        }
    }
    // No other number is baked into any translation.
    for locale in Locale::ALL {
        let template = t(locale, Msg::ErrorCodeFormat);
        assert!(!template.chars().any(|c| c.is_ascii_digit()), "{locale}: {template:?}");
    }
    assert!(GiftCode::ALPHABET.chars().all(|c| c.is_ascii_uppercase()));
}
