//! Device classification from the `User-Agent` header (`docs/api.md` §2, "Device classes").
//!
//! Sniffing is a **hint, never a gate**: it broke twice between 2024 and 2026 (newer Kobo
//! and Kobo-based tolino firmware report a bare `eReader`; a 2024 Kindle on firmware 5.18
//! no longer says `Kindle` — see `docs/research/notes/ereader-ux.devices.md` §1, §3). Every
//! gift page therefore carries a device chooser and honours `?device=`
//! ([`DeviceClass::from_choice`]).

/// What kind of device asks for the page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeviceClass {
    /// Rakuten Kobo e-reader (Nickel browser).
    Kobo,
    /// Amazon Kindle e-ink reader (not the Fire tablets, not the Kindle apps).
    Kindle,
    /// PocketBook e-reader (also Vivlio, which is PocketBook hardware).
    Pocketbook,
    /// tolino e-reader.
    Tolino,
    /// Another e-ink reader, or one that only says `eReader`.
    EinkOther,
    /// Phone or tablet.
    Mobile,
    /// Anything else: desktop browsers, tools, unknown agents.
    Desktop,
}

impl DeviceClass {
    /// Every class, in the order of the contract.
    pub const ALL: [DeviceClass; 7] = [
        Self::Kobo,
        Self::Kindle,
        Self::Pocketbook,
        Self::Tolino,
        Self::EinkOther,
        Self::Mobile,
        Self::Desktop,
    ];

    /// The class name used by the contract: `kobo`, `kindle`, `pocketbook`, `tolino`,
    /// `eink_other`, `mobile`, `desktop`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Kobo => "kobo",
            Self::Kindle => "kindle",
            Self::Pocketbook => "pocketbook",
            Self::Tolino => "tolino",
            Self::EinkOther => "eink_other",
            Self::Mobile => "mobile",
            Self::Desktop => "desktop",
        }
    }

    /// The value of the `?device=` parameter that selects this class, when there is one:
    /// `kobo`, `kindle`, `pocketbook`, `tolino`, `phone`, `computer`.
    #[must_use]
    pub const fn choice(self) -> Option<&'static str> {
        match self {
            Self::Kobo => Some("kobo"),
            Self::Kindle => Some("kindle"),
            Self::Pocketbook => Some("pocketbook"),
            Self::Tolino => Some("tolino"),
            Self::Mobile => Some("phone"),
            Self::Desktop => Some("computer"),
            Self::EinkOther => None,
        }
    }

    /// Parses a `?device=` value (`kobo|kindle|pocketbook|tolino|phone|computer`; the
    /// class names `mobile` and `desktop` are accepted too). Case-insensitive; anything
    /// else is `None` and the detection applies.
    #[must_use]
    pub fn from_choice(value: &str) -> Option<Self> {
        Some(match value.trim().to_ascii_lowercase().as_str() {
            "kobo" => Self::Kobo,
            "kindle" => Self::Kindle,
            "pocketbook" => Self::Pocketbook,
            "tolino" => Self::Tolino,
            "phone" | "mobile" => Self::Mobile,
            "computer" | "desktop" => Self::Desktop,
            _ => return None,
        })
    }

    /// Whether the device is a dedicated e-ink reader.
    #[must_use]
    pub const fn is_eink(self) -> bool {
        matches!(
            self,
            Self::Kobo | Self::Kindle | Self::Pocketbook | Self::Tolino | Self::EinkOther
        )
    }

    /// Whether the built-in browser of this class saves a file of this media type to the
    /// device (so that a direct download button is the right primary action).
    ///
    /// Kindle browsers only save Kindle formats and plain text; EPUB goes through Send to
    /// Kindle and PDF downloads are reported failing
    /// (`docs/research/notes/ereader-ux.devices.md` §2, §3.1). Every other class takes
    /// what we serve.
    #[must_use]
    pub fn browser_saves(self, media_type: &str) -> bool {
        match self {
            Self::Kindle => matches!(
                media_type.to_ascii_lowercase().as_str(),
                "application/vnd.amazon.ebook"
                    | "application/vnd.amazon.mobi8-ebook"
                    | "application/x-mobipocket-ebook"
                    | "application/x-mobi8-ebook"
                    | "text/plain"
            ),
            _ => true,
        }
    }
}

