//! The command line, as `clap` derive types.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::error::EXIT_HELP;

/// The `ebc` command line.
#[derive(Debug, Parser)]
#[command(
    name = "ebc",
    version,
    about = "eBook Cadeau: ISBN tools, metadata lookup, cover candidates, the printable gift card, preflight and QR codes",
    after_help = EXIT_HELP,
    propagate_version = true,
    arg_required_else_help = true
)]
pub(crate) struct Cli {
    /// What to do.
    #[command(subcommand)]
    pub(crate) command: Command,
}

/// The subcommands.
#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Validate, convert and hyphenate an ISBN, or find every ISBN in a text file
    Isbn(IsbnArgs),
    /// Look a book up by ISBN in the catalogue providers and print the merged record
    Lookup(LookupArgs),
    /// List cover candidates for an ISBN; save one only in operator mode (--rights-basis)
    Cover(CoverArgs),
    /// Compose the gift card as a print-ready PDF
    Card(Box<CardArgs>),
    /// Check a PDF against a print profile (exit status 2 when it has errors)
    Preflight(PreflightArgs),
    /// Write a QR code as an SVG image
    Qr(QrArgs),
    /// Rank the fonts of an index by resemblance to the title lettering of a cover image
    #[cfg(feature = "fontid")]
    Fontid(FontidArgs),
}

/// `ebc isbn`
#[derive(Debug, Args)]
pub(crate) struct IsbnArgs {
    /// An ISBN-10 or ISBN-13, with or without hyphens, spaces or an "ISBN" prefix
    #[arg(value_name = "TEXT", required_unless_present = "extract", conflicts_with = "extract")]
    pub(crate) text: Option<String>,
    /// Find every valid ISBN in a text file instead (`-` reads standard input)
    #[arg(long, value_name = "FILE")]
    pub(crate) extract: Option<PathBuf>,
    /// Print JSON
    #[arg(long)]
    pub(crate) json: bool,
}

/// Network settings shared by `lookup` and `cover`.
#[derive(Debug, Args)]
pub(crate) struct NetArgs {
    /// Answer HTTP requests from DIR/index.json instead of the network
    #[arg(long, value_name = "DIR")]
    pub(crate) offline_fixtures: Option<PathBuf>,
    /// Seconds allowed per request
    #[arg(long, value_name = "SECONDS", default_value_t = 8, value_parser = clap::value_parser!(u64).range(1..=120))]
    pub(crate) timeout: u64,
    /// Ignore HTTPS_PROXY and friends (to reach a server on this machine)
    #[arg(long)]
    pub(crate) no_proxy: bool,
    /// The User-Agent to send [default: the library's, which names this project]
    #[arg(long, value_name = "TEXT")]
    pub(crate) user_agent: Option<String>,
}

/// `ebc lookup`
#[derive(Debug, Args)]
pub(crate) struct LookupArgs {
    /// The ISBN to look up
    #[arg(value_name = "ISBN", required_unless_present = "list_providers")]
    pub(crate) isbn: Option<String>,
    /// Query only these providers, in this order (repeatable; see --list-providers)
    #[arg(long = "provider", value_name = "ID")]
    pub(crate) providers: Vec<String>,
    /// Point a provider at another server: PROVIDER=URL (repeatable). For SRU and SPARQL
    /// providers URL is the full endpoint, for the others the site root
    #[arg(long = "base-url", value_name = "PROVIDER=URL")]
    pub(crate) base_urls: Vec<String>,
    /// Google Books API key; enables that provider. Prefer the environment variable:
    /// arguments are visible to other users of the machine
    #[arg(long, value_name = "KEY", env = "EBC_GOOGLE_BOOKS_KEY", hide_env_values = true)]
    pub(crate) google_books_key: Option<String>,
    /// Contact e-mail sent to services that ask for one (Crossref)
    #[arg(long, value_name = "EMAIL", env = "EBC_CONTACT", hide_env_values = true)]
    pub(crate) contact: Option<String>,
    /// Query every selected provider even when the record is already complete
    #[arg(long)]
    pub(crate) all: bool,
    /// Print the providers (id, licence, terms, default) and exit
    #[arg(long)]
    pub(crate) list_providers: bool,
    /// Print JSON
    #[arg(long)]
    pub(crate) json: bool,
    /// Network settings.
    #[command(flatten)]
    pub(crate) net: NetArgs,
}

