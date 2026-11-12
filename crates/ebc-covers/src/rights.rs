//! The rights basis of a cover and the print routes it opens.
//!
//! This is the rights model of `docs/research/07-legal-covers-and-reseller-rights.md`
//! ("Implications for the implementation", items 1, 3, 20–24, 28, 43), encoded as data:
//!
//! | Basis | Tier | Home-print PDF | Professional print order |
//! |---|---|---|---|
//! | `none` (default) | T0 | no cover — typographic card | no cover — typographic card |
//! | `print_licence` | T1 | full cover | full cover |
//! | `promo_all_media` | T1r | **reduced-size** layout only | refused |
//! | `rights_holder_declared` | T2 | full cover | full cover (declarant identified) |
//! | `user_private_copy` | T3 | full cover, own printer, one jacket per gift code | refused |
//! | `public_domain` | T4 | full cover where the per-country test passes | same, delivery country too |
//!
//! The table is a set of conservative defaults to be confirmed by counsel; none of the
//! legal points behind it was verified at source (the research says so). This module only
//! answers "does this basis open this route, and under which conditions?" — the
//! conditions themselves ([`Condition`]) are checked by the host, which holds the data
//! (country, declarations, reseller configuration).

use core::fmt;
use core::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::CoversError;
use crate::placeholder;
use crate::sniff::{self, ImageInfo};

/// Why a cover may be printed. Every cover asset carries exactly one.
///
/// Anything found by a lookup ([`crate::candidates`]) is [`RightsBasis::None`] and stays
/// so: the service never fetches a third-party cover into a printable file on its own
/// initiative.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RightsBasis {
    /// No right recorded (tier T0). The cover must not enter any print file; the output
    /// is the typographic card.
    #[default]
    None,
    /// Per-title print licence from the publisher, passed on by the reseller (tier T1).
    PrintLicence,
    /// The reseller declared in writing that its licence covers promotion on all media,
    /// print included (tier T1r): reduced-size layout only.
    PromoAllMedia,
    /// The person ordering declared to be the author or publisher, or to hold a licence
    /// (tier T2).
    RightsHolderDeclared,
    /// The user's private copy, from a book they bought, printed on their own printer
    /// (tier T3).
    UserPrivateCopy,
    /// The artwork is in the public domain in the country of use (tier T4).
    PublicDomain,
}

impl RightsBasis {
    /// Every value, in declaration order.
    pub const ALL: [Self; 6] = [
        Self::None,
        Self::PrintLicence,
        Self::PromoAllMedia,
        Self::RightsHolderDeclared,
        Self::UserPrivateCopy,
        Self::PublicDomain,
    ];

    /// The stable machine name (`"none"`, `"print_licence"`, `"promo_all_media"`,
    /// `"rights_holder_declared"`, `"user_private_copy"`, `"public_domain"`), as used in
    /// the reseller API, the command line and the PDF metadata.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::PrintLicence => "print_licence",
            Self::PromoAllMedia => "promo_all_media",
            Self::RightsHolderDeclared => "rights_holder_declared",
            Self::UserPrivateCopy => "user_private_copy",
            Self::PublicDomain => "public_domain",
        }
    }

    /// The policy tier of the research (`"T0"` … `"T4"`, `"T1r"`).
    #[must_use]
    pub const fn tier(self) -> &'static str {
        match self {
            Self::None => "T0",
            Self::PrintLicence => "T1",
            Self::PromoAllMedia => "T1r",
            Self::RightsHolderDeclared => "T2",
            Self::UserPrivateCopy => "T3",
            Self::PublicDomain => "T4",
        }
    }

    /// `true` when a cover with this basis may appear in a **home-print PDF**: every
    /// basis except [`RightsBasis::None`]. The conditions of [`RightsBasis::gate`] still
    /// apply (reduced size for `promo_all_media`, own printer for `user_private_copy`,
    /// the per-country test for `public_domain`).
    #[must_use]
    pub const fn allows_home_print(self) -> bool {
        !matches!(self, Self::None)
    }

    /// `true` when a cover with this basis may appear in a **professional print order**:
    /// only `print_licence`, `rights_holder_declared` and `public_domain`.
    #[must_use]
    pub const fn allows_professional_print(self) -> bool {
        matches!(self, Self::PrintLicence | Self::RightsHolderDeclared | Self::PublicDomain)
    }

    /// `true` when the basis opens `route` at all.
    #[must_use]
    pub const fn allows(self, route: PrintRoute) -> bool {
        match route {
            PrintRoute::HomePrint => self.allows_home_print(),
            PrintRoute::ProfessionalOrder => self.allows_professional_print(),
        }
    }

    /// The gate: what a cover with this basis may do on `route`.
    ///
    /// # Errors
    /// [`CoversError::NoRightsBasis`] for [`RightsBasis::None`] on any route;
    /// [`CoversError::RouteNotAllowed`] when the basis does not open the route
    /// (`promo_all_media` and `user_private_copy` on a professional order).
    pub fn gate(self, route: PrintRoute) -> Result<Grant, CoversError> {
        use Condition as C;
        let grant = |placement, conditions| Grant { placement, conditions };
        match (self, route) {
            (Self::None, _) => Err(CoversError::NoRightsBasis),
            (Self::PrintLicence, _) => {
                Ok(grant(Placement::Full, &[C::ResellerWarrantyOnFile, C::CreditLine]))
            }
            (Self::PromoAllMedia, PrintRoute::HomePrint) => Ok(grant(
                Placement::ReducedSize,
                &[C::ResellerDeclarationOnFile, C::FeatureSwitchOn, C::CreditLine],
            )),
            (Self::RightsHolderDeclared, PrintRoute::HomePrint) => {
                Ok(grant(Placement::Full, &[C::CreditLine]))
            }
            (Self::RightsHolderDeclared, PrintRoute::ProfessionalOrder) => {
                Ok(grant(Placement::Full, &[C::DeclarantIdentified, C::CreditLine]))
            }
            (Self::UserPrivateCopy, PrintRoute::HomePrint) => Ok(grant(
                Placement::Full,
                &[C::OwnPrinterOnly, C::OneJacketPerGiftCode, C::FeatureSwitchOn],
            )),
            (Self::PublicDomain, _) => {
                Ok(grant(Placement::Full, &[C::PublicDomainCountryTest, C::CreditLine]))
            }
            (Self::PromoAllMedia | Self::UserPrivateCopy, PrintRoute::ProfessionalOrder) => {
                Err(CoversError::RouteNotAllowed { basis: self, route })
            }
        }
    }
}

