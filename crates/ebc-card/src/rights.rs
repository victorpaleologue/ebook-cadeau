//! The rights basis of a cover and the print routes it opens.
//!
//! This is the rights model of `docs/research/07-legal-covers-and-reseller-rights.md`
//! ("Implications for the implementation", items 1, 3 and 9) as the composer enforces it:
//!
//! | Basis | Home-print profile | Professional profile |
//! |---|---|---|
//! | `none` (default) | typographic card | typographic card |
//! | `print_licence` | full cover | full cover |
//! | `promo_all_media` | **reduced-size** layout | refused ([`CardError::RightsRouteRefused`]) |
//! | `rights_holder_declared` | full cover | full cover |
//! | `user_private_copy` | full cover (own printer) | refused |
//! | `public_domain` | full cover | full cover |
//!
//! These are conservative defaults pending a lawyer's review; nothing here is legal
//! advice. The conditions that go with a basis (declarations on file, the per-country
//! public-domain test, one jacket per gift code, …) are the **caller's** to check: the
//! composer only sees the basis.
//!
//! `ebc-covers` declares an enum with the same six values and machine names. It did not
//! compile when this crate was written (two of its modules were missing), so the type is
//! defined here; the names are identical on purpose, so that the two can be unified by a
//! re-export once `ebc-covers` builds.

use core::fmt;
use core::str::FromStr;

use ebc_layout::PrintProfile;

use crate::error::CardError;

/// Why a cover may be printed. Every cover handed to the composer carries exactly one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RightsBasis {
    /// No right recorded. The cover never enters the print file: the output is the
    /// typographic card.
    #[default]
    None,
    /// Per-title print licence from the publisher, passed on by the reseller.
    PrintLicence,
    /// The reseller declared that its licence covers promotion on all media, print
    /// included: reduced-size layout only, home print only.
    PromoAllMedia,
    /// The person ordering declared to be the author or publisher, or to hold a licence.
    RightsHolderDeclared,
    /// The user's private copy, from a book they bought, printed on their own printer.
    UserPrivateCopy,
    /// The artwork is in the public domain in the country of use.
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

    /// The stable machine name: `"none"`, `"print_licence"`, `"promo_all_media"`,
    /// `"rights_holder_declared"`, `"user_private_copy"` or `"public_domain"`. Written to
    /// the PDF as the `EBC.RightsBasis` Info key.
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

    /// `true` when a cover with this basis may appear in a home-print PDF: every basis
    /// except [`RightsBasis::None`].
    #[must_use]
    pub const fn allows_home_print(self) -> bool {
        !matches!(self, Self::None)
    }

    /// `true` when a cover with this basis may appear in a file for a professional
    /// printer: only `print_licence`, `rights_holder_declared` and `public_domain`.
    #[must_use]
    pub const fn allows_professional_print(self) -> bool {
        matches!(self, Self::PrintLicence | Self::RightsHolderDeclared | Self::PublicDomain)
    }

    /// `true` when the basis allows artwork to be *adapted* (a spine sampled from the
    /// cover, a provided spine re-fitted) once the request also says
    /// `adaptation_authorised`: only `print_licence` and `rights_holder_declared`.
    #[must_use]
    pub const fn allows_adaptation(self) -> bool {
        matches!(self, Self::PrintLicence | Self::RightsHolderDeclared)
    }

    /// The gate: how a cover with this basis may be placed on `route`.
    ///
    /// # Errors
    /// [`CardError::RightsRouteRefused`] when the basis does not open the route — `none`
    /// on any route, `promo_all_media` and `user_private_copy` on a professional one.
    /// (The composer itself never passes `none` here: a cover without a rights basis is
    /// dropped and the typographic card is produced.)
    pub fn gate(self, route: PrintRoute) -> Result<Placement, CardError> {
        let allowed = match route {
            PrintRoute::HomePrint => self.allows_home_print(),
            PrintRoute::Professional => self.allows_professional_print(),
        };
        if !allowed {
            return Err(CardError::RightsRouteRefused { basis: self, route });
        }
        Ok(if self == Self::PromoAllMedia { Placement::ReducedSize } else { Placement::Full })
    }
}

impl fmt::Display for RightsBasis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for RightsBasis {
    type Err = CardError;