impl std::fmt::Display for DeviceClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Classifies a `User-Agent` header value.
///
/// E-ink tokens are tested first, because e-reader browsers also say `Android`, `Mobile`
/// and `Safari`. The rules, in order:
///
/// | Token (case-insensitive unless noted) | Class |
/// |---|---|
/// | `Kindle Fire`, `Silk/`, ` KF…` Fire model codes | `mobile` (tablets, not e-ink) |
/// | `Kindle/` | `kindle` |
/// | `Kobo` | `kobo` |
/// | `tolino` | `tolino` |
/// | `PocketBook`, `Vivlio` | `pocketbook` |
/// | `eReader`, `E-Reader`, `NOOK`, `Bookeen`, `Cybook`, `Sony Reader`/`EBRD`/`PRS-T`, `Onyx`/`BOOX`, `E Ink`/`eink`, `reMarkable` | `eink_other` |
/// | `X11; Linux armv…` without `Android` (2024+ Kindles and other Linux e-readers posing as desktop Chrome) | `eink_other` |
/// | `iPhone`, `iPad`, `iPod`, `Android`, `Mobi`, `Windows Phone`, `Opera Mini`, `BlackBerry` | `mobile` |
/// | anything else, including an empty or absent header | `desktop` |
///
/// The function never panics, whatever the input.
#[must_use]
pub fn classify(user_agent: &str) -> DeviceClass {
    // Bound the work on hostile headers; real user agents are a few hundred bytes.
    let ua: String = user_agent.chars().take(1024).collect::<String>().to_ascii_lowercase();
    let has = |token: &str| ua.contains(token);

    // Amazon Fire tablets: colour Android tablets whose agent may contain "Kindle".
    let fire_model = ua.split(|c: char| !c.is_ascii_alphanumeric()).any(|word| {
        word.len() >= 4
            && word.len() <= 7
            && word.starts_with("kf")
            && word[2..].bytes().all(|b| b.is_ascii_alphabetic())
    });
    if has("kindle fire") || has("silk/") || (fire_model && has("android")) {
        return DeviceClass::Mobile;
    }
    if has("kindle/") {
        return DeviceClass::Kindle;
    }
    if has("kobo") {
        return DeviceClass::Kobo;
    }
    if has("tolino") {
        return DeviceClass::Tolino;
    }
    if has("pocketbook") || has("vivlio") {
        return DeviceClass::Pocketbook;
    }
    let other_eink = [
        "ereader",
        "e-reader",
        "nook",
        "bookeen",
        "cybook",
        "sony reader",
        "ebrd",
        "prs-t",
        "onyx",
        "boox",
        "e ink",
        "eink",
        "remarkable",
    ];
    if other_eink.iter().any(|token| has(token)) {
        return DeviceClass::EinkOther;
    }
    if has("x11; linux armv") && !has("android") {
        return DeviceClass::EinkOther;
    }
    let mobile =
        ["iphone", "ipad", "ipod", "android", "mobi", "windows phone", "opera mini", "blackberry"];
    if mobile.iter().any(|token| has(token)) {
        return DeviceClass::Mobile;
    }
    DeviceClass::Desktop
}

#[cfg(test)]
mod tests {
    use super::DeviceClass::{Desktop, EinkOther, Kindle, Kobo, Mobile, Pocketbook, Tolino};
    use super::*;