impl fmt::Display for RightsBasis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for RightsBasis {
    type Err = CoversError;

    /// Parses a machine name; case-insensitive, `-` accepted for `_`. An empty string is
    /// an error, not `none`: a missing argument must never be mistaken for a choice.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let wanted = s.trim();
        Self::ALL
            .into_iter()
            .find(|basis| {
                let name = basis.as_str();
                name.len() == wanted.len()
                    && name
                        .bytes()
                        .zip(wanted.bytes())
                        .all(|(a, b)| a == b.to_ascii_lowercase() || (a == b'_' && b == b'-'))
            })
            .ok_or_else(|| CoversError::UnknownRightsBasis(wanted.chars().take(64).collect()))
    }
}

/// Where the PDF is going.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrintRoute {
    /// A PDF the user prints on their own printer.
    HomePrint,
    /// An order sent to a professional printer.
    ProfessionalOrder,
}

impl fmt::Display for PrintRoute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::HomePrint => "a home-print PDF",
            Self::ProfessionalOrder => "a professional print order",
        })
    }
}

/// How the cover may be placed on the front panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Placement {
    /// The unaltered cover fills the front panel (uniform scaling, letterboxed).
    Full,
    /// The unaltered cover at reduced size on a neutral ground, with the title retyped
    /// beside it: at most 60 % of the panel's width and height (proposal of the research).
    ReducedSize,
}

/// A condition the host must have checked before using a [`Grant`]. This crate cannot
/// check them: it does not hold countries, declarations or reseller configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Condition {
    /// `print_licence`: the reseller's acceptance of the rights warranty is recorded in
    /// its configuration, and the title carries the per-title flag.
    ResellerWarrantyOnFile,
    /// `promo_all_media`: the reseller's written declaration that its licence covers
    /// promotion on print media is recorded.
    ResellerDeclarationOnFile,
    /// `rights_holder_declared` on a print order: an account, the declarant's name, role
    /// and email, and acceptance of the rights warranty, logged with the order.
    DeclarantIdentified,
    /// `user_private_copy`: the PDF is for the user's own printer; no print order, no
    /// hint towards a copy shop or print service.
    OwnPrinterOnly,
    /// `user_private_copy`: exactly one jacket per gift code, no multi-up imposition.
    OneJacketPerGiftCode,
    /// `public_domain`: the per-country term test passes for the user's country and, for
    /// a print order, the delivery country; unknown inputs fail the test.
    PublicDomainCountryTest,
    /// `user_private_copy` and `promo_all_media` sit behind per-country configuration
    /// switches that are off by default (France: only after counsel's written opinion).
    FeatureSwitchOn,
    /// A credit line for the cover is printed on the back or inside panel.
    CreditLine,
}

