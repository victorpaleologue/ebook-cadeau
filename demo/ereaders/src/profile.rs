//! Device profiles: what each e-reader family can and cannot do.
//!
//! Every capability encoded here carries a [`Claim`] that either cites the research notes or is
//! flagged as an assumption. The citations refer to
//! `docs/research/notes/ereader-ux.devices.md` (abbreviated **devices**) and
//! `docs/research/notes/ereader-ux.ecosystems.md` (**ecosystems**); grades are the notes' own:
//! **V2** two independent fetched sources, **V1** one fetched source, **M** the researcher's
//! memory only (unverified). The consolidated `docs/research/05-ereader-distribution-ux.md` did
//! not exist when this was written.
//!
//! Nothing here was measured on hardware. A profile is a *hypothesis about a device* that makes
//! the integration tests honest about constraints (no JavaScript, no EPUB on Kindle, typed
//! codes); it is not a certification.

use crate::format::FileFormat;
use crate::typing::Keyboard;

/// E-reader family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum DeviceFamily {
    /// Rakuten Kobo (also sold as "Kobo by Fnac").
    Kobo,
    /// Amazon Kindle.
    Kindle,
    /// PocketBook.
    PocketBook,
    /// Vivlio (PocketBook hardware, French booksellers' ecosystem).
    Vivlio,
    /// tolino.
    Tolino,
    /// Any other e-ink device with a basic browser.
    GenericEink,
    /// A reading application on a phone or computer (used for the OPDS / LCP flows).
    ReadingApp,
    /// A smartphone's web browser: the device that scans the card's QR code, asks for a
    /// pair code, uses the store's activation page or sends the book to a Kindle.
    Phone,
    /// A computer's web browser: where gifts are bought, and one of the device classes of
    /// the gift page.
    Computer,
}

impl DeviceFamily {
    /// The `?device=` value that selects this family on a gift page (`docs/api.md` §2,
    /// "Device classes"): `kobo`, `kindle`, `pocketbook`, `tolino`, `phone`, `computer`.
    /// `None` for families the chooser does not list.
    pub const fn device_choice(self) -> Option<&'static str> {
        match self {
            Self::Kobo => Some("kobo"),
            Self::Kindle => Some("kindle"),
            Self::PocketBook | Self::Vivlio => Some("pocketbook"),
            Self::Tolino => Some("tolino"),
            Self::Phone => Some("phone"),
            Self::Computer => Some("computer"),
            Self::GenericEink | Self::ReadingApp => None,
        }
    }

    /// The id of this family's link in the gift page's device chooser (`device-kobo`,
    /// `device-kindle`, `device-pocketbook`, `device-tolino`, `device-phone`,
    /// `device-computer`), i.e. what a person taps when the page guessed their device
    /// wrong.
    pub const fn chooser_link_id(self) -> Option<&'static str> {
        match self {
            Self::Kobo => Some("device-kobo"),
            Self::Kindle => Some("device-kindle"),
            Self::PocketBook | Self::Vivlio => Some("device-pocketbook"),
            Self::Tolino => Some("device-tolino"),
            Self::Phone => Some("device-phone"),
            Self::Computer => Some("device-computer"),
            Self::GenericEink | Self::ReadingApp => None,
        }
    }
}

/// DRM schemes a reading system may or may not open.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
#[non_exhaustive]
pub enum DrmScheme {
    /// Adobe ADEPT (`.acsm` fulfilment, Adobe ID).
    AdobeAdept,
    /// Readium LCP (licence + passphrase).
    ReadiumLcp,
    /// Kobo's account-bound DRM.
    Kobo,
    /// Amazon's account-bound DRM.
    Amazon,
    /// Encrypted resources of an unidentified scheme.
    Unknown,
}

/// Rough screen size, used by the page lint for fixed-width layouts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenClass {
    /// ~6 inch (Kobo Clara, Kindle, PocketBook Verse).
    Small,
    /// ~7–8 inch (Libra, Paperwhite 7", Era, InkPad).
    Medium,
    /// 10 inch and more, or a computer window.
    Large,
}

impl ScreenClass {
    /// Narrowest CSS viewport width a page should assume for this class.
    ///
    /// *Assumption*: the research could not establish the CSS viewport of any e-reader browser;
    /// these are conservative round numbers.
    pub fn css_width_px(self) -> u32 {
        match self {
            Self::Small => 480,
            Self::Medium => 600,
            Self::Large => 800,
        }
    }
}

/// Behaviours that differ from a textbook browser.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Quirks {
    /// The browser only saves files whose name is plain ASCII (letters, digits, `. _ - ' ( )`).
    pub ascii_download_names: bool,
    /// The browser is unreliable on ports other than 80/443. Only enforced when
    /// [`crate::browser::BrowserConfig::enforce_port_quirk`] is set, because tests bind
    /// ephemeral ports.
    pub standard_ports_only: bool,
    /// The download completes but the reading system reports the file as damaged.
    pub download_reported_damaged: bool,
    /// A note attached to every browser download (e.g. "not added to the Kindle library").
    pub download_note: Option<&'static str>,
    /// The engine predates the `Referrer-Policy` response header and sends `Referer`
    /// regardless of it.
    pub ignores_referrer_policy: bool,
}

/// File-name rules of a mail-to-device service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FileNameRules {
    /// Attachment names must be ASCII.
    pub ascii_only: bool,
    /// Attachment names must not contain commas.
    pub no_commas: bool,
    /// Longest accepted attachment name, in bytes.
    pub max_len: Option<usize>,
}

/// A "send to my device by e-mail" service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MailServiceSpec {
    /// Stable id (`send-to-kindle`, `send-to-pocketbook`).
    pub id: &'static str,
    /// Display name.
    pub name: &'static str,
    /// Domains of device addresses.
    pub domains: &'static [&'static str],
    /// Attachment formats the service accepts.
    pub accepted: &'static [FileFormat],
    /// If set, accepted EPUBs are converted to this format before delivery (no real
    /// conversion happens in the simulation; only the label changes).
    pub converts_epub_to: Option<FileFormat>,
    /// Largest message accepted at SMTP level.
    pub max_message_bytes: u64,
    /// Attachment file-name rules.
    pub file_names: FileNameRules,
    /// Whether the account owner must additionally click a confirmation e-mail before each
    /// delivery (off in the presets; see [`MailServiceSpec::send_to_kindle`]).
    pub confirmation_click: bool,
}