/// Risk ceiling of `ebc cover`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum RiskArg {
    /// Contract, open licence or the user's own file only
    Low,
    /// Also sources whose terms are unread: locating and previewing only
    Medium,
    /// Also downloads; only meaningful with --rights-basis
    High,
}

/// `ebc cover`
#[derive(Debug, Args)]
pub(crate) struct CoverArgs {
    /// The book's ISBN
    #[arg(value_name = "ISBN")]
    pub(crate) isbn: String,
    /// Operator mode: download the first usable candidate under this rights basis
    /// (print_licence, rights_holder_declared or public_domain). You are responsible for
    /// holding that right. Requires -o
    #[arg(long, value_name = "BASIS")]
    pub(crate) rights_basis: Option<String>,
    /// Where to save the image (operator mode only)
    #[arg(short, long, value_name = "FILE")]
    pub(crate) output: Option<PathBuf>,
    /// Leave out candidates riskier than this [default: medium; high in operator mode]
    #[arg(long, value_name = "RISK", value_enum)]
    pub(crate) max_risk: Option<RiskArg>,
    /// Run the fact-only requests (cover exists? declared pixel size) without downloading
    /// any image; robots.txt and per-host delays are honoured
    #[arg(long)]
    pub(crate) probe: bool,
    /// Also use a source that is off until its terms have been read: bnf, dnb (repeatable)
    #[arg(long = "source", value_name = "ID")]
    pub(crate) sources: Vec<String>,
    /// Google Books API key; enables that source
    #[arg(long, value_name = "KEY", env = "EBC_GOOGLE_BOOKS_KEY", hide_env_values = true)]
    pub(crate) google_books_key: Option<String>,
    /// Operator mode: do not save an image below this resolution on the front panel
    #[arg(long, value_name = "PPI", default_value_t = 0.0)]
    pub(crate) min_ppi: f64,
    /// Card format the print quality is judged for
    #[arg(long, value_name = "FORMAT", default_value = "poche")]
    pub(crate) format: String,
    /// Print each candidate's note on terms and obligations
    #[arg(long)]
    pub(crate) notes: bool,
    /// Print JSON
    #[arg(long)]
    pub(crate) json: bool,
    /// Network settings.
    #[command(flatten)]
    pub(crate) net: NetArgs,
}

/// `ebc card`
#[derive(Debug, Args)]
pub(crate) struct CardArgs {
    /// Title of the book
    #[arg(long, value_name = "TEXT")]
    pub(crate) title: String,
    /// Subtitle
    #[arg(long, value_name = "TEXT")]
    pub(crate) subtitle: Option<String>,
    /// Author (repeatable)
    #[arg(long = "author", value_name = "NAME")]
    pub(crate) authors: Vec<String>,
    /// Publisher
    #[arg(long, value_name = "NAME")]
    pub(crate) publisher: Option<String>,
    /// ISBN of the book: checked, but neither printed nor looked up
    #[arg(long, value_name = "ISBN")]
    pub(crate) isbn: Option<String>,
    /// Language of the book, e.g. fr or en (decides the reading direction of the spine)
    #[arg(long, value_name = "TAG")]
    pub(crate) book_language: Option<String>,

