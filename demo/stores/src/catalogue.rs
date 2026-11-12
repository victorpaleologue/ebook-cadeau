//! The demo catalogue: six French classics in the public domain, in invented "demo
//! editions" shared by both stores.
//!
//! The ISBNs are **demonstration identifiers**: syntactically valid ISBN-13s (correct EAN-13
//! check digit) taken from the range `978-66…` to `978-69…`, which the International ISBN
//! Agency has not allocated to any registration group, so they cannot collide with a real
//! book. The texts are abridged openings written from memory of the public-domain originals;
//! they are test content, not a critical edition.

use crate::covers::{CoverDesign, Frame, Ornament, TitleFace};

/// The fictional imprint named in the EPUB metadata and sent as `publisher` to the API.
pub const PUBLISHER: &str = "Éditions du Domaine public (démonstration)";

/// Label shown next to every ISBN.
pub const ISBN_LABEL: &str = "ISBN de démonstration";

/// One title of the catalogue.
#[derive(Debug)]
pub struct Book {
    /// URL-safe identifier, e.g. `candide`.
    pub id: &'static str,
    /// Main title.
    pub title: &'static str,
    /// Volume or part, when the demo edition is one tome of a longer work.
    pub subtitle: Option<&'static str>,
    /// Author's name as printed.
    pub author: &'static str,
    /// Year of first publication.
    pub year: u16,
    /// Demonstration ISBN-13 (digits only).
    pub isbn13: &'static str,
    /// Price in euro cents, VAT included.
    pub price_cents: u32,
    /// Short description shown on the product page.
    pub description: &'static str,
    /// Heading of the single chapter in the demo EPUB.
    pub chapter_title: &'static str,
    /// Opening paragraphs (abridged). A `\n` inside a paragraph is a line break (verse).
    pub paragraphs: &'static [&'static str],
    /// The typographic cover.
    pub cover: CoverDesign,
}

impl Book {
    /// Title and subtitle joined, as sent to the partner API and written in the EPUB.
    #[must_use]
    pub fn full_title(&self) -> String {
        match self.subtitle {
            Some(sub) => format!("{} — {sub}", self.title),
            None => self.title.to_owned(),
        }
    }

    /// File name offered for downloads, ASCII only: `candide.epub`.
    #[must_use]
    pub fn file_name(&self) -> String {
        format!("{}.epub", self.id)
    }
}

/// The whole catalogue, in display order.
#[must_use]
pub fn books() -> &'static [Book] {
    &BOOKS
}

/// Looks a book up by its identifier.
#[must_use]
pub fn find(id: &str) -> Option<&'static Book> {
    BOOKS.iter().find(|b| b.id == id)
}

/// Formats euro cents the French way: `2,99 €`.
#[must_use]
pub fn format_price(cents: u32) -> String {
    format!("{},{:02}\u{a0}€", cents / 100, cents % 100)
}