    /// Real-world user agents. Sources: the strings quoted in
    /// `docs/research/notes/ereader-ux.{devices,ux}.md` (uap-core and Matomo fixtures,
    /// send2ereader issues), the profiles of `demo/ereaders`, and widely published
    /// browser strings. None was captured on hardware by this project.
    const TABLE: &[(&str, DeviceClass)] = &[
        // --- Kobo ---------------------------------------------------------------------
        (
            "Mozilla/5.0 (Linux; U; Android 2.0; en-us;) AppleWebKit/533.1 (KHTML, like Gecko) Version/4.0 Mobile Safari/533.1 (Kobo Touch)",
            Kobo,
        ),
        (
            "Mozilla/5.0 (Linux; U; Android 2.0; en-us;) AppleWebKit/538.1 (KHTML, like Gecko) Version/4.0 Mobile Safari/538.1 (Kobo Touch 0390/4.38.23038)",
            Kobo,
        ),
        (
            "Mozilla/5.0 (Linux; U; Android 2.0; en-us;) AppleWebKit/538.1 (KHTML, like Gecko) Version/4.0 Mobile Safari/538.1 (Kobo Touch 0384/4.35.20400)",
            Kobo,
        ),
        // Kobo Libra Colour / Kobo-based tolino since 2024: only "eReader".
        (
            "Mozilla/5.0 (Linux; U; Android 2.0; en-us;) AppleWebKit/538.1 (KHTML, like Gecko) Version/4.0 Mobile Safari/538.1 (eReader)",
            EinkOther,
        ),
        (
            "Mozilla/5.0 (Linux; U; en-us) AppleWebKit/534.34 (KHTML, like Gecko) Version/4.0 Safari/534.34 eReader",
            EinkOther,
        ),
        // --- Kindle (e-ink) -----------------------------------------------------------
        (
            "Mozilla/5.0 (Linux; U; en-US) AppleWebKit/528.5+ (KHTML, like Gecko, Safari/528.5+) Version/4.0 Kindle/3.0 (screen 600x800; rotate)",
            Kindle,
        ),
        (
            "Mozilla/5.0 (X11; U; Linux armv7l like Android; en-us) AppleWebKit/531.2+ (KHTML, like Gecko) Version/5.0 Safari/533.2+ Kindle/3.0+",
            Kindle,
        ),
        (
            "Mozilla/4.0 (compatible; Linux 2.6.22) NetFront/3.4 Kindle/2.5 (screen 824x1200; rotate)",
            Kindle,
        ),
        ("Mozilla/4.0 (compatible; Linux 2.6.10) NetFront/3.3 Kindle/1.0 (screen 600x800)", Kindle),
        // Kindle 11th gen (2024), firmware 5.18.x: no "Kindle" token at all.
        (
            "Mozilla/5.0 (X11; Linux armv7l) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/75.0.3770.142 Safari/537.36",
            EinkOther,
        ),
        // --- Kindle Fire tablets are tablets ----------------------------------------
        (
            "Mozilla/5.0 (Linux; U; Android 2.3.4; en-us; Kindle Fire Build/GINGERBREAD) AppleWebKit/533.1 (KHTML, like Gecko) Version/4.0 Mobile Safari/533.1",
            Mobile,
        ),
        (
            "Mozilla/5.0 (Linux; U; Android 4.0.3; en-us; KFTT Build/IML74K) AppleWebKit/535.19 (KHTML, like Gecko) Silk/3.4 Mobile Safari/535.19 Silk-Accelerated=true",
            Mobile,
        ),
        (
            "Mozilla/5.0 (Linux; Android 9; KFMAWI) AppleWebKit/537.36 (KHTML, like Gecko) Silk/108.4.1 like Chrome/108.0.5359.220 Safari/537.36",
            Mobile,
        ),
        (
            "Mozilla/5.0 (Linux; Android 5.1.1; KFFOWI Build/LVY48F) AppleWebKit/537.36 (KHTML, like Gecko) Version/4.0 Chrome/59.0.3071.125 Safari/537.36",
            Mobile,
        ),
        // --- PocketBook / Vivlio ----------------------------------------------------
        (
            "Mozilla/5.0 (Linux; U; Android 2.3.4; en-us; PocketBook/629 Build/GRJ22) AppleWebKit/533.1 (KHTML, like Gecko) Version/4.0 Mobile Safari/533.1",
            Pocketbook,
        ),
        (
            "Mozilla/5.0 (Linux; U; en-us) AppleWebKit/534.34 (KHTML, like Gecko) Version/4.0 Safari/534.34 PocketBook/623 (screen 758x1024; FW T623.4.4.616) Mobile",
            Pocketbook,
        ),
        (
            "Mozilla/5.0 (X11; U; Linux armv7l; en-US) AppleWebKit/534.34 (KHTML, like Gecko) PocketBook/740 (screen 1404x1872; FW U740.6.8.2154) Version/4.0 Safari/534.34",
            Pocketbook,
        ),
        // --- tolino -------------------------------------------------------------------
        (
            "Mozilla/5.0 (Linux; Android 4.4.2; tolino vision 4 HD Build/KOT49H) AppleWebKit/537.36 (KHTML, like Gecko) Version/4.0 Chrome/30.0.0.0 Safari/537.36",
            Tolino,
        ),
        (
            "Mozilla/5.0 (Linux; U; Android 4.0.4; de-de; tolino shine Build/IMM76D) AppleWebKit/534.30 (KHTML, like Gecko) Version/4.0 Safari/534.30",
            Tolino,
        ),
        (
            "Mozilla/5.0 (Linux; U; Android 2.3.4; de-de; TOLINO_BROWSER Build/GRJ22) AppleWebKit/533.1 (KHTML, like Gecko) Version/4.0 Mobile Safari/533.1",
            Tolino,
        ),
        // --- other e-ink --------------------------------------------------------------
        (
            "Mozilla/5.0 (Linux; U; Android 2.2.1; en-us; NOOK BNRV200 Build/ERD79 1.4.3) Apple WebKit/533.1 (KHTML, like Gecko) Version/4.0 Mobile Safari/533.1",
            EinkOther,
        ),
        (
            "Mozilla/5.0 (Linux; U; en-us; EBRD1101; EXT) AppleWebKit/533.1 (KHTML, like Gecko) Version/4.0 Mobile Safari/533.1",
            EinkOther,
        ),
        (
            "Mozilla/5.0 (Linux; U; Android 2.2; fr-fr; Cybook Odyssey Build/FRF85B) AppleWebKit/533.1 (KHTML, like Gecko) Version/4.0 Mobile Safari/533.1",
            EinkOther,
        ),
        (
            "Mozilla/5.0 (Linux; Android 11; BOOX NoteAir2P) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.0.4664.104 Safari/537.36",
            EinkOther,
        ),
        // --- phones and tablets -----------------------------------------------------
        (
            "Mozilla/5.0 (iPhone; CPU iPhone OS 18_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.1 Mobile/15E148 Safari/604.1",
            Mobile,
        ),
        (
            "Mozilla/5.0 (iPad; CPU OS 16_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.6 Mobile/15E148 Safari/604.1",
            Mobile,
        ),
        (
            "Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Mobile Safari/537.36",
            Mobile,
        ),
        (
            "Mozilla/5.0 (Linux; Android 13; SM-X200) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36",
            Mobile,
        ),
        ("Mozilla/5.0 (Android 14; Mobile; rv:132.0) Gecko/132.0 Firefox/132.0", Mobile),
        (
            "Mozilla/5.0 (Windows Phone 10.0; Android 6.0.1; Microsoft; Lumia 950) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/52.0.2743.116 Mobile Safari/537.36 Edge/15.15254",
            Mobile,
        ),
        (
            "Opera/9.80 (J2ME/MIDP; Opera Mini/9.80 (S60; SymbOS; Opera Mobi/23.348; U; en) Presto/2.5.25 Version/10.54",
            Mobile,
        ),
        // --- desktop ------------------------------------------------------------------
        (
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36",
            Desktop,
        ),
        (
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.1 Safari/605.1.15",
            Desktop,
        ),
        ("Mozilla/5.0 (X11; Linux x86_64; rv:132.0) Gecko/20100101 Firefox/132.0", Desktop),
        (
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Thorium/3.1.0 Chrome/130.0.0.0 Electron/33.0.0 Safari/537.36",
            Desktop,
        ),
        (
            "Mozilla/5.0 (X11; CrOS x86_64 14541.0.0) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36",
            Desktop,
        ),
        ("Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)", Desktop),
        ("curl/8.5.0", Desktop),
        ("ebc-demo-stores/0.1.0", Desktop),
        ("", Desktop),
    ];