    /// Cover image (JPEG, PNG or WebP). Needs --cover-rights: without a rights basis no
    /// cover is ever printed
    #[arg(long, value_name = "FILE")]
    pub(crate) cover: Option<PathBuf>,
    /// Why this cover may be printed: print_licence, promo_all_media,
    /// rights_holder_declared, user_private_copy, public_domain (or none)
    #[arg(long, value_name = "BASIS", requires = "cover")]
    pub(crate) cover_rights: Option<String>,
    /// Compose the typographic card (title and author as text) and ignore --cover
    #[arg(long, conflicts_with = "cover_rights")]
    pub(crate) typographic: bool,
    /// Credit line printed for the cover, e.g. "© Éditions du Lac / Jeanne Roy"
    #[arg(long, value_name = "TEXT", requires = "cover")]
    pub(crate) cover_credit: Option<String>,
    /// The cover may be cropped by up to the bleed: full-bleed placement
    #[arg(long, requires = "cover")]
    pub(crate) trim_authorised: bool,

    /// The gift code printed on the card (twelve letters, e.g. BCDF-GHJK-LMNP)
    #[arg(
        long,
        value_name = "CODE",
        required_unless_present = "random_code",
        conflicts_with = "random_code"
    )]
    pub(crate) code: Option<String>,
    /// Draw a fresh code from the system's secure random source and print it
    #[arg(long)]
    pub(crate) random_code: bool,
    /// Address of the redemption service, e.g. `https://ebk.example`
    #[arg(long, value_name = "URL")]
    pub(crate) base_url: String,

    /// Who gives the book
    #[arg(long, value_name = "NAME")]
    pub(crate) giver: Option<String>,
    /// Who receives it
    #[arg(long, value_name = "NAME")]
    pub(crate) recipient: Option<String>,
    /// Personal message (ruled space for handwriting is left without one)
    #[arg(long, value_name = "TEXT")]
    pub(crate) message: Option<String>,
    /// Name of the store that fulfils the gift
    #[arg(long, value_name = "NAME")]
    pub(crate) store: Option<String>,
    /// Language of the printed strings: fr, en, de, es, it, nl, pt
    #[arg(long, value_name = "TAG", default_value = "fr")]
    pub(crate) locale: String,

    /// Card format: poche, a6, digest, a5, or `WIDTHxHEIGHT[xSPINE]` in millimetres
    #[arg(long, value_name = "FORMAT", default_value = "poche")]
    pub(crate) format: String,
    /// Print profile: upload (PDF/X-4, bleed, no marks), pro (the same with crop and
    /// fold marks), home-a4, home-letter, home-a3 (plain PDF on a sheet)
    #[arg(long, value_name = "PROFILE", default_value = "upload")]
    pub(crate) profile: String,
    /// CMYK ICC profile for the output intent of a PDF/X file [default: bundled FOGRA39]
    #[arg(long, value_name = "FILE", requires = "output_condition")]
    pub(crate) icc: Option<PathBuf>,
    /// Name of the printing condition of --icc in the ICC registry, e.g. FOGRA51
    #[arg(long, value_name = "NAME", requires = "icc")]
    pub(crate) output_condition: Option<String>,
    /// Creation time written to the PDF, in seconds since 1970 [default:
    /// $SOURCE_DATE_EPOCH, else now]. Fix it for byte-identical output
    #[arg(long, value_name = "SECONDS", allow_negative_numbers = true)]
    pub(crate) created_unix: Option<i64>,
    /// Add the proofing overlay (bleed, trim, creases, safe areas): a proof, not a print file
    #[arg(long)]
    pub(crate) guides: bool,

    /// Where to write the PDF (`-` for standard output)
    #[arg(short, long, value_name = "FILE")]
    pub(crate) output: PathBuf,
    /// Also write SVG previews of both sides into DIR (outside.svg, inside.svg)
    #[arg(long, value_name = "DIR")]
    pub(crate) preview_svg: Option<PathBuf>,
    /// Print a JSON summary (file, size, warnings)
    #[arg(long)]
    pub(crate) json: bool,
}