static BOOKS: [Book; 6] = [
    Book {
        id: "candide",
        title: "Candide",
        subtitle: None,
        author: "Voltaire",
        year: 1759,
        isbn13: "9786990000015",
        price_cents: 299,
        description: "Chassé du plus beau des châteaux pour un baiser, Candide parcourt un monde \
                      de guerres, de naufrages et de tremblements de terre en répétant que tout \
                      est pour le mieux. Un conte philosophique vif, drôle et féroce.",
        chapter_title: "Chapitre premier",
        paragraphs: &[
            "Il y avait en Westphalie, dans le château de monsieur le baron de \
             Thunder-ten-tronckh, un jeune garçon à qui la nature avait donné les mœurs les \
             plus douces. Sa physionomie annonçait son âme. Il avait le jugement assez droit, \
             avec l'esprit le plus simple ; c'est, je crois, pour cette raison qu'on le nommait \
             Candide.",
            "Les anciens domestiques de la maison soupçonnaient qu'il était fils de la sœur de \
             monsieur le baron et d'un bon et honnête gentilhomme du voisinage, que cette \
             demoiselle ne voulut jamais épouser parce qu'il n'avait pu prouver que soixante et \
             onze quartiers, et que le reste de son arbre généalogique avait été perdu par \
             l'injure du temps.",
            "Monsieur le baron était un des plus puissants seigneurs de la Westphalie, car son \
             château avait une porte et des fenêtres. Sa grande salle même était ornée d'une \
             tapisserie.",
        ],
        cover: CoverDesign {
            ground: "#1f4d3a",
            ink: "#f4ecd8",
            accent: "#d9b45a",
            frame: Frame::Double,
            title_face: TitleFace::Serif,
            title_lines: &["Candide"],
            title_size: 190,
            ornament: Ornament::Rule,
        },
    },
    Book {
        id: "les-miserables-1",
        title: "Les Misérables",
        subtitle: Some("Tome 1 : Fantine"),
        author: "Victor Hugo",
        year: 1862,
        isbn13: "9786990000022",
        price_cents: 399,
        description: "Un évêque qui ouvre sa porte, un forçat libéré que tout le monde chasse : \
                      le premier tome du grand roman de Victor Hugo suit Jean Valjean de Digne à \
                      Montreuil-sur-Mer, et Fantine jusqu'au bout de sa misère.",
        chapter_title: "Livre premier — Un juste. I. Monsieur Myriel",
        paragraphs: &[
            "En 1815, M. Charles-François-Bienvenu Myriel était évêque de Digne. C'était un \
             vieillard d'environ soixante-quinze ans ; il occupait le siège de Digne depuis 1806.",
            "Quoique ce détail ne touche en aucune manière au fond même de ce que nous avons à \
             raconter, il n'est peut-être pas inutile, ne fût-ce que pour être exact en tout, \
             d'indiquer ici les bruits et les propos qui avaient couru sur son compte au moment \
             où il était arrivé dans le diocèse.",
            "Vrai ou faux, ce qu'on dit des hommes tient souvent autant de place dans leur vie \
             et surtout dans leur destinée que ce qu'ils font.",
        ],
        cover: CoverDesign {
            ground: "#7a1f1f",
            ink: "#ffffff",
            accent: "#f0c9a0",
            frame: Frame::None,
            title_face: TitleFace::Display,
            title_lines: &["LES", "MISÉRABLES"],
            title_size: 230,
            ornament: Ornament::Band,
        },
    },
    Book {
        id: "monte-cristo-1",
        title: "Le Comte de Monte-Cristo",
        subtitle: Some("Tome 1"),
        author: "Alexandre Dumas",
        year: 1844,
        isbn13: "9786990000039",
        price_cents: 399,
        description: "Marseille, 1815. Le jour de ses fiançailles, le jeune marin Edmond Dantès \
                      est dénoncé et jeté au château d'If. Le premier tome d'un roman \
                      d'aventures et de vengeance qui ne se lâche pas.",
        chapter_title: "I. Marseille. — L'arrivée",
        paragraphs: &[
            "Le 24 février 1815, la vigie de Notre-Dame de la Garde signala le trois-mâts le \
             Pharaon, venant de Smyrne, Trieste et Naples.",
            "Comme d'habitude, un pilote côtier partit aussitôt du port, rasa le château d'If, \
             et alla aborder le navire entre le cap de Morgion et l'île de Rion.",
            "Aussitôt, comme d'habitude encore, la plate-forme du fort Saint-Jean s'était \
             couverte de curieux ; car c'est toujours une grande affaire à Marseille que \
             l'arrivée d'un bâtiment, surtout quand ce bâtiment, comme le Pharaon, a été \
             construit, gréé, arrimé sur les chantiers de la vieille Phocée, et appartient à un \
             armateur de la ville.",
        ],
        cover: CoverDesign {
            ground: "#14213d",
            ink: "#f2e2b0",
            accent: "#e0b040",
            frame: Frame::Single,
            title_face: TitleFace::SerifItalic,
            title_lines: &["Le Comte", "de", "Monte-Cristo"],
            title_size: 140,
            ornament: Ornament::Diamonds,
        },
    },
    Book {
        id: "madame-bovary",
        title: "Madame Bovary",
        subtitle: None,
        author: "Gustave Flaubert",
        year: 1857,
        isbn13: "9786990000046",
        price_cents: 299,
        description: "Emma rêvait de bals et de passions ; elle a épousé un officier de santé \
                      normand. Le roman qui valut un procès à son auteur, et la phrase la plus \
                      travaillée de la littérature française.",
        chapter_title: "Première partie — I",
        paragraphs: &[
            "Nous étions à l'Étude, quand le Proviseur entra, suivi d'un nouveau habillé en \
             bourgeois et d'un garçon de classe qui portait un grand pupitre. Ceux qui dormaient \
             se réveillèrent, et chacun se leva comme surpris dans son travail.",
            "Le Proviseur nous fit signe de nous rasseoir ; puis, se tournant vers le maître \
             d'études :",
            "— Monsieur Roger, lui dit-il à demi-voix, voici un élève que je vous recommande, il \
             entre en cinquième. Si son travail et sa conduite sont méritoires, il passera dans \
             les grands, où l'appelle son âge.",
        ],
        cover: CoverDesign {
            ground: "#f3ead8",
            ink: "#2b2118",
            accent: "#9c3d2e",
            frame: Frame::Single,
            title_face: TitleFace::Serif,
            title_lines: &["Madame", "Bovary"],
            title_size: 170,
            ornament: Ornament::Rule,
        },
    },
    Book {
        id: "les-fleurs-du-mal",
        title: "Les Fleurs du mal",
        subtitle: None,
        author: "Charles Baudelaire",
        year: 1857,
        isbn13: "9786990000053",
        price_cents: 199,
        description: "Spleen et idéal, tableaux parisiens, le vin, la révolte, la mort : le \
                      recueil qui a fait entrer la ville moderne et l'ennui dans la poésie.",
        chapter_title: "Au lecteur",
        paragraphs: &[
            "La sottise, l'erreur, le péché, la lésine,\nOccupent nos esprits et travaillent nos \
             corps,\nEt nous alimentons nos aimables remords,\nComme les mendiants nourrissent \
             leur vermine.",
            "Nos péchés sont têtus, nos repentirs sont lâches ;\nNous nous faisons payer \
             grassement nos aveux,\nEt nous rentrons gaiement dans le chemin bourbeux,\nCroyant \
             par de vils pleurs laver toutes nos taches.",
            "C'est l'Ennui ! — l'œil chargé d'un pleur involontaire,\nIl rêve d'échafauds en \
             fumant son houka.\nTu le connais, lecteur, ce monstre délicat,\n— Hypocrite \
             lecteur, — mon semblable, — mon frère !",
        ],
        cover: CoverDesign {
            ground: "#2a1a3a",
            ink: "#f5d7e3",
            accent: "#c2456b",
            frame: Frame::None,
            title_face: TitleFace::Sans,
            title_lines: &["Les Fleurs", "du mal"],
            title_size: 150,
            ornament: Ornament::Discs,
        },
    },
    Book {
        id: "le-horla",
        title: "Le Horla",
        subtitle: None,
        author: "Guy de Maupassant",
        year: 1887,
        isbn13: "9786990000060",
        price_cents: 99,
        description: "Un homme tient son journal au bord de la Seine. Quelque chose boit son \
                      eau la nuit. La nouvelle fantastique la plus célèbre de Maupassant, ou le \
                      récit minutieux d'une raison qui cède.",
        chapter_title: "8 mai",
        paragraphs: &[
            "8 mai. — Quelle journée admirable ! J'ai passé toute la matinée étendu sur \
             l'herbe, devant ma maison, sous l'énorme platane qui la couvre, l'abrite et \
             l'ombrage tout entière.",
            "J'aime ce pays, et j'aime y vivre parce que j'y ai mes racines, ces profondes et \
             délicates racines, qui attachent un homme à la terre où sont nés et morts ses \
             aïeux, qui l'attachent à ce qu'on pense et à ce qu'on mange, aux usages comme aux \
             nourritures, aux locutions locales, aux intonations des paysans, aux odeurs du \
             sol, des villages et de l'air lui-même.",
            "J'aime ma maison où j'ai grandi. De mes fenêtres, je vois la Seine qui coule, le \
             long de mon jardin, derrière la route, presque chez moi, la grande et large Seine, \
             qui va de Rouen au Havre, couverte de bateaux qui passent.",
        ],
        cover: CoverDesign {
            ground: "#ffffff",
            ink: "#111111",
            accent: "#111111",
            frame: Frame::None,
            title_face: TitleFace::Sans,
            title_lines: &["Le Horla"],
            title_size: 170,
            ornament: Ornament::Eclipse,
        },
    },
];