impl MailServiceSpec {
    /// Amazon's Send to Kindle by e-mail.
    ///
    /// * EPUB is accepted and converted — devices §3.1 \[V2\]; AZW3 is refused — §3.1 \[V2\];
    ///   MOBI by e-mail was dropped in 2022 — §3.1 \[V1\]. PDF and TXT acceptance is \[M\].
    /// * The sender must be on the account's approved list — §3.1 \[V1\].
    /// * 50 MB e-mail limit — §3.1 \[M\].
    /// * Amazon "recently started sending confirmation emails that you have to click on"
    ///   (§3.1 \[V1\]); model it by setting `confirmation_click = true`. It is off by default
    ///   because the note does not say when it applies.
    pub const fn send_to_kindle() -> Self {
        Self {
            id: "send-to-kindle",
            name: "Send to Kindle",
            domains: &["kindle.com", "free.kindle.com"],
            accepted: &[FileFormat::Epub, FileFormat::Pdf, FileFormat::Txt],
            converts_epub_to: Some(FileFormat::Kfx),
            max_message_bytes: 50 * 1024 * 1024,
            file_names: FileNameRules { ascii_only: false, no_commas: false, max_len: None },
            confirmation_click: false,
        }
    }

    /// PocketBook's Send-to-PocketBook (`@pbsync.com`).
    ///
    /// * The service and its domain — devices §3.4 \[V1\] (calibre special-cases it).
    /// * It "can't handle non-ascii, filenames that are long…, commas" — §3.4 \[V1\]; the
    ///   64-byte bound for "long" is an *assumption*.
    /// * Accepted formats are taken to be the device's reading formats, and the 20 MB limit is
    ///   a placeholder — both *assumptions*. Sender confirmation on first use is \[M\] and is
    ///   modelled by the approved-senders list.
    pub const fn send_to_pocketbook() -> Self {
        Self {
            id: "send-to-pocketbook",
            name: "Send-to-PocketBook",
            domains: &["pbsync.com"],
            accepted: &[
                FileFormat::Epub,
                FileFormat::Pdf,
                FileFormat::Fb2,
                FileFormat::Mobi,
                FileFormat::Txt,
            ],
            converts_epub_to: None,
            max_message_bytes: 20 * 1024 * 1024,
            file_names: FileNameRules { ascii_only: true, no_commas: true, max_len: Some(64) },
            confirmation_click: false,
        }
    }
}

/// The capability a [`Claim`] is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Capability {
    /// The `User-Agent` string.
    UserAgent,
    /// Cookie support.
    Cookies,
    /// Page weight tolerated.
    PageWeight,
    /// JavaScript support of the real browser (the simulator never runs any).
    JavaScript,
    /// File types the browser saves.
    BrowserDownloads,
    /// File types the reading system opens.
    ReadingFormats,
    /// DRM schemes.
    Drm,
    /// Mail-to-device service.
    MailService,
    /// OPDS support.
    Opds,
    /// Screen class.
    Screen,
    /// On-screen keyboard and tap counts.
    Keyboard,
    /// Quirks.
    Quirks,
}

impl Capability {
    /// Every capability a profile must document.
    pub const ALL: [Self; 12] = [
        Self::UserAgent,
        Self::Cookies,
        Self::PageWeight,
        Self::JavaScript,
        Self::BrowserDownloads,
        Self::ReadingFormats,
        Self::Drm,
        Self::MailService,
        Self::Opds,
        Self::Screen,
        Self::Keyboard,
        Self::Quirks,
    ];
}

/// Confidence grade used by the research notes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Grade {
    /// The researcher's memory only — unverified.
    M,
    /// One fetched source.
    V1,
    /// Two independent fetched sources.
    V2,
}

/// Where a capability claim comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Basis {
    /// Stated in the research notes.
    Research {
        /// Note and section, e.g. `devices §3.1`.
        section: &'static str,
        /// The note's confidence grade for that statement.
        grade: Grade,
    },
    /// Not in the research: a modelling choice, with its rationale.
    Assumption(&'static str),
}

/// One documented capability statement of a profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Claim {
    /// What it is about.
    pub capability: Capability,
    /// The statement.
    pub statement: &'static str,
    /// Its basis.
    pub basis: Basis,
}

const fn research(
    capability: Capability,
    statement: &'static str,
    section: &'static str,
    grade: Grade,
) -> Claim {
    Claim { capability, statement, basis: Basis::Research { section, grade } }
}

const fn assumption(capability: Capability, statement: &'static str, why: &'static str) -> Claim {
    Claim { capability, statement, basis: Basis::Assumption(why) }
}

const PAGE_CAP_WHY: &str = "no vendor limit is known; 1 MiB is far above the 30 kB page budget of docs/api.md §2 and bounds hostile input";
const COOKIES_WHY: &str = "WebKit-class browsers keep cookies by default; not verified per device";
const KEYBOARD_WHY: &str = "no vendor documents its on-screen layout; see typing::Keyboard";
const UA_WHY: &str = "exact string recalled from memory; the research only sources substrings";
const REFERRER_WHY: &str = "the Referrer-Policy header shipped in browsers from 2017; these engines are older, so they are assumed to send Referer whatever the page says (not verified on hardware)";
const REFERRER_CLAIM: Claim = assumption(
    Capability::Quirks,
    "ignores Referrer-Policy: a gift URL leaks through Referer to any other origin the page links to",
    REFERRER_WHY,
);

/// A simulated device (or reading application).
///
/// All fields are public so a test can derive a variant:
/// `DeviceProfile { cookies: false, ..DeviceProfile::kobo_clara() }`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceProfile {
    /// Stable identifier, e.g. `kobo-clara`.
    pub id: &'static str,
    /// Human-readable name.
    pub name: &'static str,
    /// Family.
    pub family: DeviceFamily,
    /// `User-Agent` sent with every request.
    pub user_agent: &'static str,
    /// Whether the browser keeps cookies.
    pub cookies: bool,
    /// Heaviest HTML page the browser is assumed to cope with (hard cap; the *product* budget
    /// is in [`crate::lint::LintConfig`]).
    pub max_page_bytes: usize,
    /// Largest file the browser will download.
    pub max_download_bytes: u64,
    /// File types the built-in browser saves.
    pub browser_downloads: &'static [FileFormat],
    /// File types the reading system opens.
    pub reading_formats: &'static [FileFormat],
    /// DRM schemes the reading system can unlock *over the air* (without a computer).
    pub drm: &'static [DrmScheme],
    /// Mail-to-device service of the ecosystem, if any.
    pub mail_service: Option<MailServiceSpec>,
    /// Whether the stock firmware has an OPDS client.
    pub native_opds: bool,
    /// Screen class.
    pub screen: ScreenClass,
    /// On-screen keyboard.
    pub keyboard: Keyboard,
    /// Taps from the home screen to an open browser with the address bar focused.
    pub browser_launch_taps: u32,
    /// Taps to confirm a download dialog.
    pub download_confirm_taps: u32,
    /// Quirks.
    pub quirks: Quirks,
    /// Evidence for all of the above.
    pub claims: &'static [Claim],
}