/// What a rights basis allows on one print route.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Grant {
    /// How the cover may be placed.
    pub placement: Placement,
    /// Conditions the host must have checked.
    pub conditions: &'static [Condition],
}

impl Grant {
    /// `true` when the grant carries `condition`.
    #[must_use]
    pub fn requires(&self, condition: Condition) -> bool {
        self.conditions.contains(&condition)
    }
}

/// How the bytes of a printable cover reached the caller. There are exactly these ways;
/// "found by a lookup of the hosted service" is deliberately not one of them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetOrigin {
    /// Supplied by the reseller under its per-title rights flag (feed bytes or feed URL).
    ResellerFeed,
    /// An image file the user picked or dropped in the browser.
    LocalFile,
    /// Extracted in the browser from the user's own, unencrypted EPUB.
    LocalEpub,
    /// A cover or flat full cover uploaded by a declared rights holder.
    RightsHolderUpload,
    /// Downloaded by an operator's command-line or self-hosted tool that was given an
    /// explicit rights basis; the operator is responsible for the rights.
    OperatorDownload,
}

impl AssetOrigin {
    /// The rights bases that can go with this origin.
    #[must_use]
    pub const fn compatible_bases(self) -> &'static [RightsBasis] {
        use RightsBasis as B;
        match self {
            Self::ResellerFeed => &[B::PrintLicence, B::PromoAllMedia],
            Self::LocalFile | Self::LocalEpub => {
                &[B::RightsHolderDeclared, B::UserPrivateCopy, B::PublicDomain]
            }
            Self::RightsHolderUpload => &[B::RightsHolderDeclared],
            Self::OperatorDownload => &[B::PrintLicence, B::RightsHolderDeclared, B::PublicDomain],
        }
    }
}

impl fmt::Display for AssetOrigin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::ResellerFeed => "a reseller feed",
            Self::LocalFile => "a local file",
            Self::LocalEpub => "a local EPUB",
            Self::RightsHolderUpload => "a rights-holder upload",
            Self::OperatorDownload => "an operator download",
        })
    }
}

/// Image bytes that may enter a print file: a recognised image, not a placeholder, with
/// a rights basis other than [`RightsBasis::None`] that fits where the bytes came from.
///
/// This is the only type of the crate that pairs pixels with a right to print them, and
/// the only way to obtain one is [`PrintableCover::new`], from bytes the caller already
/// holds. A [`crate::CoverCandidate`] cannot be converted into it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrintableCover {
    bytes: Vec<u8>,
    info: ImageInfo,
    basis: RightsBasis,
    origin: AssetOrigin,
}

impl PrintableCover {
    /// Wraps `bytes` with their rights basis and origin.
    ///
    /// # Errors
    /// * [`CoversError::NoRightsBasis`] when `basis` is `none`;
    /// * [`CoversError::OriginMismatch`] when the basis cannot go with the origin (e.g.
    ///   `user_private_copy` on an operator download, `print_licence` on a local file);
    /// * [`CoversError::NotAnImage`] when the bytes are not a recognised image;
    /// * [`CoversError::Placeholder`] when they are a "no cover" placeholder
    ///   ([`crate::placeholder::assess`]).
    pub fn new(
        bytes: Vec<u8>,
        basis: RightsBasis,
        origin: AssetOrigin,
    ) -> Result<Self, CoversError> {
        if basis == RightsBasis::None {
            return Err(CoversError::NoRightsBasis);
        }
        if !origin.compatible_bases().contains(&basis) {
            return Err(CoversError::OriginMismatch { origin, basis });
        }
        let details = sniff::probe_details(&bytes).ok_or(CoversError::NotAnImage)?;
        if placeholder::assess(&details.info, &bytes).is_placeholder() {
            return Err(CoversError::Placeholder);
        }
        Ok(Self { bytes, info: details.oriented(), basis, origin })
    }

    /// The encoded image.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Consumes the asset and returns the encoded image.
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    /// Format and pixel size **as displayed** (EXIF orientation applied).
    #[must_use]
    pub fn info(&self) -> ImageInfo {
        self.info
    }

    /// The rights basis; never [`RightsBasis::None`].
    #[must_use]
    pub fn basis(&self) -> RightsBasis {
        self.basis
    }

    /// Where the bytes came from.
    #[must_use]
    pub fn origin(&self) -> AssetOrigin {
        self.origin
    }