/// `ebc preflight`
#[derive(Debug, Args)]
pub(crate) struct PreflightArgs {
    /// The PDF to check (`-` reads standard input)
    #[arg(value_name = "PDF")]
    pub(crate) pdf: PathBuf,
    /// The print profile the file was made for: upload, pro, home-a4, home-letter, home-a3
    #[arg(long, value_name = "PROFILE", default_value = "upload")]
    pub(crate) profile: String,
    /// The card format the file was made for (trim size, spine width)
    #[arg(long, value_name = "FORMAT", default_value = "poche")]
    pub(crate) format: String,
    /// Check against the profile only: no expected trim size, spine or QR module
    #[arg(long, conflicts_with = "format")]
    pub(crate) generic: bool,
    /// Accept RGB colour in a professional file (for printers that convert RGB
    /// themselves; cards made by `ebc card` are CMYK)
    #[arg(long)]
    pub(crate) allow_rgb: bool,
    /// Print the full report as JSON
    #[arg(long)]
    pub(crate) json: bool,
}

/// Error-correction level of `ebc qr`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "UPPER")]
pub(crate) enum EcArg {
    /// About 7 % of the symbol may be damaged
    #[value(alias = "l")]
    L,
    /// About 15 %: the usual choice for print
    #[value(alias = "m")]
    M,
    /// About 25 %
    #[value(alias = "q")]
    Q,
    /// About 30 %
    #[value(alias = "h")]
    H,
}

/// `ebc qr`
#[derive(Debug, Args)]
pub(crate) struct QrArgs {
    /// The text to encode
    #[arg(value_name = "TEXT")]
    pub(crate) text: String,
    /// Where to write the SVG (`-` for standard output)
    #[arg(short, long, value_name = "FILE")]
    pub(crate) output: PathBuf,
    /// Minimum error-correction level
    #[arg(long, value_name = "LEVEL", value_enum, default_value = "M")]
    pub(crate) ec: EcArg,
    /// Upper-case the scheme and host of an http(s) URL so that it needs a smaller
    /// symbol (the code then decodes to that equivalent URL, not to TEXT verbatim)
    #[arg(long)]
    pub(crate) compact_url: bool,
    /// Pixels per module
    #[arg(long, value_name = "N", default_value_t = 8, value_parser = clap::value_parser!(u32).range(1..=256))]
    pub(crate) module_px: u32,
    /// Quiet zone around the symbol, in modules (the standard asks for 4)
    #[arg(long, value_name = "N", default_value_t = 4, value_parser = clap::value_parser!(u32).range(0..=64))]
    pub(crate) quiet: u32,
}

/// `ebc fontid`
#[cfg(feature = "fontid")]
#[derive(Debug, Args)]
pub(crate) struct FontidArgs {
    /// The cover image (JPEG, PNG or WebP)
    #[arg(value_name = "COVER-IMAGE")]
    pub(crate) image: PathBuf,
    /// The title as printed on the cover: finds the lettering and labels its glyphs
    #[arg(long, value_name = "TEXT")]
    pub(crate) title: String,
    /// A font index file [default: the bundled index of openly licensed fonts]
    #[arg(long, value_name = "FILE")]
    pub(crate) index: Option<PathBuf>,
    /// How many fonts to list
    #[arg(long, value_name = "N", default_value_t = 5, value_parser = clap::value_parser!(u32).range(1..=100))]
    pub(crate) top: u32,
    /// Directory holding the font files of the index (matched by file name): the best
    /// candidates are then re-ranked by rendering the title with each of them
    #[arg(long, value_name = "DIR")]
    pub(crate) fonts: Option<PathBuf>,
    /// Where the title is, in pixels, instead of detecting it: X,Y,WIDTH,HEIGHT
    #[arg(long, value_name = "X,Y,W,H")]
    pub(crate) region: Option<String>,
    /// Print JSON
    #[arg(long)]
    pub(crate) json: bool,
}