impl DeviceProfile {
    /// Kobo Clara-class device (Clara BW / Colour) with the Nickel browser.
    pub fn kobo_clara() -> Self {
        use Capability as C;
        const CLAIMS: &[Claim] = &[
            research(C::UserAgent, "the UA contains `Kobo Touch`", "devices §3.2", Grade::V1),
            assumption(C::UserAgent, "full UA string", UA_WHY),
            assumption(C::Cookies, "cookies are kept", COOKIES_WHY),
            assumption(C::PageWeight, "pages up to 1 MiB", PAGE_CAP_WHY),
            research(
                C::JavaScript,
                "the Nickel browser (QtWebKit fork) runs ES5 + XMLHttpRequest; modern syntax is not assumed",
                "devices §3.2",
                Grade::V1,
            ),
            research(
                C::BrowserDownloads,
                "the browser saves EPUB/KEPUB/PDF to the library",
                "devices §2",
                Grade::V2,
            ),
            research(
                C::ReadingFormats,
                "EPUB 2/3, KEPUB, PDF, TXT, CBZ are native",
                "devices §2",
                Grade::V1,
            ),
            research(C::ReadingFormats, "MOBI is native", "devices §2", Grade::M),
            research(
                C::Drm,
                "Kobo DRM arrives by store sync only; Adobe needs a computer; no LCP — so nothing protected can be opened from a direct download",
                "devices §2",
                Grade::M,
            ),
            research(
                C::MailService,
                "no mail-to-device service (cloud options are Dropbox/Google Drive on some models)",
                "devices §2",
                Grade::V1,
            ),
            research(C::Opds, "no native OPDS client", "devices §2", Grade::M),
            research(C::Screen, "6-inch class", "devices §2", Grade::M),
            assumption(
                C::Keyboard,
                "three-layer keyboard; 4 taps to reach the beta browser (More → Settings → Beta features → Web browser); 1 tap to confirm a download",
                KEYBOARD_WHY,
            ),
            research(
                C::Quirks,
                "the browser has problems on non-standard ports",
                "devices §3.2",
                Grade::V1,
            ),
            research(
                C::Quirks,
                "`.kepub.epub` selects the Kobo WebKit renderer, plain `.epub` the Adobe one",
                "devices §3.2",
                Grade::V2,
            ),
            REFERRER_CLAIM,
        ];
        Self {
            id: "kobo-clara",
            name: "Kobo Clara (Nickel browser)",
            family: DeviceFamily::Kobo,
            user_agent: "Mozilla/5.0 (Linux; U; Android 2.0; en-us;) AppleWebKit/538.1 (KHTML, like Gecko) Version/4.0 Mobile Safari/538.1 (Kobo Touch 0390/4.38.23038)",
            cookies: true,
            max_page_bytes: 1024 * 1024,
            max_download_bytes: 200 * 1024 * 1024,
            browser_downloads: &[FileFormat::Epub, FileFormat::Pdf],
            reading_formats: &[
                FileFormat::Epub,
                FileFormat::Pdf,
                FileFormat::Txt,
                FileFormat::Cbz,
                FileFormat::Mobi,
            ],
            drm: &[],
            mail_service: None,
            native_opds: false,
            screen: ScreenClass::Small,
            keyboard: Keyboard::three_layer(),
            browser_launch_taps: 4,
            download_confirm_taps: 1,
            quirks: Quirks {
                standard_ports_only: true,
                ignores_referrer_policy: true,
                ..Quirks::default()
            },
            claims: CLAIMS,
        }
    }

    /// A Kobo on recent firmware whose UA says `eReader` instead of `Kobo`
    /// (reported on a Libra Colour, 2026-08) — the case that breaks UA sniffing.
    pub fn kobo_ereader_ua() -> Self {
        use Capability as C;
        const CLAIMS: &[Claim] = &[
            research(
                C::UserAgent,
                "newer Kobo firmware reports a generic `eReader` and no longer `Kobo`",
                "devices §3.2",
                Grade::V1,
            ),
            assumption(C::UserAgent, "full UA string", UA_WHY),
            assumption(C::Cookies, "cookies are kept", COOKIES_WHY),
            assumption(C::PageWeight, "pages up to 1 MiB", PAGE_CAP_WHY),
            research(C::JavaScript, "as kobo-clara", "devices §3.2", Grade::V1),
            research(C::BrowserDownloads, "as kobo-clara", "devices §2", Grade::V2),
            research(C::ReadingFormats, "as kobo-clara", "devices §2", Grade::V1),
            research(C::Drm, "as kobo-clara", "devices §2", Grade::M),
            research(C::MailService, "as kobo-clara", "devices §2", Grade::V1),
            research(C::Opds, "as kobo-clara", "devices §2", Grade::M),
            research(C::Screen, "7-inch class (Libra Colour)", "devices §2", Grade::M),
            assumption(C::Keyboard, "as kobo-clara", KEYBOARD_WHY),
            research(C::Quirks, "as kobo-clara", "devices §3.2", Grade::V1),
            REFERRER_CLAIM,
        ];
        Self {
            id: "kobo-ereader-ua",
            name: "Kobo Libra Colour (firmware reporting \"eReader\")",
            user_agent: "Mozilla/5.0 (Linux; U; Android 2.0; en-us;) AppleWebKit/538.1 (KHTML, like Gecko) Version/4.0 Mobile Safari/538.1 (eReader)",
            screen: ScreenClass::Medium,
            claims: CLAIMS,
            ..Self::kobo_clara()
        }
    }