    #[test]
    fn real_world_user_agents_are_classified() {
        for (ua, expected) in TABLE {
            assert_eq!(classify(ua), *expected, "{ua}");
        }
    }

    #[test]
    fn every_class_is_reachable_by_detection() {
        for class in DeviceClass::ALL {
            assert!(TABLE.iter().any(|(_, c)| *c == class), "no sample for {class}");
        }
    }

    #[test]
    fn detection_ignores_case_and_survives_hostile_input() {
        assert_eq!(classify("KOBO TOUCH"), Kobo);
        assert_eq!(classify("something kindle/5.0"), Kindle);
        assert_eq!(classify("EREADER"), EinkOther);
        assert_eq!(classify(&"A".repeat(1_000_000)), Desktop);
        assert_eq!(classify(&"é".repeat(5_000)), Desktop);
        assert_eq!(classify("\u{0}\u{1}\u{7f}<script>alert(1)</script>"), Desktop);
        // The token is only looked for in the first 1024 characters.
        assert_eq!(classify(&format!("{}Kobo", " ".repeat(2_000))), Desktop);
        // "Kindle" alone on an Android agent (the reading app's web view) is not the e-ink device.
        assert_eq!(classify("Mozilla/5.0 (Linux; Android 14; Pixel 8) Kindle App"), Mobile);
        // A desktop mentioning the word in passing stays what it is.
        assert_eq!(
            classify(
                "Mozilla/5.0 (Windows NT 10.0) Chrome/130 Safari/537.36 SendToKindleExtension"
            ),
            Desktop
        );
    }

