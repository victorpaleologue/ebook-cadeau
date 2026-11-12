//! Localised strings and locale conventions (terminology, spine direction, dates) for
//! eBook Cadeau.
//!
//! Three things live here:
//!
//! * the **messages** printed on the card and shown by the web pages ([`Msg`], [`t`],
//!   [`tf`]), in the seven supported [`Locale`]s;
//! * the **spine direction** convention ([`SpineDirection`], [`spine_direction`],
//!   [`spine_direction_for_language`], [`spine_direction_for_book`]): whether a title set
//!   along the spine reads top-to-bottom or bottom-to-top;
//! * long **dates** ([`format_date`], [`format_unix_date`]).
//!
//! ```
//! use ebc_i18n::{Locale, Msg, SpineDirection, spine_direction_for_language, t, tf};
//!
//! assert_eq!(t(Locale::Fr, Msg::ProductName), "eBook Cadeau");
//! assert_eq!(tf(Locale::En, Msg::CardFrom, &[("name", "Alice")]), "From Alice");
//! assert_eq!(tf(Locale::Fr, Msg::CardFrom, &[("name", "Alice")]), "De la part d’Alice");
//! assert_eq!(spine_direction_for_language("fr-CA"), SpineDirection::BottomToTop);
//! ```
//!
//! # Wording rules
//!
//! The strings follow `docs/research/06-terminology.md` (§5.1 for the canonical set, §5.2
//! for the other card lines) and, for the text printed inside the card, §12 of
//! `docs/research/notes/ereader-ux.ux.md`. Strings those documents do not give were
//! drafted to the same rules.
//!
//! * The gift is **one prepaid book**, never a balance. No string uses a stored-value
//!   word ("carte cadeau", "Gutschein", "cadeaubon", "tarjeta regalo", "buono regalo",
//!   "gift card", "voucher", …): those words carry legal expectations (validity periods,
//!   refunds) that do not apply. For the same reason an expiry date is worded "to be
//!   collected before", never "valid until". A test enforces the word list.
//! * **The printed card** carries three steps ([`Msg::CardStep1`]–[`Msg::CardStep3`], with
//!   a variant of steps 2 and 3 for gifts fulfilled by a store) and one fallback line for
//!   people without a phone ([`Msg::CardNoPhone`]). It never says "QR": the steps say
//!   "this code".
//! * What the recipient types is always called a **code**. It is twelve consonants (six
//!   for a pairing code): no message mentions digits as part of it.
//! * **Register**: the card uses the infinitive of instructions in French, German,
//!   Spanish, Italian and Portuguese, so that it chooses no form of address on behalf of
//!   the giver; Dutch and English use the imperative. Where a pronoun is unavoidable
//!   (screens, mostly): French *vous*, Dutch *je*, Spanish *tú*, Italian *tu*, Portuguese
//!   (pt-PT) the formal third person, German none. Buttons are in the first person
//!   ("Récupérer mon livre"), which is register-neutral.
//! * **Nouns**: the trade spelling "eBook" (German and Romance languages), "e-book" in
//!   Dutch; French running text says « livre numérique » and only the checkout option
//!   says « ebook ». The device is « liseuse », "e-reader" (English, Dutch) or "eReader".
//! * Typography: French strings put a no-break space (U+00A0) before `? ! : ;`;
//!   apostrophes are typographic (U+2019). The languages that set a space between a
//!   number and `%` (French, German, Spanish, Portuguese) make it a no-break space; the
//!   other strings hold no no-break space at all. Only characters found in ordinary Latin
//!   text fonts are used.
//! * **The sheet** ([`Msg::PrintDuplexShortEdge`], [`Msg::PrintDuplexLongEdge`],
//!   [`Msg::PrintCutFold`]): the instructions in the margin of a sheet printed at home
//!   follow the register of the card (infinitive, or imperative in English and Dutch)
//!   and use the words of the print dialogs ("flip on the short edge", « retourner sur le
//!   bord court », „an der kurzen Kante spiegeln“).
//!
//! **Not verified.** The research itself is unverified at source, and no string has been
//! reviewed by a native speaker; Portuguese is the least reliable. See [`t_with`] for
//! overriding any message per reseller. Not implemented here, although the research asks
//! for them: per-tenant noun style and register, regional deltas (fr-CA, pt-BR, es-419),
//! and a catalogue kept in data files: the tables in `src/strings/` are the only source
//! of the strings.
//!
//! # Templates
//!
//! Some messages contain `{name}` placeholders (listed by [`Msg::placeholders`]), filled
//! by [`tf`]. Substitution is single-pass: a value that itself contains `{…}` is inserted
//! verbatim and never re-expanded. Values are **not** escaped for HTML — that is the
//! caller's (or its template engine's) job.