    /// Kindle Paperwhite-class device with the "experimental" web browser.
    pub fn kindle_paperwhite() -> Self {
        use Capability as C;
        const CLAIMS: &[Claim] = &[
            research(
                C::UserAgent,
                "classic Kindle browsers identify as `Kindle/3`",
                "devices §3.1",
                Grade::V1,
            ),
            assumption(C::UserAgent, "full UA string", UA_WHY),
            assumption(C::Cookies, "cookies are kept", COOKIES_WHY),
            assumption(C::PageWeight, "pages up to 1 MiB", PAGE_CAP_WHY),
            assumption(
                C::JavaScript,
                "an old WebKit; ES5 at best",
                "engine level could not be verified (devices, 'Could not verify')",
            ),
            research(
                C::BrowserDownloads,
                "the browser downloads AZW3 and MOBI",
                "devices §2",
                Grade::V2,
            ),
            research(
                C::BrowserDownloads,
                "the browser also downloads TXT; other types are refused",
                "devices §2",
                Grade::M,
            ),
            research(
                C::BrowserDownloads,
                "EPUB is not downloadable; PDF download is reported failing",
                "devices §2",
                Grade::V1,
            ),
            research(
                C::ReadingFormats,
                "KFX, AZW3, AZW, MOBI, PDF, TXT are native; no EPUB",
                "devices §3.1",
                Grade::V2,
            ),
            research(C::Drm, "Amazon DRM only, by account sync", "devices §2", Grade::M),
            research(
                C::MailService,
                "Send to Kindle by e-mail: approved senders, EPUB converted, AZW3/MOBI refused",
                "devices §3.1",
                Grade::V2,
            ),
            research(
                C::Opds,
                "no OPDS (KOReader on jailbroken units only)",
                "devices §2",
                Grade::V1,
            ),
            research(C::Screen, "6 to 7-inch class", "devices §2", Grade::M),
            research(
                C::Keyboard,
                "the browser is reached through the ⋮ menu → Web Browser (2 taps); a download asks for an OK (1 tap)",
                "devices §3.1",
                Grade::V1,
            ),
            assumption(C::Keyboard, "three-layer keyboard", KEYBOARD_WHY),
            research(
                C::Quirks,
                "download file names must be plain ASCII",
                "devices §3.1",
                Grade::V1,
            ),
            research(
                C::Quirks,
                "a browser download is not added to the Kindle library proper and shows no cover",
                "devices §3.1",
                Grade::V1,
            ),
            REFERRER_CLAIM,
        ];
        Self {
            id: "kindle-paperwhite",
            name: "Kindle Paperwhite (experimental browser)",
            family: DeviceFamily::Kindle,
            user_agent: "Mozilla/5.0 (X11; U; Linux armv7l like Android; en-us) AppleWebKit/531.2+ (KHTML, like Gecko) Version/5.0 Safari/533.2+ Kindle/3.0+",
            cookies: true,
            max_page_bytes: 1024 * 1024,
            max_download_bytes: 200 * 1024 * 1024,
            browser_downloads: &[FileFormat::Azw3, FileFormat::Mobi, FileFormat::Txt],
            reading_formats: &[
                FileFormat::Kfx,
                FileFormat::Azw3,
                FileFormat::Mobi,
                FileFormat::Pdf,
                FileFormat::Txt,
            ],
            drm: &[],
            mail_service: Some(MailServiceSpec::send_to_kindle()),
            native_opds: false,
            screen: ScreenClass::Small,
            keyboard: Keyboard::three_layer(),
            browser_launch_taps: 2,
            download_confirm_taps: 1,
            quirks: Quirks {
                ascii_download_names: true,
                download_note: Some(
                    "opened from a browser download: not in the Kindle library, no cover",
                ),
                ignores_referrer_policy: true,
                ..Quirks::default()
            },
            claims: CLAIMS,
        }
    }

    /// A 2024 Kindle (firmware 5.18.6) whose UA no longer contains `Kindle` — servers see a
    /// desktop browser.
    pub fn kindle_2024() -> Self {
        use Capability as C;
        const CLAIMS: &[Claim] = &[
            research(
                C::UserAgent,
                "a 2024 Kindle on firmware 5.18.6 is no longer recognised as `Kindle`",
                "devices §3.1",
                Grade::V1,
            ),
            assumption(
                C::UserAgent,
                "full UA string (a Chromium-style UA without any Kindle token)",
                UA_WHY,
            ),
            assumption(C::Cookies, "cookies are kept", COOKIES_WHY),
            assumption(C::PageWeight, "pages up to 1 MiB", PAGE_CAP_WHY),
            research(
                C::JavaScript,
                "engine reportedly Chromium-based since firmware 5.16.4",
                "devices §2",
                Grade::M,
            ),
            research(C::BrowserDownloads, "as kindle-paperwhite", "devices §2", Grade::V2),
            research(C::ReadingFormats, "as kindle-paperwhite", "devices §3.1", Grade::V2),
            research(C::Drm, "as kindle-paperwhite", "devices §2", Grade::M),
            research(C::MailService, "as kindle-paperwhite", "devices §3.1", Grade::V2),
            research(C::Opds, "as kindle-paperwhite", "devices §2", Grade::V1),
            research(C::Screen, "6-inch class", "devices §2", Grade::M),
            research(C::Keyboard, "as kindle-paperwhite", "devices §3.1", Grade::V1),
            research(C::Quirks, "as kindle-paperwhite", "devices §3.1", Grade::V1),
            assumption(
                C::Quirks,
                "honours Referrer-Policy",
                "follows from the reportedly Chromium-based engine (devices §2 [M])",
            ),
        ];
        Self {
            id: "kindle-2024",
            name: "Kindle 2024 (UA without \"Kindle\")",
            user_agent: "Mozilla/5.0 (X11; Linux armv7l) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/75.0.3770.142 Safari/537.36",
            quirks: Quirks { ignores_referrer_policy: false, ..Self::kindle_paperwhite().quirks },
            claims: CLAIMS,
            ..Self::kindle_paperwhite()
        }
    }