    /// Parses a machine name; case-insensitive, `-` accepted for `_`. An empty string is
    /// an error, not `none`: a missing argument must never be mistaken for a choice.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let wanted = s.trim().to_ascii_lowercase().replace('-', "_");
        Self::ALL
            .into_iter()
            .find(|basis| basis.as_str() == wanted)
            .ok_or_else(|| CardError::UnknownRightsBasis(s.trim().chars().take(64).collect()))
    }
}

/// Where the PDF is going, as far as the rights gate is concerned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrintRoute {
    /// A sheet the user prints on their own printer (a profile with a
    /// [`sheet`](PrintProfile::sheet)).
    HomePrint,
    /// A file for a professional printer (a profile without a sheet: trim + bleed, with
    /// or without marks).
    Professional,
}

impl PrintRoute {
    /// The route a profile stands for: [`PrintRoute::HomePrint`] for sheet-based
    /// profiles (`home_*`), [`PrintRoute::Professional`] for everything else. The route
    /// cannot be chosen independently of the profile, so that a press-ready file can
    /// never be produced under a home-only rights basis.
    #[must_use]
    pub fn of(profile: &PrintProfile) -> Self {
        if profile.sheet.is_some() { Self::HomePrint } else { Self::Professional }
    }
}

impl fmt::Display for PrintRoute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::HomePrint => "a home-print profile",
            Self::Professional => "a professional print profile",
        })
    }
}

/// How the cover is placed on the front panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Placement {
    /// The whole cover on the front panel: letterboxed inside the safe area by default,
    /// full bleed when the request says `trim_authorised`.
    Full,
    /// The whole cover at no more than 60 % of the panel's width and height, on a neutral
    /// ground, with the title retyped next to it (`promo_all_media`).
    ReducedSize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_round_trip() {
        for basis in RightsBasis::ALL {
            assert_eq!(basis.as_str().parse::<RightsBasis>().ok(), Some(basis));
            assert_eq!(basis.to_string(), basis.as_str());
        }
        assert_eq!("Print-Licence".parse::<RightsBasis>().ok(), Some(RightsBasis::PrintLicence));
        assert!(matches!("".parse::<RightsBasis>(), Err(CardError::UnknownRightsBasis(_))));
        assert!(matches!("fair_use".parse::<RightsBasis>(), Err(CardError::UnknownRightsBasis(_))));
        assert_eq!(RightsBasis::default(), RightsBasis::None);
    }

    #[test]
    fn the_gate_is_the_table_of_the_module_documentation() {
        use Placement::{Full, ReducedSize};
        use RightsBasis as B;
        let home = |b: B| b.gate(PrintRoute::HomePrint).ok();
        let pro = |b: B| b.gate(PrintRoute::Professional).ok();
        assert_eq!((home(B::None), pro(B::None)), (None, None));
        assert_eq!((home(B::PrintLicence), pro(B::PrintLicence)), (Some(Full), Some(Full)));
        assert_eq!((home(B::PromoAllMedia), pro(B::PromoAllMedia)), (Some(ReducedSize), None));
        assert_eq!(
            (home(B::RightsHolderDeclared), pro(B::RightsHolderDeclared)),
            (Some(Full), Some(Full))
        );
        assert_eq!((home(B::UserPrivateCopy), pro(B::UserPrivateCopy)), (Some(Full), None));
        assert_eq!((home(B::PublicDomain), pro(B::PublicDomain)), (Some(Full), Some(Full)));
        assert!(matches!(
            B::UserPrivateCopy.gate(PrintRoute::Professional),
            Err(CardError::RightsRouteRefused {
                basis: B::UserPrivateCopy,
                route: PrintRoute::Professional
            })
        ));
    }

    #[test]
    fn routes_follow_the_profile() {
        use ebc_layout::Sheet;
        assert_eq!(PrintRoute::of(&PrintProfile::professional_eu()), PrintRoute::Professional);
        assert_eq!(PrintRoute::of(&PrintProfile::upload_eu()), PrintRoute::Professional);
        assert_eq!(PrintRoute::of(&PrintProfile::home(Sheet::A4)), PrintRoute::HomePrint);
        assert_eq!(PrintRoute::of(&PrintProfile::home(Sheet::Letter)), PrintRoute::HomePrint);
    }

    #[test]
    fn only_licensed_bases_allow_adaptation() {
        let allowed: Vec<_> =
            RightsBasis::ALL.into_iter().filter(|b| b.allows_adaptation()).collect();
        assert_eq!(allowed, [RightsBasis::PrintLicence, RightsBasis::RightsHolderDeclared]);
    }
}