#![forbid(unsafe_code)]

mod date;
mod spine;
mod strings;
mod template;

pub use date::{date_from_unix, format_date, format_unix_date};
pub use ebc_model::Locale;
pub use spine::{
    SpineDirection, known_spine_direction, spine_direction, spine_direction_for_book,
    spine_direction_for_language,
};
pub use template::{format_template, placeholders_in};

/// Declares [`Msg`] together with its stable keys and the complete list of variants.
macro_rules! messages {
    ($( $(#[$doc:meta])* $variant:ident => $key:literal, [$($placeholder:literal),*]; )+) => {
        /// A translatable message.
        ///
        /// Variants are grouped by where they are used: the product and the checkout
        /// option of a store, the printed card, the redemption pages, the card creator.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[non_exhaustive]
        pub enum Msg {
            $( $(#[$doc])* $variant, )+
        }

        impl Msg {
            /// Every message, in declaration order.
            pub const ALL: &'static [Msg] = &[ $( Msg::$variant, )+ ];

            /// A stable `snake_case` identifier, for JSON catalogues and per-reseller
            /// overrides.
            #[must_use]
            pub const fn key(self) -> &'static str {
                match self { $( Msg::$variant => $key, )+ }
            }

            /// The names of the `{placeholder}`s every translation of this message
            /// contains, to be supplied to [`tf`].
            #[must_use]
            pub const fn placeholders(self) -> &'static [&'static str] {
                match self { $( Msg::$variant => &[$($placeholder),*], )+ }
            }
        }
    };
}

messages! {
    // ---- Product and checkout (a store's "eBook cadeau" option) -------------------------

    /// Generic name of the product, in the site's language ("eBook cadeau").
    ProductName => "product_name", [];
    /// One-line description of the product.
    ProductTagline => "product_tagline", [];
    /// Label of the checkout option that turns an eBook purchase into a gift.
    CheckoutOption => "checkout_option", [];
    /// Help text under the checkout option.
    CheckoutOptionHelp => "checkout_option_help", [];
    /// Sub-option: the buyer prints the card.
    CheckoutPrintAtHome => "checkout_print_at_home", [];
    /// Sub-option: the card is printed professionally and posted.
    CheckoutPrintedByPost => "checkout_printed_by_post", [];

    // ---- The printed card ------------------------------------------------------------

    /// Headline inside the card.
    CardHeadline => "card_headline", [];
    /// Who gives the book. Placeholder: `name`.
    CardFrom => "card_from", ["name"];
    /// Who receives the book. Placeholder: `name`.
    CardTo => "card_to", ["name"];
    /// Heading above the three steps.
    CardHowTo => "card_how_to", [];
    /// Step 1: scan the code with a phone camera. The card never says "QR": the printed
    /// steps say "this code" (see [`Msg::QrWord`] for screens).
    CardStep1 => "card_step1", [];
    /// Step 2 (open file): choose where to read.
    CardStep2 => "card_step2", [];
    /// Step 3 (open file): the book arrives.
    CardStep3 => "card_step3", [];
    /// Step 2 when a store fulfils the gift: sign in. Placeholder: `store`.
    CardStep2Reseller => "card_step2_reseller", ["store"];
    /// Step 3 when a store fulfils the gift: the book appears in the library.
    CardStep3Reseller => "card_step3_reseller", [];
    /// Fallback line for recipients without a camera phone: open the browser of the
    /// e-reader or of a computer, go to the address, type the code. Placeholder: `url`
    /// (the short address to type, normally a bare host).
    CardNoPhone => "card_no_phone", ["url"];
    /// Label printed before the code.
    CodeLabel => "code_label", [];
    /// Reassurance line: the book is already paid for.
    CardAlreadyPaid => "card_already_paid", [];
    /// Collect-before line, when the gift has an expiry date. Worded "to be collected
    /// before", never "valid until", which belongs to stored-value cards. Placeholder:
    /// `date` (see [`format_date`]).
    CardValidUntil => "card_valid_until", ["date"];
    /// Line saying the gift has no time limit.
    CardNoExpiry => "card_no_expiry", [];
    /// Where to get help. Placeholder: `url`.
    CardHelp => "card_help", ["url"];
    /// Default text of the back panel when the book has no blurb.
    CardBackDefault => "card_back_default", [];
    /// Legal line: what the card gives — one digital copy of a named book, already paid.
    /// Placeholders: `title`, `author`. Followed on the card by [`Msg::CardSingleUse`] and
    /// [`Msg::CardNoExpiry`] or [`Msg::CardValidUntil`].
    CardLegalLine => "card_legal_line", ["title", "author"];
    /// [`Msg::CardLegalLine`] for a book without a known author. Placeholder: `title`.
    CardLegalLineNoAuthor => "card_legal_line_no_author", ["title"];
    /// Legal mention: the code works once.
    CardSingleUse => "card_single_use", [];
    /// Guide printed next to a fold line (home printing).
    FoldHere => "fold_here", [];
    /// Guide printed next to a cut line (home printing).
    CutHere => "cut_here", [];
    /// Small credit line.
    PrintedWith => "printed_with", [];
    /// The word for an eBook, as a stand-alone label. Never interpolated into a sentence:
    /// sentences that name the book are stored whole.
    EbookWord => "ebook_word", [];
    /// The word for a dedicated reading device.
    EreaderWord => "ereader_word", [];
    /// The words for a QR code, for screens, alternative text and help pages — not for
    /// the printed card.
    QrWord => "qr_word", [];

    // ---- The printed sheet: margin notes, cover credit, job line ------------------------

    /// Instruction in the margin of a sheet printed at home, for a sheet that is turned
    /// over on its **short** edge: print at 100 % (actual size), on both sides, flip on
    /// the short edge. Which of this and [`Msg::PrintDuplexLongEdge`] applies is a matter
    /// of geometry (`ebc_layout::SheetPlacement::duplex_flip`), not of language.
    PrintDuplexShortEdge => "print_duplex_short_edge", [];
    /// [`Msg::PrintDuplexShortEdge`] for a sheet that is turned over on its **long** edge.
    PrintDuplexLongEdge => "print_duplex_long_edge", [];
    /// Instruction in the margin of a sheet printed at home: cut on the solid marks, fold
    /// on the dashed ones.
    PrintCutFold => "print_cut_fold", [];
    /// Credit line for the cover image, with a credit text supplied by the caller and
    /// printed as given. Placeholder: `credit` (e.g. `© Jeanne Roy / Atelier du Lac`).
    CoverCredit => "cover_credit", ["credit"];
    /// Credit line for the cover image when only the publisher is known. Placeholder:
    /// `publisher`.
    CoverCreditPublisher => "cover_credit_publisher", ["publisher"];
    /// Credit line for a cover image in the public domain.
    CoverCreditPublicDomain => "cover_credit_public_domain", [];
    /// Credit line for a cover image whose rights holder is not known to the composer.
    CoverCreditUnknown => "cover_credit_unknown", [];
    /// The size of the card, for the job line in the slug of a press sheet. Placeholders:
    /// `width` and `height` of one cover and `spine`, the width of the spine, all in
    /// millimetres, as plain numbers.
    CardFormatLine => "card_format_line", ["width", "height", "spine"];
    /// Label: the outside of the card (back cover, spine and front cover) — the
    /// counterpart of [`Msg::InsideLabel`], also printed in the job line of a press sheet.
    OutsideLabel => "outside_label", [];

    // ---- Redemption pages ------------------------------------------------------------

    /// Title of the redemption page.
    RedeemTitle => "redeem_title", [];
    /// Instruction above the code field.
    RedeemIntro => "redeem_intro", [];
    /// The main button: get the book.
    RedeemCta => "redeem_cta", [];
    /// Greeting naming the giver. Placeholder: `name`.
    RedeemGiftFrom => "redeem_gift_from", ["name"];
    /// Greeting when the giver is not named.
    RedeemGiftAnonymous => "redeem_gift_anonymous", [];
    /// Question above the list of devices.
    ChooseDevice => "choose_device", [];
    /// Device choice: the device showing the page.
    DeviceThis => "device_this", [];
    /// Device choice: a dedicated e-reader.
    DeviceEreader => "device_ereader", [];
    /// Device choice: phone or tablet.
    DevicePhoneTablet => "device_phone_tablet", [];
    /// Device choice: computer.
    DeviceComputer => "device_computer", [];
    /// Button: download the file.
    DownloadCta => "download_cta", [];
    /// Button: read in the browser.
    ReadNowCta => "read_now_cta", [];
    /// Button: send the file to an e-reader.
    SendToEreaderCta => "send_to_ereader_cta", [];
    /// Button: add to the store library.
    AddToLibraryCta => "add_to_library_cta", [];
    /// Button leading to the store. Placeholder: `store`.
    ContinueAtStore => "continue_at_store", ["store"];
    /// How to redeem from the e-reader's own browser. Placeholder: `url`.
    EreaderBrowserHint => "ereader_browser_hint", ["url"];
    /// Label of an e-mail address field.
    EmailLabel => "email_label", [];
    /// Button: send.
    SendCta => "send_cta", [];
    /// Confirmation after sending to an e-reader.
    SentConfirmation => "sent_confirmation", [];
    /// Notice on a gift that was already redeemed.
    AlreadyRedeemed => "already_redeemed", [];
    /// Closing wish.
    EnjoyReading => "enjoy_reading", [];
    /// Error: the check character does not match (`CodeError::Check`).
    ErrorCodeTypo => "error_code_typo", [];
    /// Error: wrong length or character (`CodeError::Length`, `CodeError::InvalidChar`).
    /// Placeholder: `count`, the number of letters of the code that was expected
    /// (`GiftCode::LEN` or `PairCode::LEN`). Codes are consonants only.
    ErrorCodeFormat => "error_code_format", ["count"];
    /// Error: well-formed code that was never issued.
    ErrorCodeUnknown => "error_code_unknown", [];
    /// Error: the gift has expired.
    ErrorGiftExpired => "error_gift_expired", [];
    /// Error: the gift was cancelled.
    ErrorGiftRevoked => "error_gift_revoked", [];
    /// Error: the gift is not active yet.
    ErrorGiftPending => "error_gift_pending", [];
    /// Error: rate limit reached.
    ErrorTooManyAttempts => "error_too_many_attempts", [];
    /// Error: anything else.
    ErrorGeneric => "error_generic", [];

    // ---- Card creator (web) -----------------------------------------------------------

    /// Title of the creation page.
    CreateTitle => "create_title", [];
    /// Field label: ISBN.
    FieldIsbn => "field_isbn", [];
    /// Field label: title.
    FieldTitle => "field_title", [];
    /// Field label: author.
    FieldAuthor => "field_author", [];
    /// Field label: the giver's name.
    FieldGiver => "field_giver", [];
    /// Field label: the recipient's name.
    FieldRecipient => "field_recipient", [];
    /// Field label: personal message.
    FieldMessage => "field_message", [];
    /// Field label: cover image.
    FieldCover => "field_cover", [];
    /// Field label: the eBook file.
    FieldFile => "field_file", [];
    /// Button: look the ISBN up.
    LookupCta => "lookup_cta", [];
    /// Button: generate the card.
    GenerateCta => "generate_cta", [];
    /// Button: download the PDF.
    DownloadPdfCta => "download_pdf_cta", [];
    /// Button: print at home.
    PrintAtHomeCta => "print_at_home_cta", [];
    /// Button: order a professional print.
    OrderPrintCta => "order_print_cta", [];
    /// Label: preview.
    PreviewLabel => "preview_label", [];
    /// Label: the spine panel.
    SpineLabel => "spine_label", [];
    /// Label: the front cover panel.
    FrontLabel => "front_label", [];
    /// Label: the back cover panel.
    BackLabel => "back_label", [];
    /// Label: the inside of the card.
    InsideLabel => "inside_label", [];
    /// Question introducing the rights-basis choices.
    RightsBasisLabel => "rights_basis_label", [];
    /// Rights basis: the uploader's own work.
    RightsOwnWork => "rights_own_work", [];
    /// Rights basis: public domain.
    RightsPublicDomain => "rights_public_domain", [];
    /// Rights basis: open licence.
    RightsOpenLicence => "rights_open_licence", [];
    /// Rights basis: authorised distributor.
    RightsLicensedDistributor => "rights_licensed_distributor", [];
    /// Checkbox: the uploader confirms the rights to the file and the cover.
    RightsConfirm => "rights_confirm", [];
    /// Notice in the upload dialog: store purchases cannot be passed on.
    UploadPurchasedNotice => "upload_purchased_notice", [];
    /// Error: no cover image found.
    CoverNotFound => "cover_not_found", [];
    /// Error: invalid ISBN.
    IsbnInvalid => "isbn_invalid", [];
    /// Error: no catalogue record for the ISBN.
    BookNotFound => "book_not_found", [];
    /// Progress indicator.
    Loading => "loading", [];
    /// Label of the language selector.
    LanguageLabel => "language_label", [];
    /// Notice shown by the demonstration stores.
    DemoNotice => "demo_notice", [];
}

impl Msg {
    /// Finds a message by its [`key`](Self::key).
    #[must_use]
    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|msg| msg.key() == key)
    }
}