    /// PocketBook e-ink device (Verse / Era / InkPad class).
    pub fn pocketbook() -> Self {
        use Capability as C;
        const CLAIMS: &[Claim] = &[
            assumption(
                C::UserAgent,
                "a WebKit UA carrying a `PocketBook/` token",
                "the research sources no PocketBook UA at all",
            ),
            assumption(C::Cookies, "cookies are kept", COOKIES_WHY),
            assumption(C::PageWeight, "pages up to 1 MiB", PAGE_CAP_WHY),
            research(C::JavaScript, "WebKit-based browser; level unknown", "devices §2", Grade::M),
            research(
                C::BrowserDownloads,
                "the browser downloads book files",
                "devices §2",
                Grade::M,
            ),
            assumption(
                C::BrowserDownloads,
                "`.lcpl` and `.acsm` downloads are handed to the reading system",
                "on-device licence handling is listed as unverified (devices §3.4) and is the first thing to test on real hardware",
            ),
            research(
                C::ReadingFormats,
                "EPUB, PDF, FB2, MOBI (no DRM), TXT are native",
                "devices §2",
                Grade::V1,
            ),
            research(C::Drm, "Adobe ADE on the device and Readium LCP", "devices §2", Grade::M),
            research(
                C::MailService,
                "Send-to-PocketBook (`@pbsync.com`)",
                "devices §3.4",
                Grade::V1,
            ),
            research(
                C::Opds,
                "OPDS on the e-ink firmware is unclear; treated as absent",
                "devices §2",
                Grade::M,
            ),
            research(C::Screen, "6-inch class (Verse)", "devices §2", Grade::M),
            assumption(
                C::Keyboard,
                "keyboard with a number row; 2 taps to open the browser; 1 tap to confirm a download",
                KEYBOARD_WHY,
            ),
            assumption(C::Quirks, "none modelled", "nothing sourced"),
            REFERRER_CLAIM,
        ];
        Self {
            id: "pocketbook",
            name: "PocketBook (e-ink firmware)",
            family: DeviceFamily::PocketBook,
            user_agent: "Mozilla/5.0 (Linux; U; Android 2.3.4; en-us; PocketBook/629 Build/GRJ22) AppleWebKit/533.1 (KHTML, like Gecko) Version/4.0 Mobile Safari/533.1",
            cookies: true,
            max_page_bytes: 1024 * 1024,
            max_download_bytes: 200 * 1024 * 1024,
            browser_downloads: &[
                FileFormat::Epub,
                FileFormat::Pdf,
                FileFormat::Fb2,
                FileFormat::Mobi,
                FileFormat::Txt,
                FileFormat::Lcpl,
                FileFormat::Acsm,
            ],
            reading_formats: &[
                FileFormat::Epub,
                FileFormat::Pdf,
                FileFormat::Fb2,
                FileFormat::Mobi,
                FileFormat::Txt,
            ],
            drm: &[DrmScheme::AdobeAdept, DrmScheme::ReadiumLcp],
            mail_service: Some(MailServiceSpec::send_to_pocketbook()),
            native_opds: false,
            screen: ScreenClass::Small,
            keyboard: Keyboard::with_number_row(),
            browser_launch_taps: 2,
            download_confirm_taps: 1,
            quirks: Quirks { ignores_referrer_policy: true, ..Quirks::default() },
            claims: CLAIMS,
        }
    }

    /// Vivlio: PocketBook hardware with the French booksellers' firmware.
    pub fn vivlio() -> Self {
        use Capability as C;
        const CLAIMS: &[Claim] = &[
            research(
                C::UserAgent,
                "Vivlio is PocketBook hardware; the same UA is assumed",
                "devices §3.4",
                Grade::M,
            ),
            assumption(C::Cookies, "cookies are kept", COOKIES_WHY),
            assumption(C::PageWeight, "pages up to 1 MiB", PAGE_CAP_WHY),
            research(C::JavaScript, "as PocketBook", "devices §2", Grade::M),
            research(C::BrowserDownloads, "as PocketBook", "devices §2", Grade::M),
            research(C::ReadingFormats, "as PocketBook", "devices §2", Grade::M),
            research(
                C::Drm,
                "Adobe and LCP (Vivlio staff co-authored an LCP note)",
                "devices §3.4",
                Grade::M,
            ),
            research(
                C::MailService,
                "PocketBook channels are assumed available",
                "devices §2",
                Grade::M,
            ),
            research(C::Opds, "unknown; treated as absent", "devices §2", Grade::M),
            research(C::Screen, "6-inch class", "devices §2", Grade::M),
            assumption(C::Keyboard, "as PocketBook", KEYBOARD_WHY),
            assumption(C::Quirks, "none modelled", "nothing sourced"),
            REFERRER_CLAIM,
        ];
        Self {
            id: "vivlio",
            name: "Vivlio (PocketBook hardware)",
            family: DeviceFamily::Vivlio,
            claims: CLAIMS,
            ..Self::pocketbook()
        }
    }

    /// tolino shine 5 / shine color / vision color (2024+), which run a modified Kobo firmware.
    pub fn tolino() -> Self {
        use Capability as C;
        const CLAIMS: &[Claim] = &[
            research(C::UserAgent, "the UA contains `eReader`", "devices §3.3", Grade::V1),
            assumption(C::UserAgent, "full UA string", UA_WHY),
            assumption(C::Cookies, "cookies are kept", COOKIES_WHY),
            assumption(C::PageWeight, "pages up to 1 MiB", PAGE_CAP_WHY),
            research(C::JavaScript, "Kobo-derived firmware: as Kobo", "devices §3.3", Grade::V2),
            research(
                C::BrowserDownloads,
                "browser download confirmed by a user (2024-08)",
                "devices §3.3",
                Grade::V1,
            ),
            research(C::ReadingFormats, "EPUB, PDF, TXT", "devices §2", Grade::V1),
            research(
                C::Drm,
                "Adobe; LCP is low-confidence and not modelled",
                "devices §2",
                Grade::M,
            ),
            research(
                C::MailService,
                "none; own files go through the tolino cloud web upload",
                "devices §3.3",
                Grade::V1,
            ),
            research(C::Opds, "no native OPDS", "devices §2", Grade::M),
            research(C::Screen, "6-inch class (shine)", "devices §2", Grade::M),
            assumption(C::Keyboard, "as Kobo", KEYBOARD_WHY),
            research(
                C::Quirks,
                "behaves as a Kobo (detected as one over USB)",
                "devices §3.3",
                Grade::V1,
            ),
            REFERRER_CLAIM,
        ];
        Self {
            id: "tolino",
            name: "tolino shine/vision (2024+, Kobo-based)",
            family: DeviceFamily::Tolino,
            user_agent: "Mozilla/5.0 (Linux; U; Android 2.0; en-us;) AppleWebKit/538.1 (KHTML, like Gecko) Version/4.0 Mobile Safari/538.1 (eReader)",
            reading_formats: &[FileFormat::Epub, FileFormat::Pdf, FileFormat::Txt],
            drm: &[DrmScheme::AdobeAdept],
            claims: CLAIMS,
            ..Self::kobo_clara()
        }
    }

