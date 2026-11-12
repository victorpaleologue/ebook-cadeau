//! The product catalogue of the fictional printer.
//!
//! Every product is a "book jacket" card printed flat on both sides and creased twice:
//! `back cover | spine | front cover`, i.e. a trim width of `2 × cover width + spine`.
//!
//! **Prices are demonstration placeholders.** They are anchored on the orders of magnitude
//! collected in `docs/research/notes/printers.web2print.md` (a single print-on-demand card
//! lands around 6 € with postage; a batch of 100 folded cards costs 36–60 € before tax at
//! mass web-to-print shops, more for a larger, twice-creased card) but they are not quotes
//! from any real printer.

use serde::Serialize;
use serde::ser::SerializeMap;

/// Countries the fictional printer posts to (ISO 3166-1 alpha-2).
pub const SHIPS_TO: &[&str] = &["FR", "BE", "LU", "DE", "NL", "ES", "IT", "PT", "AT", "IE"];

/// A price ladder: `(quantity, total price in cents for that quantity)`, ascending.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PriceLadder(pub &'static [(u32, u32)]);

impl Serialize for PriceLadder {
    /// Serialises as the JSON object of `docs/api.md` §5: `{"1": 590, "10": 2400}`.
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (quantity, cents) in self.0 {
            map.serialize_entry(&quantity.to_string(), cents)?;
        }
        map.end()
    }
}

/// One product of the catalogue.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Product {
    /// Identifier quoted in orders, e.g. `card-poche`.
    pub id: &'static str,
    /// Display name (French).
    pub name: &'static str,
    /// Trim size of the flat spread, `[width, height]` in millimetres.
    pub trim_mm: [f64; 2],
    /// Size of one cover panel, `[width, height]` in millimetres.
    pub cover_mm: [f64; 2],
    /// Bleed required on every side, in millimetres.
    pub bleed_mm: f64,
    /// Width of the spine panel between the two creases, in millimetres.
    pub spine_mm: f64,
    /// Number of pages expected in the PDF (outside spread, inside spread).
    pub pages: u32,
    /// Paper description.
    pub paper: &'static str,
    /// Printing process.
    pub process: &'static str,
    /// The PDF standard the file should follow.
    pub pdf_standard: &'static str,
    /// When `true`, RGB content and transparency are refused (offset workflow); when
    /// `false`, RGB images are accepted and converted by the printer (digital workflow).
    pub cmyk_only: bool,
    /// Whether a PDF/X output intent is required.
    pub output_intent_required: bool,
    /// Minimum effective image resolution, in pixels per inch.
    pub min_ppi: f64,
    /// Total ink limit, in percent.
    pub max_total_ink_percent: f32,
    /// Smallest quantity that can be ordered.
    pub min_quantity: u32,
    /// Price ladder: quantity → total price in cents for that quantity, delivery included.
    pub price_cents: PriceLadder,
    /// Currency of the prices.
    pub currency: &'static str,
    /// What the price covers.
    pub price_includes: &'static str,
    /// Honest label: these are not real quotes.
    pub price_basis: &'static str,
    /// Countries delivered.
    pub ships_to: &'static [&'static str],
}

impl Product {
    /// Total price in cents for `quantity` copies: the unit price of the highest rung not
    /// above `quantity`, multiplied by `quantity` and rounded up to a cent. `None` below the
    /// minimum quantity or when the result does not fit.
    #[must_use]
    pub fn price_for(&self, quantity: u32) -> Option<u32> {
        if quantity < self.min_quantity || quantity == 0 {
            return None;
        }
        let (rung, total) =
            self.price_cents.0.iter().copied().rfind(|(q, _)| *q > 0 && *q <= quantity)?;
        u32::try_from((u64::from(total) * u64::from(quantity)).div_ceil(u64::from(rung))).ok()
    }
}

const PRICE_INCLUDES: &str = "impression recto verso, deux rainages, livraison à une adresse (lettre suivie en France), TVA 20 %";
const PRICE_BASIS: &str = "prix de démonstration, indicatifs : ordres de grandeur relevés dans docs/research, pas des devis";
const DIGITAL_PAPER: &str = "Carte couchée demi-mate 350 g/m², fibres issues de forêts gérées durablement (mention fictive), sans pelliculage";

const fn digital(
    id: &'static str,
    name: &'static str,
    cover_mm: [f64; 2],
    price_cents: &'static [(u32, u32)],
) -> Product {
    Product {
        id,
        name,
        trim_mm: [2.0 * cover_mm[0] + SPINE_MM, cover_mm[1]],
        cover_mm,
        bleed_mm: BLEED_MM,
        spine_mm: SPINE_MM,
        pages: 2,
        paper: DIGITAL_PAPER,
        process: "numérique, à l'unité",
        pdf_standard: "PDF/X-4",
        cmyk_only: false,
        output_intent_required: true,
        min_ppi: 150.0,
        max_total_ink_percent: 300.0,
        min_quantity: 1,
        price_cents: PriceLadder(price_cents),
        currency: "EUR",
        price_includes: PRICE_INCLUDES,
        price_basis: PRICE_BASIS,
        ships_to: SHIPS_TO,
    }
}

/// Bleed asked by every product, in millimetres.
pub const BLEED_MM: f64 = 3.0;
/// Spine width of every product, in millimetres.
pub const SPINE_MM: f64 = 10.0;