/// The translation of `msg` that `locale` itself provides, or `None` if that locale has
/// none (in which case [`t`] falls back to English, then French).
#[must_use]
pub fn translation(locale: Locale, msg: Msg) -> Option<&'static str> {
    let text = strings::text(locale, msg);
    (!text.is_empty()).then_some(text)
}

/// The text of `msg` in `locale`.
///
/// Never panics. Falls back to English, then French, if `locale` has no translation
/// (every message currently has one in every locale; a test enforces it).
#[must_use]
pub fn t(locale: Locale, msg: Msg) -> &'static str {
    with_fallback(locale, |l| strings::text(l, msg))
}

/// Applies the fallback chain `locale → English → French` to a lookup function.
fn with_fallback(locale: Locale, lookup: impl Fn(Locale) -> &'static str) -> &'static str {
    [locale, Locale::En, Locale::Fr]
        .into_iter()
        .map(lookup)
        .find(|text| !text.is_empty())
        .unwrap_or_default()
}

/// The text of `msg` in `locale` with `{name}` placeholders replaced.
///
/// `args` are `(placeholder, value)` pairs; see [`Msg::placeholders`] for the names a
/// message expects. Placeholders without a matching argument are left as they are;
/// arguments without a matching placeholder are ignored. See [`format_template`] for the
/// exact rules.
#[must_use]
pub fn tf(locale: Locale, msg: Msg, args: &[(&str, &str)]) -> String {
    format_template(t(locale, msg), args)
}

/// Like [`t`], but lets a caller replace individual messages — for instance a reseller
/// that spells "eBook" differently or has its own checkout wording.
///
/// The first non-blank override for `msg` wins; otherwise the built-in text is returned.
#[must_use]
pub fn t_with<'a>(locale: Locale, msg: Msg, overrides: &'a [(Msg, &'a str)]) -> &'a str {
    overrides
        .iter()
        .find(|(overridden, text)| *overridden == msg && !text.trim().is_empty())
        .map_or_else(|| t(locale, msg), |(_, text)| *text)
}

/// Like [`tf`], on top of [`t_with`].
#[must_use]
pub fn tf_with(
    locale: Locale,
    msg: Msg,
    args: &[(&str, &str)],
    overrides: &[(Msg, &str)],
) -> String {
    format_template(t_with(locale, msg, overrides), args)
}

/// Every message of `locale` as `(key, text)` pairs in declaration order — the shape a
/// web front end wants as a JSON object.
#[must_use]
pub fn catalog(locale: Locale) -> Vec<(&'static str, &'static str)> {
    Msg::ALL.iter().map(|msg| (msg.key(), t(locale, *msg))).collect()
}

/// The name of a language in that language, for a language selector: `"Français"`,
/// `"English"`, `"Deutsch"`…
#[must_use]
pub const fn native_name(locale: Locale) -> &'static str {
    match locale {
        Locale::Fr => "Français",
        Locale::En => "English",
        Locale::De => "Deutsch",
        Locale::Es => "Español",
        Locale::It => "Italiano",
        Locale::Nl => "Nederlands",
        Locale::Pt => "Português",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_goes_to_english_then_french() {
        let only_fr = |l: Locale| if l == Locale::Fr { "fr" } else { "" };
        let fr_en = |l: Locale| match l {
            Locale::Fr => "fr",
            Locale::En => "en",
            _ => "",
        };
        let all = |l: Locale| l.code();
        for locale in Locale::ALL {
            assert_eq!(with_fallback(locale, only_fr), "fr");
            assert_eq!(with_fallback(locale, all), locale.code());
            let expected = if locale == Locale::Fr { "fr" } else { "en" };
            assert_eq!(with_fallback(locale, fr_en), expected);
            assert_eq!(with_fallback(locale, |_| ""), "", "nothing anywhere: empty, no panic");
        }
    }

    #[test]
    fn keys_are_unique_snake_case_and_reversible() {
        let mut keys: Vec<&str> = Msg::ALL.iter().map(|m| m.key()).collect();
        for key in &keys {
            assert!(!key.is_empty());
            assert!(
                key.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_'),
                "{key}"
            );
        }
        for msg in Msg::ALL {
            assert_eq!(Msg::from_key(msg.key()), Some(*msg));
        }
        keys.sort_unstable();
        let before = keys.len();
        keys.dedup();
        assert_eq!(keys.len(), before, "duplicate keys");
        assert_eq!(Msg::from_key("no_such_key"), None);
        assert_eq!(Msg::from_key(""), None);
        assert_eq!(Msg::ProductName.key(), "product_name");
        assert_eq!(Msg::CardStep1.key(), "card_step1");
    }
}