    /// Older Android-based tolino (Vision 4 HD): the download completes but the reading system
    /// reports the file as damaged, so the flow must offer a fallback.
    pub fn tolino_legacy_android() -> Self {
        use Capability as C;
        const CLAIMS: &[Claim] = &[
            assumption(C::UserAgent, "an Android 4.4 WebView UA naming the device", UA_WHY),
            assumption(C::Cookies, "cookies are kept", COOKIES_WHY),
            assumption(C::PageWeight, "pages up to 1 MiB", PAGE_CAP_WHY),
            assumption(C::JavaScript, "Android 4.4 WebView", "not in the research"),
            research(
                C::BrowserDownloads,
                "the browser download completes",
                "devices §3.3",
                Grade::V1,
            ),
            research(C::ReadingFormats, "EPUB, PDF, TXT", "devices §2", Grade::V1),
            research(C::Drm, "Adobe", "devices §2", Grade::M),
            research(C::MailService, "none", "devices §3.3", Grade::V1),
            research(C::Opds, "no native OPDS", "devices §2", Grade::M),
            research(C::Screen, "6-inch class", "devices §2", Grade::M),
            assumption(
                C::Keyboard,
                "Android keyboard approximated by the three-layer model",
                KEYBOARD_WHY,
            ),
            research(
                C::Quirks,
                "the downloaded file is reported \"damaged\"",
                "devices §3.3",
                Grade::V1,
            ),
            REFERRER_CLAIM,
        ];
        Self {
            id: "tolino-legacy-android",
            name: "tolino Vision 4 HD (Android-based, legacy)",
            family: DeviceFamily::Tolino,
            user_agent: "Mozilla/5.0 (Linux; Android 4.4.2; tolino vision 4 HD Build/KOT49H) AppleWebKit/537.36 (KHTML, like Gecko) Version/4.0 Chrome/30.0.0.0 Safari/537.36",
            reading_formats: &[FileFormat::Epub, FileFormat::Pdf, FileFormat::Txt],
            drm: &[DrmScheme::AdobeAdept],
            browser_launch_taps: 2,
            quirks: Quirks {
                download_reported_damaged: true,
                ignores_referrer_policy: true,
                ..Quirks::default()
            },
            claims: CLAIMS,
            ..Self::kobo_clara()
        }
    }

    /// A deliberately pessimistic e-ink browser: no cookies, small pages only, EPUB and PDF.
    /// Not a real device — the baseline every redemption page must survive.
    pub fn generic_eink() -> Self {
        use Capability as C;
        const WHY: &str = "pessimistic baseline, not a real device";
        const CLAIMS: &[Claim] = &[
            assumption(C::UserAgent, "a WebKit UA with the generic `eReader` token", WHY),
            assumption(C::Cookies, "no cookies (docs/api.md §2: none are needed)", WHY),
            assumption(C::PageWeight, "pages up to 256 KiB", WHY),
            assumption(C::JavaScript, "none", WHY),
            assumption(C::BrowserDownloads, "EPUB and PDF", WHY),
            assumption(C::ReadingFormats, "EPUB, PDF, TXT", WHY),
            assumption(C::Drm, "none", WHY),
            assumption(C::MailService, "none", WHY),
            assumption(C::Opds, "none", WHY),
            assumption(C::Screen, "6-inch class", WHY),
            assumption(C::Keyboard, "three-layer keyboard; 3 taps to open the browser", WHY),
            assumption(C::Quirks, "ASCII file names only", WHY),
            REFERRER_CLAIM,
        ];
        Self {
            id: "generic-eink",
            name: "Generic e-ink browser (pessimistic baseline)",
            family: DeviceFamily::GenericEink,
            user_agent: "Mozilla/5.0 (Linux; U; en-us) AppleWebKit/534.34 (KHTML, like Gecko) Version/4.0 Safari/534.34 eReader",
            cookies: false,
            max_page_bytes: 256 * 1024,
            max_download_bytes: 100 * 1024 * 1024,
            browser_downloads: &[FileFormat::Epub, FileFormat::Pdf],
            reading_formats: &[FileFormat::Epub, FileFormat::Pdf, FileFormat::Txt],
            drm: &[],
            mail_service: None,
            native_opds: false,
            screen: ScreenClass::Small,
            keyboard: Keyboard::three_layer(),
            browser_launch_taps: 3,
            download_confirm_taps: 1,
            quirks: Quirks {
                ascii_download_names: true,
                ignores_referrer_policy: true,
                ..Quirks::default()
            },
            claims: CLAIMS,
        }
    }

    /// Thorium Reader on a computer: the best-evidenced OPDS + LCP reading system. Not an
    /// e-ink device; used by the OPDS and (simulated) LCP flows.
    pub fn thorium_desktop() -> Self {
        use Capability as C;
        const CLAIMS: &[Claim] = &[
            assumption(C::UserAgent, "an Electron UA with a `Thorium/` token", UA_WHY),
            assumption(C::Cookies, "cookies are kept", "Chromium default"),
            assumption(
                C::PageWeight,
                "pages up to 8 MiB",
                "a desktop application; bound only to stop hostile input",
            ),
            assumption(
                C::JavaScript,
                "modern engine (never executed by the simulator)",
                "Electron",
            ),
            research(
                C::BrowserDownloads,
                "imports `.epub` and `.lcpl` files",
                "devices §4",
                Grade::V1,
            ),
            research(C::ReadingFormats, "EPUB (and PDF)", "devices §4", Grade::V1),
            research(C::Drm, "Readium LCP in production builds; no Adobe", "devices §4", Grade::V1),
            assumption(C::MailService, "none", "a desktop application has no device address"),
            research(C::Opds, "OPDS catalogues are supported", "devices §4", Grade::V1),
            assumption(C::Screen, "large", "a computer window"),
            assumption(
                C::Keyboard,
                "a physical keyboard approximated by a number-row layout; no launch taps",
                "taps are not meaningful on a computer",
            ),
            assumption(C::Quirks, "none", "nothing sourced"),
        ];
        Self {
            id: "thorium-desktop",
            name: "Thorium Reader (desktop reading app)",
            family: DeviceFamily::ReadingApp,
            user_agent: "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Thorium/3.1.0 Chrome/130.0.0.0 Electron/33.0.0 Safari/537.36",
            cookies: true,
            max_page_bytes: 8 * 1024 * 1024,
            max_download_bytes: 500 * 1024 * 1024,
            browser_downloads: &[FileFormat::Epub, FileFormat::Pdf, FileFormat::Lcpl],
            reading_formats: &[FileFormat::Epub, FileFormat::Pdf],
            drm: &[DrmScheme::ReadiumLcp],
            mail_service: None,
            native_opds: true,
            screen: ScreenClass::Large,
            keyboard: Keyboard::with_number_row(),
            browser_launch_taps: 0,
            download_confirm_taps: 0,
            quirks: Quirks::default(),
            claims: CLAIMS,
        }
    }