static PRODUCTS: [Product; 5] = [
    digital(
        "card-poche",
        "Carte jaquette format poche (2 × 110 × 178 mm + dos 10 mm)",
        [110.0, 178.0],
        &[(1, 590), (10, 2400), (25, 4500), (50, 6500), (100, 8900)],
    ),
    digital(
        "card-a6",
        "Carte jaquette format A6 (2 × 105 × 148 mm + dos 10 mm)",
        [105.0, 148.0],
        &[(1, 490), (10, 2000), (25, 3800), (50, 5500), (100, 7500)],
    ),
    digital(
        "card-digest",
        "Carte jaquette format broché (2 × 130 × 200 mm + dos 10 mm)",
        [130.0, 200.0],
        &[(1, 690), (10, 2900), (25, 5400), (50, 7800), (100, 10_900)],
    ),
    digital(
        "card-a5",
        "Carte jaquette format A5 (2 × 148 × 210 mm + dos 10 mm)",
        [148.0, 210.0],
        &[(1, 790), (10, 3300), (25, 6200), (50, 8900), (100, 12_500)],
    ),
    Product {
        id: "card-poche-offset",
        name: "Carte jaquette format poche, tirage offset (à partir de 250 exemplaires)",
        trim_mm: [230.0, 178.0],
        cover_mm: [110.0, 178.0],
        bleed_mm: BLEED_MM,
        spine_mm: SPINE_MM,
        pages: 2,
        paper: "Carte couchée demi-mate 350 g/m², fibres issues de forêts gérées durablement (mention fictive), pelliculage mat recto",
        process: "offset quadri recto verso",
        pdf_standard: "PDF/X-1a",
        cmyk_only: true,
        output_intent_required: true,
        min_ppi: 150.0,
        max_total_ink_percent: 300.0,
        min_quantity: 250,
        price_cents: PriceLadder(&[(250, 12_900), (500, 16_900), (1000, 22_900)]),
        currency: "EUR",
        price_includes: "impression recto verso, deux rainages, livraison à une adresse, TVA 20 %",
        price_basis: PRICE_BASIS,
        ships_to: SHIPS_TO,
    },
];

/// The whole catalogue.
#[must_use]
pub fn products() -> &'static [Product] {
    &PRODUCTS
}

/// Looks a product up by its identifier.
#[must_use]
pub fn find(id: &str) -> Option<&'static Product> {
    PRODUCTS.iter().find(|p| p.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_four_card_formats_are_offered_with_the_documented_geometry() {
        let expect = [
            ("card-poche", [230.0, 178.0]),
            ("card-a6", [220.0, 148.0]),
            ("card-digest", [270.0, 200.0]),
            ("card-a5", [306.0, 210.0]),
        ];
        for (id, trim) in expect {
            let p = find(id).unwrap_or_else(|| panic!("{id} missing"));
            assert_eq!(p.trim_mm, trim, "{id}");
            assert_eq!(p.trim_mm[0], 2.0 * p.cover_mm[0] + p.spine_mm);
            assert_eq!((p.bleed_mm, p.spine_mm, p.pages), (3.0, 10.0, 2));
            assert!(!p.cmyk_only && p.output_intent_required && p.min_ppi == 150.0);
            assert!(p.paper.contains("350 g/m²"));
            assert_eq!(p.min_quantity, 1);
        }
        assert!(find("card-poche-offset").unwrap().cmyk_only);
        assert!(find("poster").is_none());
        let ids: std::collections::HashSet<_> = products().iter().map(|p| p.id).collect();
        assert_eq!(ids.len(), products().len());
    }

    #[test]
    fn ladders_are_ascending_and_unit_prices_decrease() {
        for p in products() {
            let ladder = p.price_cents.0;
            assert!(!ladder.is_empty());
            assert_eq!(ladder[0].0, p.min_quantity, "{}: first rung is the minimum", p.id);
            for pair in ladder.windows(2) {
                let ((q0, t0), (q1, t1)) = (pair[0], pair[1]);
                assert!(q1 > q0 && t1 > t0, "{}: ladder must ascend", p.id);
                assert!(
                    u64::from(t1) * u64::from(q0) < u64::from(t0) * u64::from(q1),
                    "{}: unit price must fall with quantity",
                    p.id
                );
            }
        }
    }

    #[test]
    fn prices_follow_the_ladder() {
        let p = find("card-poche").unwrap();
        assert_eq!(p.price_for(1), Some(590));
        assert_eq!(p.price_for(3), Some(1770));
        assert_eq!(p.price_for(10), Some(2400));
        assert_eq!(p.price_for(11), Some(2640));
        assert_eq!(p.price_for(99), Some(12_870));
        assert_eq!(p.price_for(100), Some(8900));
        assert_eq!(p.price_for(0), None);
        assert_eq!(p.price_for(u32::MAX), None, "an absurd quantity does not overflow");
        let offset = find("card-poche-offset").unwrap();
        assert_eq!(offset.price_for(1), None);
        assert_eq!(offset.price_for(249), None);
        assert_eq!(offset.price_for(250), Some(12_900));
        assert_eq!(offset.price_for(300), Some(15_480));
    }

    #[test]
    fn json_shape_matches_the_api_document() {
        let value = serde_json::to_value(find("card-poche").unwrap()).unwrap();
        assert_eq!(value["id"], "card-poche");
        assert_eq!(value["trim_mm"], serde_json::json!([230.0, 178.0]));
        assert_eq!(value["bleed_mm"], 3.0);
        assert_eq!(value["spine_mm"], 10.0);
        assert_eq!(value["price_cents"]["1"], 590);
        assert_eq!(value["price_cents"]["10"], 2400);
        assert!(value["paper"].as_str().unwrap().starts_with("Carte couchée"));
        assert!(value["price_basis"].as_str().unwrap().contains("démonstration"));
    }
}