    #[test]
    fn choices_round_trip_and_reject_unknown_values() {
        for class in DeviceClass::ALL {
            match class.choice() {
                Some(choice) => assert_eq!(DeviceClass::from_choice(choice), Some(class)),
                None => assert_eq!(class, EinkOther),
            }
        }
        assert_eq!(DeviceClass::from_choice("PHONE"), Some(Mobile));
        assert_eq!(DeviceClass::from_choice(" computer "), Some(Desktop));
        assert_eq!(DeviceClass::from_choice("mobile"), Some(Mobile));
        assert_eq!(DeviceClass::from_choice("desktop"), Some(Desktop));
        for unknown in ["", "eink_other", "toaster", "kobo2", "<script>", "kobo,kindle"] {
            assert_eq!(DeviceClass::from_choice(unknown), None, "{unknown:?}");
        }
        let names: Vec<&str> = DeviceClass::ALL.iter().map(|c| c.as_str()).collect();
        assert_eq!(
            names,
            ["kobo", "kindle", "pocketbook", "tolino", "eink_other", "mobile", "desktop"]
        );
        assert_eq!(Kobo.to_string(), "kobo");
    }

    #[test]
    fn kindle_browsers_only_save_kindle_formats() {
        assert!(!Kindle.browser_saves("application/epub+zip"));
        assert!(!Kindle.browser_saves("application/pdf"));
        assert!(Kindle.browser_saves("application/vnd.amazon.mobi8-ebook"));
        assert!(Kindle.browser_saves("application/x-mobipocket-ebook"));
        assert!(Kindle.browser_saves("TEXT/PLAIN"));
        for class in [Kobo, Pocketbook, Tolino, EinkOther, Mobile, Desktop] {
            assert!(class.browser_saves("application/epub+zip"), "{class}");
            assert!(class.browser_saves("application/pdf"), "{class}");
        }
        assert!(Kobo.is_eink() && Kindle.is_eink() && EinkOther.is_eink());
        assert!(!Mobile.is_eink() && !Desktop.is_eink());
    }
}