    /// A smartphone's web browser (the phone that scans the QR code of the card).
    ///
    /// Not an e-reader: it is the *other* device of the pairing, Send-to-Kindle and
    /// store-activation journeys. The simulator still never runs JavaScript, so driving a
    /// page with this profile also proves the page needs none.
    pub fn phone() -> Self {
        use Capability as C;
        const WHY: &str = "a current smartphone browser; modelled, not measured";
        const CLAIMS: &[Claim] = &[
            assumption(
                C::UserAgent,
                "an iPhone Safari UA (contains `iPhone` and `Mobile`)",
                UA_WHY,
            ),
            assumption(C::Cookies, "cookies are kept", WHY),
            assumption(C::PageWeight, "pages up to 8 MiB", "bound only to stop hostile input"),
            assumption(C::JavaScript, "modern engine (never executed by the simulator)", WHY),
            assumption(
                C::BrowserDownloads,
                "saves EPUB and PDF to the Files app, from where the share sheet reaches reading apps",
                WHY,
            ),
            assumption(C::ReadingFormats, "EPUB and PDF in the built-in books app", WHY),
            assumption(C::Drm, "none in the browser itself", WHY),
            assumption(C::MailService, "none", "a phone has no device address"),
            assumption(C::Opds, "none in the browser (reading apps have their own client)", WHY),
            assumption(C::Screen, "6-inch class", WHY),
            assumption(
                C::Keyboard,
                "three-layer on-screen keyboard; one tap to open the browser, one to confirm a download",
                KEYBOARD_WHY,
            ),
            assumption(C::Quirks, "none; honours Referrer-Policy", WHY),
        ];
        Self {
            id: "phone",
            name: "Smartphone (web browser)",
            family: DeviceFamily::Phone,
            user_agent: "Mozilla/5.0 (iPhone; CPU iPhone OS 18_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.1 Mobile/15E148 Safari/604.1",
            cookies: true,
            max_page_bytes: 8 * 1024 * 1024,
            max_download_bytes: 500 * 1024 * 1024,
            browser_downloads: &[FileFormat::Epub, FileFormat::Pdf],
            reading_formats: &[FileFormat::Epub, FileFormat::Pdf],
            drm: &[],
            mail_service: None,
            native_opds: false,
            screen: ScreenClass::Small,
            keyboard: Keyboard::three_layer(),
            browser_launch_taps: 1,
            download_confirm_taps: 1,
            quirks: Quirks::default(),
            claims: CLAIMS,
        }
    }

    /// A computer's web browser: the buyer in a store, or the "computer" device class of a
    /// gift page. Like [`phone`](Self::phone), it is not an e-reader and runs no JavaScript.
    pub fn computer() -> Self {
        use Capability as C;
        const WHY: &str = "a current desktop browser; modelled, not measured";
        const CLAIMS: &[Claim] = &[
            assumption(C::UserAgent, "a Firefox-on-Linux UA", UA_WHY),
            assumption(C::Cookies, "cookies are kept", WHY),
            assumption(C::PageWeight, "pages up to 8 MiB", "bound only to stop hostile input"),
            assumption(C::JavaScript, "modern engine (never executed by the simulator)", WHY),
            assumption(C::BrowserDownloads, "saves any file to the downloads folder", WHY),
            assumption(
                C::ReadingFormats,
                "EPUB and PDF (with a reading application installed)",
                WHY,
            ),
            assumption(C::Drm, "none in the browser itself", WHY),
            assumption(C::MailService, "none", "a computer has no device address"),
            assumption(C::Opds, "none in the browser", WHY),
            assumption(C::Screen, "large", "a computer window"),
            assumption(
                C::Keyboard,
                "a physical keyboard approximated by a number-row layout; no launch taps",
                "taps are not meaningful on a computer",
            ),
            assumption(C::Quirks, "none; honours Referrer-Policy", WHY),
        ];
        Self {
            id: "computer",
            name: "Computer (web browser)",
            family: DeviceFamily::Computer,
            user_agent: "Mozilla/5.0 (X11; Linux x86_64; rv:132.0) Gecko/20100101 Firefox/132.0",
            cookies: true,
            max_page_bytes: 8 * 1024 * 1024,
            max_download_bytes: 500 * 1024 * 1024,
            browser_downloads: &[FileFormat::Epub, FileFormat::Pdf],
            reading_formats: &[FileFormat::Epub, FileFormat::Pdf],
            drm: &[],
            mail_service: None,
            native_opds: false,
            screen: ScreenClass::Large,
            keyboard: Keyboard::with_number_row(),
            browser_launch_taps: 0,
            download_confirm_taps: 0,
            quirks: Quirks::default(),
            claims: CLAIMS,
        }
    }

    /// Every built-in profile.
    pub fn all() -> Vec<Self> {
        vec![
            Self::kobo_clara(),
            Self::kobo_ereader_ua(),
            Self::kindle_paperwhite(),
            Self::kindle_2024(),
            Self::pocketbook(),
            Self::vivlio(),
            Self::tolino(),
            Self::tolino_legacy_android(),
            Self::generic_eink(),
            Self::thorium_desktop(),
            Self::phone(),
            Self::computer(),
        ]
    }