    /// What this cover may do on `route` (see [`RightsBasis::gate`]).
    ///
    /// # Errors
    /// [`CoversError::RouteNotAllowed`] when the basis does not open the route.
    pub fn gate(&self, route: PrintRoute) -> Result<Grant, CoversError> {
        self.basis.gate(route)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_none_and_none_opens_nothing() {
        assert_eq!(RightsBasis::default(), RightsBasis::None);
        for route in [PrintRoute::HomePrint, PrintRoute::ProfessionalOrder] {
            assert!(!RightsBasis::None.allows(route));
            assert_eq!(RightsBasis::None.gate(route), Err(CoversError::NoRightsBasis));
        }
    }

    #[test]
    fn home_print_accepts_everything_but_none() {
        for basis in RightsBasis::ALL {
            assert_eq!(basis.allows_home_print(), basis != RightsBasis::None, "{basis}");
            assert_eq!(basis.gate(PrintRoute::HomePrint).is_ok(), basis.allows_home_print());
        }
    }

    #[test]
    fn professional_order_accepts_exactly_three_bases() {
        let allowed: Vec<RightsBasis> =
            RightsBasis::ALL.into_iter().filter(|b| b.allows_professional_print()).collect();
        assert_eq!(
            allowed,
            [
                RightsBasis::PrintLicence,
                RightsBasis::RightsHolderDeclared,
                RightsBasis::PublicDomain
            ]
        );
        for basis in RightsBasis::ALL {
            let gate = basis.gate(PrintRoute::ProfessionalOrder);
            assert_eq!(gate.is_ok(), basis.allows_professional_print(), "{basis}");
            // Whatever a professional order accepts, home print accepts too.
            if gate.is_ok() {
                assert!(basis.allows_home_print());
            }
        }
        assert_eq!(
            RightsBasis::UserPrivateCopy.gate(PrintRoute::ProfessionalOrder),
            Err(CoversError::RouteNotAllowed {
                basis: RightsBasis::UserPrivateCopy,
                route: PrintRoute::ProfessionalOrder
            })
        );
    }

    #[test]
    fn conditions_per_basis() {
        let home = |b: RightsBasis| b.gate(PrintRoute::HomePrint).expect("home print");
        assert_eq!(home(RightsBasis::PromoAllMedia).placement, Placement::ReducedSize);
        assert!(home(RightsBasis::PromoAllMedia).requires(Condition::ResellerDeclarationOnFile));
        assert_eq!(home(RightsBasis::PrintLicence).placement, Placement::Full);
        let private = home(RightsBasis::UserPrivateCopy);
        assert!(private.requires(Condition::OwnPrinterOnly));
        assert!(private.requires(Condition::OneJacketPerGiftCode));
        assert!(private.requires(Condition::FeatureSwitchOn));
        assert!(home(RightsBasis::PublicDomain).requires(Condition::PublicDomainCountryTest));
        let order =
            RightsBasis::RightsHolderDeclared.gate(PrintRoute::ProfessionalOrder).expect("order");
        assert!(order.requires(Condition::DeclarantIdentified));
        assert!(!home(RightsBasis::RightsHolderDeclared).requires(Condition::DeclarantIdentified));
    }

    #[test]
    fn names_round_trip() {
        for basis in RightsBasis::ALL {
            assert_eq!(basis.as_str().parse::<RightsBasis>(), Ok(basis));
            assert_eq!(basis.to_string(), basis.as_str());
            let json = serde_json::to_string(&basis).expect("serialises");
            assert_eq!(json, format!("\"{}\"", basis.as_str()));
            assert_eq!(serde_json::from_str::<RightsBasis>(&json).expect("parses"), basis);
        }
        assert_eq!(" Print-Licence ".parse::<RightsBasis>(), Ok(RightsBasis::PrintLicence));
        for bad in ["", "licence", "none ", "print_licence2", "public domain", "é"] {
            let parsed = bad.parse::<RightsBasis>();
            if bad == "none " {
                assert_eq!(parsed, Ok(RightsBasis::None));
            } else {
                assert!(matches!(parsed, Err(CoversError::UnknownRightsBasis(_))), "{bad:?}");
            }
        }
        let tiers: Vec<&str> = RightsBasis::ALL.iter().map(|b| b.tier()).collect();
        assert_eq!(tiers, ["T0", "T1", "T1r", "T2", "T3", "T4"]);
    }

    #[test]
    fn origins_constrain_the_basis() {
        for origin in [
            AssetOrigin::ResellerFeed,
            AssetOrigin::LocalFile,
            AssetOrigin::LocalEpub,
            AssetOrigin::RightsHolderUpload,
            AssetOrigin::OperatorDownload,
        ] {
            assert!(!origin.compatible_bases().contains(&RightsBasis::None), "{origin}");
            assert!(!origin.compatible_bases().is_empty());
        }
        assert!(
            !AssetOrigin::OperatorDownload
                .compatible_bases()
                .contains(&RightsBasis::UserPrivateCopy)
        );
        assert!(!AssetOrigin::LocalFile.compatible_bases().contains(&RightsBasis::PrintLicence));
    }
}
