//! The rights gate on real files, placeholder detection on real bytes, and the
//! quality → upscale path on decoded pixels.

mod common;

use common::*;
use ebc_covers::placeholder::{Reason, Suspicion, assess, assess_bytes, assess_pixels, PixelThresholds};
use ebc_covers::quality::Thresholds;
use ebc_covers::sniff::{Format, probe};
use ebc_covers::{
    AssetOrigin, Condition, CoversError, Placement, PrintRoute, PrintableCover, RightsBasis, UpscaleOptions,
    Verdict, is_placeholder, is_placeholder_pixels, print_quality, print_quality_with, upscale,
};
use ebc_scene::Size;
use jpeg_encoder::ColorType;

const POCHE: Size = Size::new(110.0, 178.0);

#[test]
fn printable_cover_needs_a_basis_an_image_and_a_matching_origin() {
    let cover = png_rgb(300, 450);
    // No basis, no printable cover — whatever the origin.
    for origin in [AssetOrigin::ResellerFeed, AssetOrigin::LocalFile, AssetOrigin::LocalEpub, AssetOrigin::RightsHolderUpload, AssetOrigin::OperatorDownload] {
        assert_eq!(PrintableCover::new(cover.clone(), RightsBasis::None, origin), Err(CoversError::NoRightsBasis));
        // Every basis the origin admits works; every other one is a mismatch.
        for basis in RightsBasis::ALL {
            let result = PrintableCover::new(cover.clone(), basis, origin);
            if basis == RightsBasis::None {
                continue;
            }
            if origin.compatible_bases().contains(&basis) {
                let asset = result.expect("compatible");
                assert_eq!((asset.basis(), asset.origin()), (basis, origin));
                assert_eq!(asset.bytes(), &cover[..]);
            } else {
                assert_eq!(result, Err(CoversError::OriginMismatch { origin, basis }));
            }
        }
    }
    // A private copy can only come from the user's own file, never from a download or a feed.
    assert!(PrintableCover::new(cover.clone(), RightsBasis::UserPrivateCopy, AssetOrigin::OperatorDownload).is_err());
    assert!(PrintableCover::new(cover.clone(), RightsBasis::UserPrivateCopy, AssetOrigin::ResellerFeed).is_err());
    // Not an image; a placeholder.
    let html = b"<html><body>Image not available</body></html>".repeat(10);
    assert_eq!(PrintableCover::new(html, RightsBasis::PublicDomain, AssetOrigin::LocalFile), Err(CoversError::NotAnImage));
    assert_eq!(
        PrintableCover::new(png_rgb(1, 1), RightsBasis::PublicDomain, AssetOrigin::LocalFile),
        Err(CoversError::Placeholder)
    );
    assert_eq!(
        PrintableCover::new(png_rgb(60, 90), RightsBasis::RightsHolderDeclared, AssetOrigin::RightsHolderUpload),
        Err(CoversError::Placeholder)
    );
}

#[test]
fn printable_cover_routes() {
    let cover = png_rgb(300, 450);
    let private = PrintableCover::new(cover.clone(), RightsBasis::UserPrivateCopy, AssetOrigin::LocalEpub).expect("asset");
    let grant = private.gate(PrintRoute::HomePrint).expect("home print");
    assert_eq!(grant.placement, Placement::Full);
    assert!(grant.requires(Condition::OwnPrinterOnly) && grant.requires(Condition::OneJacketPerGiftCode));
    assert_eq!(
        private.gate(PrintRoute::ProfessionalOrder),
        Err(CoversError::RouteNotAllowed { basis: RightsBasis::UserPrivateCopy, route: PrintRoute::ProfessionalOrder })
    );

    let promo = PrintableCover::new(cover.clone(), RightsBasis::PromoAllMedia, AssetOrigin::ResellerFeed).expect("asset");
    assert_eq!(promo.gate(PrintRoute::HomePrint).expect("home").placement, Placement::ReducedSize);
    assert!(promo.gate(PrintRoute::ProfessionalOrder).is_err());

    let licensed = PrintableCover::new(cover.clone(), RightsBasis::PrintLicence, AssetOrigin::ResellerFeed).expect("asset");
    for route in [PrintRoute::HomePrint, PrintRoute::ProfessionalOrder] {
        assert_eq!(licensed.gate(route).expect("licensed").placement, Placement::Full);
    }
    let declared = PrintableCover::new(cover, RightsBasis::RightsHolderDeclared, AssetOrigin::RightsHolderUpload).expect("asset");
    assert!(declared.gate(PrintRoute::ProfessionalOrder).expect("order").requires(Condition::DeclarantIdentified));
    // Error messages name the basis by its machine name.
    let message = private.gate(PrintRoute::ProfessionalOrder).expect_err("refused").to_string();
    assert!(message.contains("user_private_copy") && message.contains("professional print order"), "{message}");
}