    /// Looks a profile up by id, or by a family shorthand (`kobo`, `kindle`, `pocketbook`,
    /// `vivlio`, `tolino`, `generic`, `eink`, `thorium`, `mobile`, `desktop`).
    /// Case-insensitive.
    pub fn by_name(name: &str) -> Option<Self> {
        let wanted = name.trim().to_ascii_lowercase();
        let alias = match wanted.as_str() {
            "kobo" => "kobo-clara",
            "kindle" => "kindle-paperwhite",
            "generic" | "eink" | "e-ink" => "generic-eink",
            "thorium" => "thorium-desktop",
            "mobile" | "smartphone" => "phone",
            "desktop" => "computer",
            other => other,
        };
        Self::all().into_iter().find(|p| p.id == alias)
    }

    /// Whether the built-in browser saves this file type.
    pub fn browser_downloads(&self, format: FileFormat) -> bool {
        self.browser_downloads.contains(&format)
    }

    /// Whether the reading system opens this file type.
    pub fn reads(&self, format: FileFormat) -> bool {
        self.reading_formats.contains(&format)
    }

    /// Whether the reading system can unlock this DRM scheme over the air.
    pub fn unlocks(&self, scheme: DrmScheme) -> bool {
        self.drm.contains(&scheme)
    }

    /// The claims about one capability.
    pub fn claims_about(&self, capability: Capability) -> impl Iterator<Item = &Claim> {
        self.claims.iter().filter(move |c| c.capability == capability)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn every_capability_of_every_profile_is_cited_or_marked_as_assumption() {
        for profile in DeviceProfile::all() {
            for capability in Capability::ALL {
                assert!(
                    profile.claims_about(capability).next().is_some(),
                    "{} has no claim about {capability:?}",
                    profile.id
                );
            }
            for claim in profile.claims {
                assert!(!claim.statement.trim().is_empty());
                match claim.basis {
                    Basis::Research { section, .. } => {
                        assert!(
                            section.starts_with("devices §") || section.starts_with("ecosystems §"),
                            "{}: citation {section:?} does not name a research section",
                            profile.id
                        );
                    }
                    Basis::Assumption(why) => assert!(!why.trim().is_empty()),
                }
            }
        }
    }

    #[test]
    fn profile_ids_are_unique_and_resolvable() {
        let all = DeviceProfile::all();
        let ids: HashSet<_> = all.iter().map(|p| p.id).collect();
        assert_eq!(ids.len(), all.len());
        for profile in &all {
            assert_eq!(DeviceProfile::by_name(profile.id).unwrap().id, profile.id);
            assert!(!profile.user_agent.is_empty() && profile.user_agent.is_ascii());
        }
        assert_eq!(DeviceProfile::by_name(" KOBO ").unwrap().id, "kobo-clara");
        assert_eq!(DeviceProfile::by_name("kindle").unwrap().id, "kindle-paperwhite");
        assert!(DeviceProfile::by_name("nook").is_none());
        assert!(DeviceProfile::by_name("").is_none());
    }

    #[test]
    fn the_kindle_takes_no_epub_but_its_mail_service_does() {
        let kindle = DeviceProfile::kindle_paperwhite();
        assert!(!kindle.browser_downloads(FileFormat::Epub));
        assert!(!kindle.reads(FileFormat::Epub));
        assert!(kindle.browser_downloads(FileFormat::Azw3));
        let service = kindle.mail_service.unwrap();
        assert!(service.accepted.contains(&FileFormat::Epub));
        assert!(!service.accepted.contains(&FileFormat::Azw3));
        assert!(!service.accepted.contains(&FileFormat::Mobi));
        assert_eq!(service.converts_epub_to, Some(FileFormat::Kfx));
        assert!(kindle.reads(FileFormat::Kfx));
    }

    #[test]
    fn user_agents_carry_or_lack_the_tokens_servers_sniff() {
        assert!(DeviceProfile::kobo_clara().user_agent.contains("Kobo Touch"));
        let libra = DeviceProfile::kobo_ereader_ua();
        assert!(libra.user_agent.contains("eReader") && !libra.user_agent.contains("Kobo"));
        assert!(DeviceProfile::kindle_paperwhite().user_agent.contains("Kindle/3"));
        assert!(!DeviceProfile::kindle_2024().user_agent.contains("Kindle"));
        assert!(DeviceProfile::tolino().user_agent.contains("eReader"));
        assert!(DeviceProfile::pocketbook().user_agent.contains("PocketBook/"));
    }

    #[test]
    fn families_map_onto_the_device_chooser_of_the_gift_page() {
        // docs/api.md §2: `?device=kobo|kindle|pocketbook|tolino|phone|computer` and the
        // links `device-kobo` … `device-computer`.
        let expected = [
            ("kobo-clara", Some("kobo")),
            ("kobo-ereader-ua", Some("kobo")),
            ("kindle-paperwhite", Some("kindle")),
            ("kindle-2024", Some("kindle")),
            ("pocketbook", Some("pocketbook")),
            ("vivlio", Some("pocketbook")),
            ("tolino", Some("tolino")),
            ("generic-eink", None),
            ("thorium-desktop", None),
            ("phone", Some("phone")),
            ("computer", Some("computer")),
        ];
        for (id, choice) in expected {
            let profile = DeviceProfile::by_name(id).unwrap();
            assert_eq!(profile.family.device_choice(), choice, "{id}");
            assert_eq!(
                profile.family.chooser_link_id().map(str::to_owned),
                choice.map(|c| format!("device-{c}")),
                "{id}"
            );
        }
        assert!(DeviceProfile::phone().user_agent.contains("iPhone"));
        assert_eq!(DeviceProfile::by_name("Mobile").unwrap().id, "phone");
        assert_eq!(DeviceProfile::by_name("desktop").unwrap().id, "computer");
    }

    #[test]
    fn only_lcp_capable_systems_unlock_lcp() {
        let lcp: Vec<_> = DeviceProfile::all()
            .into_iter()
            .filter(|p| p.unlocks(DrmScheme::ReadiumLcp))
            .map(|p| p.id)
            .collect();
        assert_eq!(lcp, vec!["pocketbook", "vivlio", "thorium-desktop"]);
        // Every reading system opens what its own browser saves, licences aside.
        for profile in DeviceProfile::all() {
            for format in profile.browser_downloads.iter().filter(|f| !f.is_licence()) {
                assert!(
                    profile.reads(*format),
                    "{} downloads {format} but cannot read it",
                    profile.id
                );
            }
        }
    }
}