/// Whether `digits` is a syntactically valid ISBN-13: 13 ASCII digits, prefix 978 or 979,
/// correct EAN-13 check digit.
#[must_use]
pub fn is_valid_isbn13(digits: &str) -> bool {
    let bytes = digits.as_bytes();
    if bytes.len() != 13 || !bytes.iter().all(u8::is_ascii_digit) {
        return false;
    }
    if !(digits.starts_with("978") || digits.starts_with("979")) {
        return false;
    }
    let sum: u32 = bytes
        .iter()
        .enumerate()
        .map(|(i, b)| u32::from(b - b'0') * if i % 2 == 0 { 1 } else { 3 })
        .sum();
    sum % 10 == 0
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn six_books_with_unique_ids_and_isbns() {
        assert_eq!(books().len(), 6);
        let ids: HashSet<_> = books().iter().map(|b| b.id).collect();
        let isbns: HashSet<_> = books().iter().map(|b| b.isbn13).collect();
        assert_eq!(ids.len(), 6);
        assert_eq!(isbns.len(), 6);
        for b in books() {
            assert!(
                b.id.bytes().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-'),
                "{} must be URL-safe",
                b.id
            );
            assert_eq!(find(b.id).map(|f| f.isbn13), Some(b.isbn13));
            assert!(b.price_cents > 0 && b.price_cents < 2_000);
            assert!(!b.description.is_empty() && b.paragraphs.len() >= 2);
            assert!(b.year < 1900, "public domain: {}", b.title);
        }
        assert!(find("nope").is_none());
        assert!(find("").is_none());
    }

    #[test]
    fn isbns_are_valid_and_outside_any_allocated_group() {
        for b in books() {
            assert!(is_valid_isbn13(b.isbn13), "{}", b.isbn13);
            // 978-66 … 978-69 is not allocated to any registration group.
            let group = &b.isbn13[3..5];
            assert!(("66"..="69").contains(&group), "{} is in an allocated range", b.isbn13);
        }
    }

    #[test]
    fn isbn_validation_rejects_near_misses() {
        assert!(is_valid_isbn13("9782070360024"));
        assert!(!is_valid_isbn13("9782070360025"), "wrong check digit");
        assert!(!is_valid_isbn13("978207036002"), "too short");
        assert!(!is_valid_isbn13("97820703600240"), "too long");
        assert!(!is_valid_isbn13("978207036002X"));
        assert!(!is_valid_isbn13("4006381333931"), "valid EAN-13 but not Bookland");
        assert!(!is_valid_isbn13("９７８２０７０３６００２４"), "full-width digits");
        assert!(!is_valid_isbn13(""));
    }

    #[test]
    fn prices_and_titles_format() {
        assert_eq!(format_price(299), "2,99\u{a0}€");
        assert_eq!(format_price(5), "0,05\u{a0}€");
        assert_eq!(format_price(1000), "10,00\u{a0}€");
        let hugo = find("les-miserables-1").unwrap();
        assert_eq!(hugo.full_title(), "Les Misérables — Tome 1 : Fantine");
        assert_eq!(hugo.file_name(), "les-miserables-1.epub");
        assert_eq!(find("candide").unwrap().full_title(), "Candide");
    }
}
