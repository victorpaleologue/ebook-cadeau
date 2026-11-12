//! The rights basis of a cover and the print routes it opens.
//!
//! The types are those of `ebc-covers` ([`RightsBasis`], [`PrintRoute`], [`Placement`]),
//! re-exported so that one rights basis travels unchanged from the place a cover is
//! accepted to the composer. This module adds what the composer needs on top: which
//! route a print profile stands for, and the gate as it is enforced here.
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
//! This is the rights model of `docs/research/07-legal-covers-and-reseller-rights.md`
//! ("Implications for the implementation", items 1, 3, 9 and 12): conservative defaults
//! pending a lawyer's review, not legal advice. The conditions that go with a basis
//! ([`ebc_covers::Condition`]: declarations on file, the per-country public-domain test,
//! one jacket per gift code, …) are the **caller's** to check — the composer only sees
//! the basis.

use ebc_layout::PrintProfile;

pub use ebc_covers::{Placement, PrintRoute, RightsBasis};

use crate::error::CardError;

/// The route a profile stands for: [`PrintRoute::HomePrint`] for sheet-based profiles
/// (`home_*`), [`PrintRoute::ProfessionalOrder`] for everything else (trim + bleed, with
/// or without marks).
///
/// The route cannot be chosen independently of the profile, so that a press-ready file
/// is never produced under a home-only rights basis.
#[must_use]
pub fn route_of(profile: &PrintProfile) -> PrintRoute {
    if profile.sheet.is_some() { PrintRoute::HomePrint } else { PrintRoute::ProfessionalOrder }
}

/// The gate: how a cover with rights basis `basis` may be placed on `route`.
///
/// # Errors
/// [`CardError::RightsRouteRefused`] when the basis does not open the route — `none` on
/// any route, `promo_all_media` and `user_private_copy` on a professional one. (The
/// composer itself never asks for `none`: a cover without a rights basis is dropped and
/// the typographic card is produced.)
pub fn placement(basis: RightsBasis, route: PrintRoute) -> Result<Placement, CardError> {
    basis
        .gate(route)
        .map(|grant| grant.placement)
        .map_err(|_| CardError::RightsRouteRefused { basis, route })
}

/// `true` when the basis allows artwork to be *adapted* (a spine sampled from the cover,
/// a provided spine re-fitted) once the request also says `adaptation_authorised`: only
/// `print_licence` and `rights_holder_declared`.
#[must_use]
pub const fn allows_adaptation(basis: RightsBasis) -> bool {
    matches!(basis, RightsBasis::PrintLicence | RightsBasis::RightsHolderDeclared)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_gate_is_the_table_of_the_module_documentation() {
        use Placement::{Full, ReducedSize};
        use RightsBasis as B;
        let home = |b: B| placement(b, PrintRoute::HomePrint).ok();
        let pro = |b: B| placement(b, PrintRoute::ProfessionalOrder).ok();
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
            placement(B::UserPrivateCopy, PrintRoute::ProfessionalOrder),
            Err(CardError::RightsRouteRefused {
                basis: B::UserPrivateCopy,
                route: PrintRoute::ProfessionalOrder
            })
        ));
        // The gate agrees with the predicates of `ebc-covers` for every basis.
        for basis in B::ALL {
            assert_eq!(home(basis).is_some(), basis.allows_home_print(), "{basis}");
            assert_eq!(pro(basis).is_some(), basis.allows_professional_print(), "{basis}");
        }
        assert_eq!(B::default(), B::None);
    }

    #[test]
    fn routes_follow_the_profile() {
        use ebc_layout::Sheet;
        assert_eq!(route_of(&PrintProfile::professional_eu()), PrintRoute::ProfessionalOrder);
        assert_eq!(route_of(&PrintProfile::upload_eu()), PrintRoute::ProfessionalOrder);
        assert_eq!(route_of(&PrintProfile::home(Sheet::A4)), PrintRoute::HomePrint);
        assert_eq!(route_of(&PrintProfile::home(Sheet::Letter)), PrintRoute::HomePrint);
    }

    #[test]
    fn only_licensed_bases_allow_adaptation() {
        let allowed: Vec<_> =
            RightsBasis::ALL.into_iter().filter(|b| allows_adaptation(*b)).collect();
        assert_eq!(allowed, [RightsBasis::PrintLicence, RightsBasis::RightsHolderDeclared]);
    }
}