#[test]
fn printable_cover_reports_the_displayed_size() {
    // A landscape JPEG whose EXIF orientation says "rotate 90°" is a portrait cover.
    let bytes = jpeg_custom(600, 400, ColorType::Rgb, false, None, Some(&tiff_orientation(6, true)), 0);
    let asset = PrintableCover::new(bytes, RightsBasis::RightsHolderDeclared, AssetOrigin::LocalFile).expect("asset");
    assert_eq!((asset.info().format, asset.info().width, asset.info().height), (Format::Jpeg, 400, 600));
    let q = print_quality(&asset.info(), POCHE);
    assert!((q.placed_mm.w - 110.0).abs() < 1e-9, "the width binds for a 2:3 image on a poche front");
    let _ = asset.into_bytes();
}

#[test]
fn placeholders_on_real_bytes() {
    // The classic 1 x 1 GIF.
    let pixel = gif(1, 1);
    let info = probe(&pixel).expect("gif");
    assert!(is_placeholder(&info, &pixel));
    let reasons = assess(&info, &pixel).reasons;
    assert!(reasons.contains(&Reason::OnePixel));
    // A thumbnail-sized icon.
    let icon = png_rgb(64, 64);
    assert!(is_placeholder(&probe(&icon).expect("png"), &icon));
    // A real-looking cover passes every byte-level check.
    let cover = jpeg_baseline(400, 600);
    assert!(assess_bytes(&cover).is_clean(), "{:?}", assess_bytes(&cover));
    // A big flat tile is not rejected on bytes alone, but it is suspicious (it
    // compresses to almost nothing) and the pixel check rejects it.
    let tile = jpeg_flat(400, 600, 204);
    let bytes_level = assess_bytes(&tile);
    assert!(!bytes_level.is_placeholder());
    assert!(
        bytes_level.suspicions.iter().any(|s| matches!(s, Suspicion::LowBitsPerPixel { .. })),
        "{bytes_level:?} for {} bytes",
        tile.len()
    );
    assert!(is_placeholder_pixels(&decode(&tile)));
    // (A flat PNG from a fast encoder is several kilobytes and escapes the byte-level
    // hint; the pixel check is the one that decides.)
    assert!(is_placeholder_pixels(&decode(&png_flat(400, 600, 204))));
    assert!(!is_placeholder_pixels(&decode(&cover)));
    // An animated GIF of cover size is flagged, not rejected.
    let animated = gif_animated(300, 450);
    let a = assess_bytes(&animated);
    assert!(a.suspicions.contains(&Suspicion::Animated), "{a:?}");
    // An HTML error page served as an image.
    assert!(assess_bytes(b"<!doctype html><title>404</title>").reasons.contains(&Reason::NotAnImage));
    // A gallery hash recognises the same picture in another container.
    let hash = ebc_covers::placeholder::dhash(&decode(&cover));
    let again = decode(&encode(&image::load_from_memory(&cover).expect("decode"), image::ImageFormat::Png));
    let hit = assess_pixels(&again, &PixelThresholds::default(), &[hash]);
    assert!(hit.reasons.iter().any(|r| matches!(r, Reason::KnownHash { distance } if *distance <= 2)));
}

#[test]
fn quality_then_upscale_on_decoded_pixels() {
    // An Open Library "-L"-sized cover: 325 x 500 px on a poche front.
    let bytes = jpeg_baseline(325, 500);
    let info = probe(&bytes).expect("jpeg");
    let q = print_quality(&info, POCHE);
    assert_eq!(q.verdict, Verdict::Insufficient);
    assert!(q.effective_ppi > 70.0 && q.effective_ppi < 76.0, "{}", q.effective_ppi);
    assert!(q.upscale_factor > 3.9 && q.upscale_factor < 4.3);
    // Even the home thresholds call it insufficient.
    assert_eq!(print_quality_with(&info, POCHE, &Thresholds::HOME).verdict, Verdict::Insufficient);

    let pixels = decode(&bytes);
    let up = upscale(&pixels, q.target_px.0, q.target_px.1, &UpscaleOptions::mild_sharpen()).expect("upscale");
    assert_eq!((up.width(), up.height()), q.target_px);
    assert!(up.is_opaque());
    // Resampling is not print quality: the rating is still made on the native pixels,
    // and the upscaled grid merely reaches the target size.
    let after = print_quality(&ebc_covers::ImageInfo::new(Format::Jpeg, up.width(), up.height()), POCHE);
    assert_eq!(after.verdict, Verdict::Excellent);
    assert_eq!(q.verdict, Verdict::Insufficient);
    // The average colour is preserved by the resampling (no brightness drift).
    let mean = |p: &ebc_scene::Pixmap| {
        let sum: u64 = p.data().chunks_exact(4).map(|px| u64::from(px[0]) + u64::from(px[1]) + u64::from(px[2])).sum();
        sum as f64 / (f64::from(p.width()) * f64::from(p.height()) * 3.0)
    };
    assert!((mean(&pixels) - mean(&up)).abs() < 1.5, "{} vs {}", mean(&pixels), mean(&up));
}
