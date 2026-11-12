//! The strings of the web app, in the seven supported languages.
//!
//! [`i18n_catalog`] merges two sources into one `{key: text}` object per locale:
//!
//! * every message of `ebc-i18n` under its own key (`field_isbn`, `lookup_cta`, …) —
//!   the words printed on the card and the creator's basic labels;
//! * the strings only this page needs ([`UI_STRINGS`]: keys starting with `ui_`,
//!   `warn_` or `err_`), kept here so that the page itself holds no text and a test can
//!   check that every language has every string with the same placeholders.
//!
//! Placeholders are `{name}`; the page substitutes them in a single pass and inserts the
//! result as text, never as markup.
//!
//! **Not reviewed by native speakers.** The strings follow the wording rules of
//! `ebc-i18n` (no stored-value words such as "gift card" or "voucher"; "code", never
//! "QR"; French *vous*) and of the legal research: the page never presents itself as a
//! way to print a book's cover — it offers the typographic card, or an image *the user
//! may print*.

use std::collections::BTreeMap;

use ebc_i18n::{Locale, Msg, t};

use super::{ApiError, to_json};

/// Number of supported locales; the columns of [`UI_STRINGS`] follow [`Locale::ALL`]:
/// fr, en, de, es, it, nl, pt.
pub const LOCALES: usize = 7;

/// The page's own strings: key, then one text per locale in the order of
/// [`Locale::ALL`].
pub const UI_STRINGS: &[(&str, [&str; LOCALES])] = &[
    // ---- General -----------------------------------------------------------------------
    (
        "ui_skip",
        [
            "Aller au formulaire",
            "Skip to the form",
            "Zum Formular springen",
            "Ir al formulario",
            "Vai al modulo",
            "Naar het formulier",
            "Ir para o formulário",
        ],
    ),
    (
        "ui_intro",
        [
            "Offrez un livre numérique comme un vrai livre\u{a0}: une jaquette à imprimer et à plier, avec le code qui amène le livre sur la liseuse.",
            "Give an eBook like a real book: a jacket to print and fold, with the code that brings the book onto the e-reader.",
            "Verschenken Sie ein eBook wie ein echtes Buch: ein Umschlag zum Ausdrucken und Falten, mit dem Code, der das Buch auf den eReader bringt.",
            "Regala un eBook como un libro de verdad: una sobrecubierta para imprimir y doblar, con el código que lleva el libro al eReader.",
            "Regala un eBook come un libro vero: una sovraccoperta da stampare e piegare, con il codice che porta il libro sull’eReader.",
            "Geef een e-book als een echt boek: een omslag om te printen en te vouwen, met de code die het boek op de e-reader zet.",
            "Ofereça um eBook como um livro verdadeiro: uma sobrecapa para imprimir e dobrar, com o código que leva o livro para o eReader.",
        ],
    ),
    (
        "ui_no_wasm",
        [
            "Cette page a besoin de WebAssembly, que ce navigateur ne propose pas. Essayez avec un navigateur récent.",
            "This page needs WebAssembly, which this browser does not provide. Try a recent browser.",
            "Diese Seite benötigt WebAssembly, das dieser Browser nicht bietet. Versuchen Sie es mit einem aktuellen Browser.",
            "Esta página necesita WebAssembly, que este navegador no ofrece. Prueba con un navegador reciente.",
            "Questa pagina richiede WebAssembly, che questo browser non offre. Prova con un browser recente.",
            "Deze pagina heeft WebAssembly nodig, en deze browser biedt dat niet. Probeer een recente browser.",
            "Esta página precisa de WebAssembly, que este navegador não oferece. Experimente um navegador recente.",
        ],
    ),
    (
        "ui_engine_error",
        [
            "Le moteur de mise en page n’a pas pu démarrer\u{a0}: {reason}",
            "The layout engine could not start: {reason}",
            "Die Satz-Engine konnte nicht starten: {reason}",
            "El motor de maquetación no pudo iniciarse: {reason}",
            "Il motore di impaginazione non si è avviato: {reason}",
            "De opmaakmotor kon niet starten: {reason}",
            "O motor de paginação não arrancou: {reason}",
        ],
    ),
    (
        "ui_privacy",
        [
            "La carte est composée dans votre navigateur. Aucun cookie, aucun traceur.",
            "The card is composed in your browser. No cookies, no tracking.",
            "Die Karte wird in Ihrem Browser gesetzt. Keine Cookies, kein Tracking.",
            "La tarjeta se compone en tu navegador. Sin cookies ni rastreadores.",
            "Il biglietto è composto nel tuo browser. Nessun cookie, nessun tracciamento.",
            "De kaart wordt in je browser opgemaakt. Geen cookies, geen tracking.",
            "O cartão é composto no seu navegador. Sem cookies, sem rastreio.",
        ],
    ),
    (
        "ui_optional",
        ["facultatif", "optional", "optional", "opcional", "facoltativo", "optioneel", "opcional"],
    ),
    (
        "ui_step_of",
        [
            "Étape {n} sur {total}",
            "Step {n} of {total}",
            "Schritt {n} von {total}",
            "Paso {n} de {total}",
            "Passaggio {n} di {total}",
            "Stap {n} van {total}",
            "Passo {n} de {total}",
        ],
    ),
    // ---- Step titles -------------------------------------------------------------------
    (
        "ui_step_book",
        ["Le livre", "The book", "Das Buch", "El libro", "Il libro", "Het boek", "O livro"],
    ),
    (
        "ui_step_cover",
        [
            "La couverture",
            "The cover",
            "Der Umschlag",
            "La cubierta",
            "La copertina",
            "De omslag",
            "A capa",
        ],
    ),
    (
        "ui_step_spine",
        ["Le dos", "The spine", "Der Rücken", "El lomo", "Il dorso", "De rug", "A lombada"],
    ),
    (
        "ui_step_message",
        [
            "Votre message",
            "Your message",
            "Ihre Nachricht",
            "Tu mensaje",
            "Il tuo messaggio",
            "Je boodschap",
            "A sua mensagem",
        ],
    ),
    (
        "ui_step_ebook",
        [
            "Le livre numérique",
            "The eBook",
            "Das eBook",
            "El eBook",
            "L’eBook",
            "Het e-book",
            "O eBook",
        ],
    ),
    (
        "ui_step_card",
        ["La carte", "The card", "Die Karte", "La tarjeta", "Il biglietto", "De kaart", "O cartão"],
    ),
    // ---- Step 1: the book --------------------------------------------------------------
    (
        "ui_isbn_hint",
        [
            "13 ou 10 chiffres, avec ou sans tirets.",
            "13 or 10 digits, with or without hyphens.",
            "13 oder 10 Ziffern, mit oder ohne Bindestriche.",
            "13 o 10 cifras, con o sin guiones.",
            "13 o 10 cifre, con o senza trattini.",
            "13 of 10 cijfers, met of zonder streepjes.",
            "13 ou 10 algarismos, com ou sem hífenes.",
        ],
    ),
    (
        "ui_lookup_privacy",
        [
            "La recherche interroge depuis votre navigateur\u{a0}: {sources}.",
            "The search queries, from your browser: {sources}.",
            "Die Suche fragt von Ihrem Browser aus ab: {sources}.",
            "La búsqueda consulta desde tu navegador: {sources}.",
            "La ricerca interroga dal tuo browser: {sources}.",
            "De zoekopdracht raadpleegt vanuit je browser: {sources}.",
            "A pesquisa consulta a partir do seu navegador: {sources}.",
        ],
    ),
    (
        "ui_by_hand",
        [
            "Ou saisissez les informations vous-même",
            "Or fill in the details yourself",
            "Oder geben Sie die Angaben selbst ein",
            "O introduce los datos a mano",
            "Oppure inserisci i dati a mano",
            "Of vul de gegevens zelf in",
            "Ou preencha os dados à mão",
        ],
    ),
    (
        "ui_field_subtitle",
        [
            "Sous-titre",
            "Subtitle",
            "Untertitel",
            "Subtítulo",
            "Sottotitolo",
            "Ondertitel",
            "Subtítulo",
        ],
    ),
    (
        "ui_field_publisher",
        ["Éditeur", "Publisher", "Verlag", "Editorial", "Editore", "Uitgever", "Editora"],
    ),
    (
        "ui_book_language",
        [
            "Langue du livre",
            "Language of the book",
            "Sprache des Buches",
            "Idioma del libro",
            "Lingua del libro",
            "Taal van het boek",
            "Língua do livro",
        ],
    ),
    (
        "ui_language_auto",
        [
            "Non précisée",
            "Not specified",
            "Nicht angegeben",
            "Sin especificar",
            "Non specificata",
            "Niet opgegeven",
            "Não indicada",
        ],
    ),
    (
        "ui_found_in",
        [
            "Trouvé dans\u{a0}: {sources}",
            "Found in: {sources}",
            "Gefunden in: {sources}",
            "Encontrado en: {sources}",
            "Trovato in: {sources}",
            "Gevonden in: {sources}",
            "Encontrado em: {sources}",
        ],
    ),
    (
        "ui_lookup_failed",
        [
            "Le catalogue n’a pas répondu. Saisissez les informations à la main.",
            "The catalogue did not answer. Fill in the details by hand.",
            "Der Katalog hat nicht geantwortet. Geben Sie die Angaben von Hand ein.",
            "El catálogo no respondió. Introduce los datos a mano.",
            "Il catalogo non ha risposto. Inserisci i dati a mano.",
            "De catalogus gaf geen antwoord. Vul de gegevens zelf in.",
            "O catálogo não respondeu. Preencha os dados à mão.",
        ],
    ),
    (
        "ui_title_required",
        [
            "Indiquez le titre du livre.",
            "Enter the title of the book.",
            "Geben Sie den Titel des Buches ein.",
            "Indica el título del libro.",
            "Indica il titolo del libro.",
            "Vul de titel van het boek in.",
            "Indique o título do livro.",
        ],
    ),
    // ---- Step 2: the cover -------------------------------------------------------------
    (
        "ui_cover_typographic",
        [
            "Carte typographique (recommandé)",
            "Typographic card (recommended)",
            "Typografische Karte (empfohlen)",
            "Tarjeta tipográfica (recomendado)",
            "Biglietto tipografico (consigliato)",
            "Typografische kaart (aanbevolen)",
            "Cartão tipográfico (recomendado)",
        ],
    ),
    (
        "ui_cover_typographic_help",
        [
            "Le titre et l’auteur composés sur un fond uni. Aucune image, donc aucune autorisation à obtenir.",
            "Title and author set in type on a plain colour. No picture, so no permission to obtain.",
            "Titel und Autor, gesetzt auf einfarbigem Grund. Kein Bild, also keine Genehmigung nötig.",
            "Título y autor compuestos sobre un fondo liso. Sin imagen, así que no hace falta ningún permiso.",
            "Titolo e autore composti su un fondo a tinta unita. Nessuna immagine, quindi nessun permesso da chiedere.",
            "Titel en auteur gezet op een effen kleur. Geen afbeelding, dus geen toestemming nodig.",
            "Título e autor compostos sobre um fundo liso. Sem imagem, logo sem autorização a pedir.",
        ],
    ),
    (
        "ui_cover_own",
        [
            "Imprimer une image que j’ai le droit d’imprimer",
            "Print an image I am allowed to print",
            "Ein Bild drucken, das ich drucken darf",
            "Imprimir una imagen que puedo imprimir",
            "Stampare un’immagine che posso stampare",
            "Een afbeelding printen die ik mag printen",
            "Imprimir uma imagem que posso imprimir",
        ],
    ),
    (
        "ui_cover_own_help",
        [
            "Choisissez un fichier image sur votre appareil. Il reste dans votre navigateur.",
            "Choose an image file from your device. It stays in your browser.",
            "Wählen Sie eine Bilddatei von Ihrem Gerät. Sie bleibt in Ihrem Browser.",
            "Elige un archivo de imagen de tu dispositivo. Se queda en tu navegador.",
            "Scegli un file immagine dal tuo dispositivo. Resta nel tuo browser.",
            "Kies een afbeeldingsbestand op je apparaat. Het blijft in je browser.",
            "Escolha um ficheiro de imagem no seu dispositivo. Fica no seu navegador.",
        ],
    ),
    (
        "ui_cover_why",
        [
            "Pourquoi\u{a0}? Une couverture est une œuvre protégée\u{a0}: l’imprimer demande d’en avoir le droit. Votre propre image, ou une image que l’on vous a autorisé à imprimer, est le choix sûr.",
            "Why? A cover is protected artwork: printing it requires the right to do so. Your own image, or one you were authorised to print, is the safe choice.",
            "Warum? Ein Umschlag ist ein geschütztes Werk: Wer ihn druckt, braucht das Recht dazu. Ihr eigenes Bild oder eines, das Sie drucken dürfen, ist die sichere Wahl.",
            "¿Por qué? Una cubierta es una obra protegida: imprimirla exige tener derecho a hacerlo. Tu propia imagen, o una que te hayan autorizado a imprimir, es la opción segura.",
            "Perché? Una copertina è un’opera protetta: per stamparla occorre averne il diritto. Una tua immagine, o una che sei autorizzato a stampare, è la scelta sicura.",
            "Waarom? Een omslag is beschermd werk: wie het print, moet daar het recht toe hebben. Je eigen afbeelding, of een die je mag printen, is de veilige keuze.",
            "Porquê? Uma capa é uma obra protegida: imprimi-la exige ter esse direito. A sua própria imagem, ou uma que foi autorizado a imprimir, é a escolha segura.",
        ],
    ),
    (
        "ui_legal_note",
        [
            "Lire la note juridique",
            "Read the legal note",
            "Rechtlichen Hinweis lesen",
            "Leer la nota legal",
            "Leggi la nota legale",
            "Lees de juridische toelichting",
            "Ler a nota legal",
        ],
    ),
    (
        "ui_preview_only_title",
        [
            "Est-ce bien ce livre\u{a0}?",
            "Is this the book?",
            "Ist es dieses Buch?",
            "¿Es este el libro?",
            "È questo il libro?",
            "Is dit het boek?",
            "É este o livro?",
        ],
    ),
    (
        "ui_preview_only",
        [
            "Image affichée pour reconnaître le livre, depuis {source}. Elle n’est jamais imprimée sur la carte.",
            "Shown so that you can recognise the book, from {source}. It is never printed on the card.",
            "Wird angezeigt, damit Sie das Buch erkennen, von {source}. Es wird nie auf die Karte gedruckt.",
            "Se muestra para que reconozcas el libro, desde {source}. Nunca se imprime en la tarjeta.",
            "Mostrata per riconoscere il libro, da {source}. Non viene mai stampata sul biglietto.",
            "Getoond zodat je het boek herkent, van {source}. Het wordt nooit op de kaart geprint.",
            "Mostrada para reconhecer o livro, a partir de {source}. Nunca é impressa no cartão.",
        ],
    ),
    (
        "ui_choose_image",
        [
            "Fichier image (JPEG, PNG ou WebP)",
            "Image file (JPEG, PNG or WebP)",
            "Bilddatei (JPEG, PNG oder WebP)",
            "Archivo de imagen (JPEG, PNG o WebP)",
            "File immagine (JPEG, PNG o WebP)",
            "Afbeeldingsbestand (JPEG, PNG of WebP)",
            "Ficheiro de imagem (JPEG, PNG ou WebP)",
        ],
    ),
    (
        "ui_rights_legend",
        [
            "Pourquoi pouvez-vous imprimer cette image\u{a0}?",
            "Why may you print this image?",
            "Warum dürfen Sie dieses Bild drucken?",
            "¿Por qué puedes imprimir esta imagen?",
            "Perché puoi stampare questa immagine?",
            "Waarom mag je deze afbeelding printen?",
            "Porque pode imprimir esta imagem?",
        ],
    ),
    (
        "ui_rights_own_work",
        [
            "C’est ma propre création",
            "It is my own work",
            "Es ist mein eigenes Werk",
            "Es obra mía",
            "È opera mia",
            "Het is mijn eigen werk",
            "É obra minha",
        ],
    ),
    (
        "ui_rights_print_licence",
        [
            "J’ai une licence pour l’imprimer",
            "I hold a licence to print it",
            "Ich habe eine Lizenz für den Druck",
            "Tengo una licencia para imprimirla",
            "Ho una licenza per stamparla",
            "Ik heb een licentie om ze te printen",
            "Tenho uma licença para a imprimir",
        ],
    ),
    (
        "ui_rights_publisher_authorised",
        [
            "L’éditeur m’y a autorisé",
            "The publisher authorised me",
            "Der Verlag hat es mir erlaubt",
            "La editorial me ha autorizado",
            "L’editore mi ha autorizzato",
            "De uitgever heeft me toestemming gegeven",
            "A editora autorizou-me",
        ],
    ),
    (
        "ui_rights_public_domain",
        [
            "Elle est dans le domaine public",
            "It is in the public domain",
            "Es ist gemeinfrei",
            "Está en el dominio público",
            "È di pubblico dominio",
            "Ze behoort tot het publieke domein",
            "Está no domínio público",
        ],
    ),
    (
        "ui_rights_private_copy",
        [
            "Copie privée, d’un livre que j’ai acheté, pour mon imprimante à la maison",
            "Private copy, from a book I bought, for my own printer at home",
            "Privatkopie eines Buches, das ich gekauft habe, für meinen eigenen Drucker zu Hause",
            "Copia privada, de un libro que he comprado, para mi impresora de casa",
            "Copia privata, da un libro che ho comprato, per la mia stampante di casa",
            "Privékopie, van een boek dat ik heb gekocht, voor mijn eigen printer thuis",
            "Cópia privada, de um livro que comprei, para a minha impressora de casa",
        ],
    ),
    (
        "ui_rights_required",
        [
            "Choisissez l’une de ces déclarations pour imprimer l’image. Sans déclaration, la carte typographique est utilisée.",
            "Choose one of these statements to print the image. Without one, the typographic card is used.",
            "Wählen Sie eine dieser Erklärungen, um das Bild zu drucken. Ohne Erklärung wird die typografische Karte verwendet.",
            "Elige una de estas declaraciones para imprimir la imagen. Sin declaración se usa la tarjeta tipográfica.",
            "Scegli una di queste dichiarazioni per stampare l’immagine. Senza dichiarazione si usa il biglietto tipografico.",
            "Kies een van deze verklaringen om de afbeelding te printen. Zonder verklaring wordt de typografische kaart gebruikt.",
            "Escolha uma destas declarações para imprimir a imagem. Sem declaração, usa-se o cartão tipográfico.",
        ],
    ),
    (
        "ui_rights_pro_disabled",
        [
            "Indisponible avec cette déclaration\u{a0}: une copie privée ne peut être imprimée que sur votre propre imprimante.",
            "Not available with this statement: a private copy may only be printed on your own printer.",
            "Mit dieser Erklärung nicht verfügbar: Eine Privatkopie darf nur auf Ihrem eigenen Drucker gedruckt werden.",
            "No disponible con esta declaración: una copia privada solo puede imprimirse en tu propia impresora.",
            "Non disponibile con questa dichiarazione: una copia privata si può stampare solo con la propria stampante.",
            "Niet beschikbaar met deze verklaring: een privékopie mag alleen op je eigen printer worden geprint.",
            "Indisponível com esta declaração: uma cópia privada só pode ser impressa na sua própria impressora.",
        ],
    ),
    (
        "ui_own_printer_only",
        [
            "Ce fichier est à imprimer sur votre propre imprimante uniquement.",
            "This file is to be printed on your own printer only.",
            "Diese Datei darf nur auf Ihrem eigenen Drucker gedruckt werden.",
            "Este archivo es solo para imprimir en tu propia impresora.",
            "Questo file va stampato solo con la tua stampante.",
            "Dit bestand mag alleen op je eigen printer worden geprint.",
            "Este ficheiro é para imprimir apenas na sua própria impressora.",
        ],
    ),
    (
        "ui_public_domain_note",
        [
            "Le domaine public dépend du pays et de l’année de décès de l’artiste. La vérification vous revient.",
            "Public domain depends on the country and on the year the artist died. Checking it is up to you.",
            "Gemeinfreiheit hängt vom Land und vom Todesjahr der Künstlerin oder des Künstlers ab. Die Prüfung liegt bei Ihnen.",
            "El dominio público depende del país y del año de fallecimiento del artista. La comprobación te corresponde a ti.",
            "Il pubblico dominio dipende dal paese e dall’anno di morte dell’artista. La verifica spetta a te.",
            "Het publieke domein hangt af van het land en van het sterfjaar van de kunstenaar. De controle is aan jou.",
            "O domínio público depende do país e do ano da morte do artista. A verificação cabe-lhe a si.",
        ],
    ),
    (
        "ui_cover_credit",
        [
            "Crédit de l’image",
            "Credit line for the image",
            "Bildnachweis",
            "Crédito de la imagen",
            "Crediti dell’immagine",
            "Bronvermelding van de afbeelding",
            "Crédito da imagem",
        ],
    ),
    (
        "ui_quality_excellent",
        [
            "Assez nette pour l’impression ({ppi} ppp).",
            "Sharp enough for print ({ppi} ppi).",
            "Scharf genug für den Druck ({ppi} ppi).",
            "Suficientemente nítida para imprimir ({ppi} ppp).",
            "Abbastanza nitida per la stampa ({ppi} ppi).",
            "Scherp genoeg om te printen ({ppi} ppi).",
            "Suficientemente nítida para impressão ({ppi} ppp).",
        ],
    ),
    (
        "ui_quality_acceptable",
        [
            "Correcte pour l’impression ({ppi} ppp).",
            "Good enough for print ({ppi} ppi).",
            "Für den Druck ausreichend ({ppi} ppi).",
            "Correcta para imprimir ({ppi} ppp).",
            "Adeguata per la stampa ({ppi} ppi).",
            "Voldoende om te printen ({ppi} ppi).",
            "Adequada para impressão ({ppi} ppp).",
        ],
    ),
    (
        "ui_quality_low",
        [
            "Elle paraîtra un peu floue à l’impression ({ppi} ppp).",
            "It will look slightly soft in print ({ppi} ppi).",
            "Es wird im Druck etwas unscharf wirken ({ppi} ppi).",
            "Se verá algo borrosa al imprimir ({ppi} ppp).",
            "In stampa apparirà un po’ sfocata ({ppi} ppi).",
            "Ze zal geprint wat onscherp ogen ({ppi} ppi).",
            "Ficará um pouco desfocada na impressão ({ppi} ppp).",
        ],
    ),
    (
        "ui_quality_insufficient",
        [
            "Trop peu de pixels\u{a0}: elle sera floue à l’impression ({ppi} ppp).",
            "Too few pixels: it will look blurred in print ({ppi} ppi).",
            "Zu wenige Pixel: Es wird im Druck unscharf ({ppi} ppi).",
            "Demasiados pocos píxeles: saldrá borrosa al imprimir ({ppi} ppp).",
            "Troppo pochi pixel: in stampa risulterà sfocata ({ppi} ppi).",
            "Te weinig pixels: ze wordt geprint onscherp ({ppi} ppi).",
            "Píxeis a menos: ficará desfocada na impressão ({ppi} ppp).",
        ],
    ),
    (
        "ui_image_size",
        [
            "{width} × {height} px",
            "{width} × {height} px",
            "{width} × {height} px",
            "{width} × {height} px",
            "{width} × {height} px",
            "{width} × {height} px",
            "{width} × {height} px",
        ],
    ),
    (
        "ui_cover_placeholder",
        [
            "Ce fichier ressemble à une image «\u{a0}couverture indisponible\u{a0}», pas à une couverture.",
            "This file looks like a “no cover available” picture, not a cover.",
            "Diese Datei sieht aus wie ein Platzhalter „kein Cover verfügbar“, nicht wie ein Umschlag.",
            "Este archivo parece una imagen de «cubierta no disponible», no una cubierta.",
            "Questo file sembra un’immagine «copertina non disponibile», non una copertina.",
            "Dit bestand lijkt een plaatje ‘geen omslag beschikbaar’, geen omslag.",
            "Este ficheiro parece uma imagem de «capa indisponível», não uma capa.",
        ],
    ),
    (
        "ui_cover_unreadable",
        [
            "Ce fichier n’est pas une image JPEG, PNG ou WebP lisible.",
            "This file is not a readable JPEG, PNG or WebP image.",
            "Diese Datei ist kein lesbares JPEG-, PNG- oder WebP-Bild.",
            "Este archivo no es una imagen JPEG, PNG o WebP legible.",
            "Questo file non è un’immagine JPEG, PNG o WebP leggibile.",
            "Dit bestand is geen leesbare JPEG-, PNG- of WebP-afbeelding.",
            "Este ficheiro não é uma imagem JPEG, PNG ou WebP legível.",
        ],
    ),
    (
        "ui_cover_too_large",
        [
            "Cette image est trop grande.",
            "This image is too large.",
            "Dieses Bild ist zu groß.",
            "Esta imagen es demasiado grande.",
            "Questa immagine è troppo grande.",
            "Deze afbeelding is te groot.",
            "Esta imagem é demasiado grande.",
        ],
    ),
    (
        "ui_remove_image",
        [
            "Retirer l’image",
            "Remove the image",
            "Bild entfernen",
            "Quitar la imagen",
            "Rimuovi l’immagine",
            "Afbeelding verwijderen",
            "Remover a imagem",
        ],
    ),
    (
        "ui_ground_colour",
        [
            "Couleur de la carte",
            "Card colour",
            "Farbe der Karte",
            "Color de la tarjeta",
            "Colore del biglietto",
            "Kleur van de kaart",
            "Cor do cartão",
        ],
    ),
    (
        "ui_ground_auto",
        [
            "Couleur automatique",
            "Automatic colour",
            "Automatische Farbe",
            "Color automático",
            "Colore automatico",
            "Automatische kleur",
            "Cor automática",
        ],
    ),
    // ---- Step 3: the spine -------------------------------------------------------------
    (
        "ui_spine_help",
        [
            "Le dos mesure 10\u{a0}mm et porte le titre, comme sur un vrai livre.",
            "The spine is 10 mm wide and carries the title, as on a real book.",
            "Der Rücken ist 10 mm breit und trägt den Titel, wie bei einem echten Buch.",
            "El lomo mide 10 mm y lleva el título, como en un libro de verdad.",
            "Il dorso è largo 10 mm e porta il titolo, come in un libro vero.",
            "De rug is 10 mm breed en draagt de titel, zoals bij een echt boek.",
            "A lombada tem 10 mm e leva o título, como num livro verdadeiro.",
        ],
    ),
    (
        "ui_spine_typeset",
        [
            "Titre sur la couleur de la carte (recommandé)",
            "Title on the card colour (recommended)",
            "Titel auf der Kartenfarbe (empfohlen)",
            "Título sobre el color de la tarjeta (recomendado)",
            "Titolo sul colore del biglietto (consigliato)",
            "Titel op de kleur van de kaart (aanbevolen)",
            "Título sobre a cor do cartão (recomendado)",
        ],
    ),
    (
        "ui_spine_cover_colour",
        [
            "Titre sur une couleur unie calculée d’après mon image",
            "Title on a plain colour computed from my image",
            "Titel auf einer einfarbigen Fläche, berechnet aus meinem Bild",
            "Título sobre un color liso calculado a partir de mi imagen",
            "Titolo su una tinta unita calcolata dalla mia immagine",
            "Titel op een effen kleur, berekend uit mijn afbeelding",
            "Título sobre uma cor lisa calculada a partir da minha imagem",
        ],
    ),
    (
        "ui_spine_edge",
        [
            "Prolonger le bord de mon image sur le dos",
            "Extend the edge of my image onto the spine",
            "Den Rand meines Bildes auf den Rücken verlängern",
            "Prolongar el borde de mi imagen sobre el lomo",
            "Prolungare il bordo della mia immagine sul dorso",
            "De rand van mijn afbeelding over de rug doortrekken",
            "Prolongar a margem da minha imagem para a lombada",
        ],
    ),
    (
        "ui_adapt_tick",
        [
            "Je suis autorisé à adapter cette couverture",
            "I am authorised to adapt this cover",
            "Ich darf diesen Umschlag bearbeiten",
            "Tengo autorización para adaptar esta cubierta",
            "Sono autorizzato ad adattare questa copertina",
            "Ik heb toestemming om deze omslag aan te passen",
            "Estou autorizado a adaptar esta capa",
        ],
    ),
    (
        "ui_adapt_help",
        [
            "Ne cochez que si le titulaire des droits permet de modifier l’œuvre.",
            "Only tick this if the rights holder allows the artwork to be altered.",
            "Nur ankreuzen, wenn der Rechteinhaber eine Veränderung des Werks erlaubt.",
            "Márcalo solo si el titular de los derechos permite modificar la obra.",
            "Seleziona solo se il titolare dei diritti consente di modificare l’opera.",
            "Vink dit alleen aan als de rechthebbende toestaat dat het werk wordt gewijzigd.",
            "Assinale apenas se o titular dos direitos permitir alterar a obra.",
        ],
    ),
    (
        "ui_adapt_unavailable",
        [
            "Adapter l’image demande une image et une déclaration qui le permet (création personnelle, licence ou autorisation de l’éditeur).",
            "Adapting needs an image and a statement that allows it (own work, licence or the publisher’s authorisation).",
            "Eine Bearbeitung setzt ein Bild und eine Erklärung voraus, die sie erlaubt (eigenes Werk, Lizenz oder Erlaubnis des Verlags).",
            "Adaptar exige una imagen y una declaración que lo permita (obra propia, licencia o autorización de la editorial).",
            "Per adattare servono un’immagine e una dichiarazione che lo consenta (opera propria, licenza o autorizzazione dell’editore).",
            "Aanpassen vraagt een afbeelding en een verklaring die dat toestaat (eigen werk, licentie of toestemming van de uitgever).",
            "Adaptar exige uma imagem e uma declaração que o permita (obra própria, licença ou autorização da editora).",
        ],
    ),
    (
        "ui_title_font",
        [
            "Caractère du titre",
            "Title typeface",
            "Schrift des Titels",
            "Tipografía del título",
            "Carattere del titolo",
            "Lettertype van de titel",
            "Tipo de letra do título",
        ],
    ),
    (
        "ui_font_detect",
        [
            "Proposer le caractère libre le plus proche",
            "Suggest the closest open typeface",
            "Die ähnlichste freie Schrift vorschlagen",
            "Sugerir la tipografía libre más parecida",
            "Suggerisci il carattere libero più simile",
            "Het meest gelijkende vrije lettertype voorstellen",
            "Sugerir o tipo de letra livre mais parecido",
        ],
    ),
    (
        "ui_font_detect_help",
        [
            "Compare le lettrage de votre image aux caractères libres de ce projet. Le lettrage lui-même n’est jamais copié.",
            "Compares the lettering of your image with this project’s open typefaces. The lettering itself is never copied.",
            "Vergleicht den Schriftzug Ihres Bildes mit den freien Schriften dieses Projekts. Der Schriftzug selbst wird nie kopiert.",
            "Compara el rótulo de tu imagen con las tipografías libres de este proyecto. El rótulo en sí nunca se copia.",
            "Confronta il lettering della tua immagine con i caratteri liberi di questo progetto. Il lettering non viene mai copiato.",
            "Vergelijkt de belettering van je afbeelding met de vrije lettertypes van dit project. De belettering zelf wordt nooit gekopieerd.",
            "Compara o lettering da sua imagem com os tipos de letra livres deste projeto. O lettering em si nunca é copiado.",
        ],
    ),
    (
        "ui_font_suggested",
        [
            "Caractère libre le plus proche\u{a0}: {family}",
            "Closest open typeface: {family}",
            "Ähnlichste freie Schrift: {family}",
            "Tipografía libre más parecida: {family}",
            "Carattere libero più simile: {family}",
            "Meest gelijkende vrije lettertype: {family}",
            "Tipo de letra livre mais parecido: {family}",
        ],
    ),
    (
        "ui_font_uncertain",
        [
            "Pas de correspondance nette\u{a0}; le plus proche est {family}",
            "No clear match; the nearest is {family}",
            "Keine klare Übereinstimmung; am nächsten kommt {family}",
            "Sin coincidencia clara; la más cercana es {family}",
            "Nessuna corrispondenza netta; il più vicino è {family}",
            "Geen duidelijke overeenkomst; het dichtst in de buurt komt {family}",
            "Sem correspondência clara; o mais próximo é {family}",
        ],
    ),
    (
        "ui_font_none",
        [
            "Aucun lettrage reconnu dans l’image.",
            "No lettering found in the image.",
            "Im Bild wurde kein Schriftzug erkannt.",
            "No se ha reconocido ningún rótulo en la imagen.",
            "Nessun lettering riconosciuto nell’immagine.",
            "Geen belettering gevonden in de afbeelding.",
            "Nenhum lettering reconhecido na imagem.",
        ],
    ),
    (
        "ui_font_unavailable",
        [
            "Détection du caractère indisponible",
            "Font detection unavailable",
            "Schrifterkennung nicht verfügbar",
            "Detección de tipografía no disponible",
            "Rilevamento del carattere non disponibile",
            "Lettertypeherkenning niet beschikbaar",
            "Deteção do tipo de letra indisponível",
        ],
    ),
    (
        "ui_font_needs_image",
        [
            "Ajoutez une image à l’étape 2 pour utiliser la détection.",
            "Add an image in step 2 to use detection.",
            "Fügen Sie in Schritt 2 ein Bild hinzu, um die Erkennung zu nutzen.",
            "Añade una imagen en el paso 2 para usar la detección.",
            "Aggiungi un’immagine al passaggio 2 per usare il rilevamento.",
            "Voeg in stap 2 een afbeelding toe om de herkenning te gebruiken.",
            "Adicione uma imagem no passo 2 para usar a deteção.",
        ],
    ),
    (
        "ui_direction_is",
        [
            "Sens de lecture\u{a0}: {direction}. Il suit la langue du livre.",
            "Reading direction: {direction}. It follows the language of the book.",
            "Leserichtung: {direction}. Sie folgt der Sprache des Buches.",
            "Sentido de lectura: {direction}. Sigue el idioma del libro.",
            "Senso di lettura: {direction}. Segue la lingua del libro.",
            "Leesrichting: {direction}. Die volgt de taal van het boek.",
            "Sentido de leitura: {direction}. Segue a língua do livro.",
        ],
    ),
    (
        "ui_direction_top_to_bottom",
        [
            "de haut en bas",
            "top to bottom",
            "von oben nach unten",
            "de arriba abajo",
            "dall’alto in basso",
            "van boven naar beneden",
            "de cima para baixo",
        ],
    ),
    (
        "ui_direction_bottom_to_top",
        [
            "de bas en haut",
            "bottom to top",
            "von unten nach oben",
            "de abajo arriba",
            "dal basso in alto",
            "van beneden naar boven",
            "de baixo para cima",
        ],
    ),
    // ---- Step 4: the message -----------------------------------------------------------
    (
        "ui_card_language",
        [
            "Langue de la carte",
            "Language of the card",
            "Sprache der Karte",
            "Idioma de la tarjeta",
            "Lingua del biglietto",
            "Taal van de kaart",
            "Língua do cartão",
        ],
    ),
    (
        "ui_message_hint",
        [
            "Laissez vide pour l’écrire à la main sur la carte.",
            "Leave empty to write it by hand on the card.",
            "Leer lassen, um sie von Hand auf die Karte zu schreiben.",
            "Déjalo vacío para escribirlo a mano en la tarjeta.",
            "Lascia vuoto per scriverlo a mano sul biglietto.",
            "Laat leeg om ze met de hand op de kaart te schrijven.",
            "Deixe vazio para a escrever à mão no cartão.",
        ],
    ),
    // ---- Step 5: the eBook -------------------------------------------------------------
    (
        "ui_ebook_upload",
        [
            "J’ai le fichier et le droit de le transmettre",
            "I have the file and the right to pass it on",
            "Ich habe die Datei und darf sie weitergeben",
            "Tengo el archivo y derecho a transmitirlo",
            "Ho il file e il diritto di trasmetterlo",
            "Ik heb het bestand en mag het doorgeven",
            "Tenho o ficheiro e o direito de o transmitir",
        ],
    ),
    (
        "ui_ebook_code",
        [
            "J’ai déjà un code (remis par ma librairie)",
            "I already have a code (from my bookshop)",
            "Ich habe bereits einen Code (von meiner Buchhandlung)",
            "Ya tengo un código (de mi librería)",
            "Ho già un codice (dalla mia libreria)",
            "Ik heb al een code (van mijn boekhandel)",
            "Já tenho um código (da minha livraria)",
        ],
    ),
    (
        "ui_ebook_demo",
        [
            "Essayer avec un code de démonstration",
            "Try with a demonstration code",
            "Mit einem Demo-Code ausprobieren",
            "Probar con un código de demostración",
            "Provare con un codice dimostrativo",
            "Uitproberen met een democode",
            "Experimentar com um código de demonstração",
        ],
    ),
    (
        "ui_ebook_demo_note",
        [
            "Pour essayer la carte\u{a0}: ce code ne donne accès à aucun livre.",
            "For trying the card out: this code leads to no book.",
            "Zum Ausprobieren der Karte: Dieser Code führt zu keinem Buch.",
            "Para probar la tarjeta: este código no da acceso a ningún libro.",
            "Per provare il biglietto: questo codice non porta a nessun libro.",
            "Om de kaart uit te proberen: deze code geeft geen toegang tot een boek.",
            "Para experimentar o cartão: este código não dá acesso a nenhum livro.",
        ],
    ),
    (
        "ui_ebook_upload_help",
        [
            "EPUB ou PDF sans DRM, 50\u{a0}Mo au plus. Le fichier est envoyé à {host}, qui le remet au destinataire.",
            "EPUB or PDF without DRM, 50 MB at most. The file is sent to {host}, which delivers it to the recipient.",
            "EPUB oder PDF ohne DRM, höchstens 50 MB. Die Datei wird an {host} gesendet und von dort an die beschenkte Person ausgeliefert.",
            "EPUB o PDF sin DRM, 50 MB como máximo. El archivo se envía a {host}, que lo entrega al destinatario.",
            "EPUB o PDF senza DRM, al massimo 50 MB. Il file viene inviato a {host}, che lo consegna al destinatario.",
            "EPUB of pdf zonder DRM, hoogstens 50 MB. Het bestand wordt naar {host} gestuurd, dat het aan de ontvanger bezorgt.",
            "EPUB ou PDF sem DRM, 50 MB no máximo. O ficheiro é enviado para {host}, que o entrega ao destinatário.",
        ],
    ),
    (
        "ui_create_gift",
        [
            "Créer le cadeau",
            "Create the gift",
            "Geschenk anlegen",
            "Crear el regalo",
            "Crea il regalo",
            "Cadeau aanmaken",
            "Criar a oferta",
        ],
    ),
    (
        "ui_gift_created",
        [
            "Cadeau créé. Code\u{a0}: {code}",
            "Gift created. Code: {code}",
            "Geschenk angelegt. Code: {code}",
            "Regalo creado. Código: {code}",
            "Regalo creato. Codice: {code}",
            "Cadeau aangemaakt. Code: {code}",
            "Oferta criada. Código: {code}",
        ],
    ),
    (
        "ui_manage_key",
        [
            "Clé de gestion, à conserver pour annuler le cadeau ou remplacer le code\u{a0}: {key}",
            "Management key, to keep for cancelling the gift or replacing the code: {key}",
            "Verwaltungsschlüssel, bitte aufbewahren, um das Geschenk zu stornieren oder den Code zu ersetzen: {key}",
            "Clave de gestión, guárdala para cancelar el regalo o sustituir el código: {key}",
            "Chiave di gestione, da conservare per annullare il regalo o sostituire il codice: {key}",
            "Beheersleutel, bewaar die om het cadeau te annuleren of de code te vervangen: {key}",
            "Chave de gestão, a guardar para cancelar a oferta ou substituir o código: {key}",
        ],
    ),
    (
        "ui_code_hint",
        [
            "12 lettres, par exemple BCDF-GHJK-LMNP.",
            "12 letters, for example BCDF-GHJK-LMNP.",
            "12 Buchstaben, zum Beispiel BCDF-GHJK-LMNP.",
            "12 letras, por ejemplo BCDF-GHJK-LMNP.",
            "12 lettere, per esempio BCDF-GHJK-LMNP.",
            "12 letters, bijvoorbeeld BCDF-GHJK-LMNP.",
            "12 letras, por exemplo BCDF-GHJK-LMNP.",
        ],
    ),
    (
        "ui_code_ok",
        [
            "Code bien formé\u{a0}: {code}",
            "Well-formed code: {code}",
            "Code korrekt aufgebaut: {code}",
            "Código bien formado: {code}",
            "Codice ben formato: {code}",
            "Code correct opgebouwd: {code}",
            "Código bem formado: {code}",
        ],
    ),
    (
        "ui_site_address",
        [
            "Adresse du site où le code est saisi",
            "Address of the site where the code is entered",
            "Adresse der Website, auf der der Code eingegeben wird",
            "Dirección del sitio donde se introduce el código",
            "Indirizzo del sito in cui si inserisce il codice",
            "Adres van de site waar de code wordt ingevoerd",
            "Endereço do sítio onde o código é introduzido",
        ],
    ),
    (
        "ui_server_unreachable",
        [
            "Le serveur n’a pas répondu. Vérifiez votre connexion et réessayez.",
            "The server did not answer. Check your connection and try again.",
            "Der Server hat nicht geantwortet. Prüfen Sie Ihre Verbindung und versuchen Sie es erneut.",
            "El servidor no respondió. Comprueba tu conexión e inténtalo de nuevo.",
            "Il server non ha risposto. Controlla la connessione e riprova.",
            "De server gaf geen antwoord. Controleer je verbinding en probeer het opnieuw.",
            "O servidor não respondeu. Verifique a ligação e tente de novo.",
        ],
    ),
    (
        "ui_server_refused",
        [
            "Le serveur a refusé\u{a0}: {reason}",
            "The server refused: {reason}",
            "Der Server hat abgelehnt: {reason}",
            "El servidor lo ha rechazado: {reason}",
            "Il server ha rifiutato: {reason}",
            "De server weigerde: {reason}",
            "O servidor recusou: {reason}",
        ],
    ),
    (
        "ui_need_file_and_basis",
        [
            "Choisissez un fichier et une déclaration.",
            "Choose a file and a statement.",
            "Wählen Sie eine Datei und eine Erklärung.",
            "Elige un archivo y una declaración.",
            "Scegli un file e una dichiarazione.",
            "Kies een bestand en een verklaring.",
            "Escolha um ficheiro e uma declaração.",
        ],
    ),
    (
        "ui_uploading",
        [
            "Envoi…",
            "Sending…",
            "Wird gesendet …",
            "Enviando…",
            "Invio…",
            "Bezig met verzenden…",
            "A enviar…",
        ],
    ),
    // ---- Step 6: the card --------------------------------------------------------------
    ("ui_format", ["Format", "Size", "Format", "Formato", "Formato", "Formaat", "Formato"]),
    (
        "ui_profile",
        ["Impression", "Printing", "Druck", "Impresión", "Stampa", "Printen", "Impressão"],
    ),
    (
        "ui_profile_home_a4",
        [
            "Chez moi, feuille A4",
            "At home, A4 sheet",
            "Zu Hause, A4-Blatt",
            "En casa, hoja A4",
            "A casa, foglio A4",
            "Thuis, A4-vel",
            "Em casa, folha A4",
        ],
    ),
    (
        "ui_profile_home_letter",
        [
            "Chez moi, feuille US Letter",
            "At home, US Letter sheet",
            "Zu Hause, US-Letter-Blatt",
            "En casa, hoja US Letter",
            "A casa, foglio US Letter",
            "Thuis, US Letter-vel",
            "Em casa, folha US Letter",
        ],
    ),
    (
        "ui_profile_home_a3",
        [
            "Chez moi, feuille A3",
            "At home, A3 sheet",
            "Zu Hause, A3-Blatt",
            "En casa, hoja A3",
            "A casa, foglio A3",
            "Thuis, A3-vel",
            "Em casa, folha A3",
        ],
    ),
    (
        "ui_profile_upload_eu",
        [
            "Imprimeur en ligne (PDF/X-4, fonds perdus de 3\u{a0}mm, sans repères)",
            "Online printer (PDF/X-4, 3 mm bleed, no marks)",
            "Online-Druckerei (PDF/X-4, 3 mm Beschnitt, ohne Marken)",
            "Imprenta en línea (PDF/X-4, sangrado de 3 mm, sin marcas)",
            "Tipografia online (PDF/X-4, abbondanza di 3 mm, senza crocini)",
            "Onlinedrukker (PDF/X-4, 3 mm afloop, zonder snijtekens)",
            "Gráfica online (PDF/X-4, sangria de 3 mm, sem marcas)",
        ],
    ),
    (
        "ui_profile_professional_eu",
        [
            "Imprimeur (PDF/X-4 avec traits de coupe)",
            "Print shop (PDF/X-4 with crop marks)",
            "Druckerei (PDF/X-4 mit Schnittmarken)",
            "Imprenta (PDF/X-4 con marcas de corte)",
            "Tipografia (PDF/X-4 con crocini di taglio)",
            "Drukkerij (PDF/X-4 met snijtekens)",
            "Gráfica (PDF/X-4 com marcas de corte)",
        ],
    ),
    (
        "ui_format_unfit",
        [
            "Ce format ne tient pas sur cette feuille.",
            "This size does not fit on this sheet.",
            "Dieses Format passt nicht auf dieses Blatt.",
            "Este formato no cabe en esta hoja.",
            "Questo formato non entra in questo foglio.",
            "Dit formaat past niet op dit vel.",
            "Este formato não cabe nesta folha.",
        ],
    ),
    (
        "ui_guides",
        [
            "Afficher les repères de coupe et de pli (épreuve)",
            "Show cutting and folding guides (proof)",
            "Schnitt- und Falzhilfen anzeigen (Probedruck)",
            "Mostrar guías de corte y plegado (prueba)",
            "Mostra le guide di taglio e piega (bozza)",
            "Snij- en vouwlijnen tonen (proef)",
            "Mostrar guias de corte e dobra (prova)",
        ],
    ),
    (
        "ui_warnings",
        [
            "Avant d’imprimer",
            "Before you print",
            "Vor dem Drucken",
            "Antes de imprimir",
            "Prima di stampare",
            "Voor je print",
            "Antes de imprimir",
        ],
    ),
    (
        "ui_no_warnings",
        [
            "Rien à signaler.",
            "Nothing to report.",
            "Keine Hinweise.",
            "Nada que señalar.",
            "Niente da segnalare.",
            "Niets te melden.",
            "Nada a assinalar.",
        ],
    ),
    (
        "ui_duplex_hint",
        [
            "Imprimez en recto verso, retournement sur le bord court, à 100\u{a0}% (sans mise à l’échelle).",
            "Print on both sides, flipping on the short edge, at 100 % (no scaling).",
            "Beidseitig drucken, an der kurzen Kante wenden, in 100 % (ohne Skalierung).",
            "Imprime a doble cara, volteando por el borde corto, al 100 % (sin ajustar el tamaño).",
            "Stampa fronte-retro, capovolgendo sul lato corto, al 100 % (senza ridimensionare).",
            "Print dubbelzijdig, omdraaien over de korte zijde, op 100 % (zonder schalen).",
            "Imprima frente e verso, virando pela margem curta, a 100 % (sem redimensionar).",
        ],
    ),
    (
        "ui_pdf_ready",
        [
            "Le PDF est prêt ({size}).",
            "The PDF is ready ({size}).",
            "Das PDF ist fertig ({size}).",
            "El PDF está listo ({size}).",
            "Il PDF è pronto ({size}).",
            "De pdf is klaar ({size}).",
            "O PDF está pronto ({size}).",
        ],
    ),
    (
        "ui_generating",
        [
            "Composition de la carte…",
            "Composing the card…",
            "Die Karte wird gesetzt …",
            "Componiendo la tarjeta…",
            "Composizione del biglietto…",
            "De kaart wordt opgemaakt…",
            "A compor o cartão…",
        ],
    ),
    (
        "ui_need_code",
        [
            "Créez le cadeau ou saisissez un code à l’étape 5.",
            "Create the gift or enter a code in step 5.",
            "Legen Sie in Schritt 5 das Geschenk an oder geben Sie einen Code ein.",
            "Crea el regalo o introduce un código en el paso 5.",
            "Crea il regalo o inserisci un codice al passaggio 5.",
            "Maak het cadeau aan of voer een code in bij stap 5.",
            "Crie a oferta ou introduza um código no passo 5.",
        ],
    ),
    (
        "ui_order_unavailable",
        [
            "La commande d’une carte imprimée n’est pas proposée sur ce site.",
            "Ordering a printed card is not offered on this site.",
            "Die Bestellung einer gedruckten Karte wird auf dieser Website nicht angeboten.",
            "Este sitio no ofrece el pedido de una tarjeta impresa.",
            "Questo sito non offre l’ordine di un biglietto stampato.",
            "Deze site biedt geen bestelling van een gedrukte kaart aan.",
            "Este sítio não propõe a encomenda de um cartão impresso.",
        ],
    ),
    (
        "ui_order_needs_pro",
        [
            "Pour commander, créez le cadeau sur ce site (étape 5) et choisissez une impression par un imprimeur.",
            "To order, create the gift on this site (step 5) and choose printing by a printer.",
            "Zum Bestellen legen Sie das Geschenk auf dieser Website an (Schritt 5) und wählen Sie den Druck durch eine Druckerei.",
            "Para pedir, crea el regalo en este sitio (paso 5) y elige la impresión en imprenta.",
            "Per ordinare, crea il regalo su questo sito (passaggio 5) e scegli la stampa in tipografia.",
            "Om te bestellen maak je het cadeau op deze site aan (stap 5) en kies je voor drukken bij een drukker.",
            "Para encomendar, crie a oferta neste sítio (passo 5) e escolha a impressão numa gráfica.",
        ],
    ),
    (
        "ui_order_address",
        [
            "Adresse de livraison",
            "Delivery address",
            "Lieferadresse",
            "Dirección de entrega",
            "Indirizzo di consegna",
            "Bezorgadres",
            "Morada de entrega",
        ],
    ),
    (
        "ui_address_name",
        [
            "Nom du destinataire",
            "Recipient’s name",
            "Name des Empfängers",
            "Nombre del destinatario",
            "Nome del destinatario",
            "Naam van de ontvanger",
            "Nome do destinatário",
        ],
    ),
    (
        "ui_address_line1",
        [
            "Numéro et rue",
            "Street address",
            "Straße und Hausnummer",
            "Calle y número",
            "Via e numero civico",
            "Straat en huisnummer",
            "Rua e número",
        ],
    ),
    (
        "ui_address_line2",
        [
            "Complément d’adresse (facultatif)",
            "Address line 2 (optional)",
            "Adresszusatz (optional)",
            "Información adicional (opcional)",
            "Altre indicazioni (facoltativo)",
            "Adresregel 2 (optioneel)",
            "Complemento de morada (opcional)",
        ],
    ),
    (
        "ui_address_postal_code",
        [
            "Code postal",
            "Postal code",
            "Postleitzahl",
            "Código postal",
            "CAP",
            "Postcode",
            "Código postal",
        ],
    ),
    ("ui_address_city", ["Ville", "City", "Ort", "Ciudad", "Città", "Plaats", "Localidade"]),
    ("ui_address_country", ["Pays", "Country", "Land", "País", "Paese", "Land", "País"]),
    (
        "ui_order_address_incomplete",
        [
            "Indiquez le nom, la rue, le code postal et la ville.",
            "Enter the name, street, postal code and city.",
            "Geben Sie Name, Straße, Postleitzahl und Ort an.",
            "Indica el nombre, la calle, el código postal y la ciudad.",
            "Indica nome, via, CAP e città.",
            "Vul de naam, straat, postcode en plaats in.",
            "Indique o nome, a rua, o código postal e a localidade.",
        ],
    ),
    (
        "ui_order_rejected",
        [
            "L’imprimeur a refusé la commande\u{a0}: {reason}",
            "The printer refused the order: {reason}",
            "Die Druckerei hat die Bestellung abgelehnt: {reason}",
            "La imprenta ha rechazado el pedido: {reason}",
            "La tipografia ha rifiutato l’ordine: {reason}",
            "De drukker heeft de bestelling geweigerd: {reason}",
            "A gráfica recusou a encomenda: {reason}",
        ],
    ),
    (
        "ui_order_country_refused",
        [
            "Cet imprimeur ne livre pas dans ce pays.",
            "This printer does not deliver to that country.",
            "Diese Druckerei liefert nicht in dieses Land.",
            "Esta imprenta no envía a ese país.",
            "Questa tipografia non spedisce in quel paese.",
            "Deze drukker levert niet in dat land.",
            "Esta gráfica não entrega nesse país.",
        ],
    ),
    (
        "err_invalid_address",
        [
            "L’adresse est incomplète ou incorrecte\u{a0}: vérifiez le champ signalé.",
            "The address is incomplete or incorrect: check the marked field.",
            "Die Adresse ist unvollständig oder fehlerhaft: Prüfen Sie das markierte Feld.",
            "La dirección está incompleta o es incorrecta: revisa el campo señalado.",
            "L’indirizzo è incompleto o errato: controlla il campo segnalato.",
            "Het adres is onvolledig of onjuist: controleer het gemarkeerde veld.",
            "A morada está incompleta ou incorreta: verifique o campo assinalado.",
        ],
    ),
    (
        "err_printer_unavailable",
        [
            "L’imprimeur ne répond pas pour le moment. Réessayez plus tard.",
            "The printer is not answering right now. Try again later.",
            "Die Druckerei antwortet gerade nicht. Versuchen Sie es später erneut.",
            "La imprenta no responde en este momento. Inténtalo más tarde.",
            "La tipografia non risponde al momento. Riprova più tardi.",
            "De drukker reageert nu niet. Probeer het later opnieuw.",
            "A gráfica não responde de momento. Tente mais tarde.",
        ],
    ),
    (
        "ui_order_sent",
        [
            "Commande transmise\u{a0}: {id}",
            "Order sent: {id}",
            "Bestellung übermittelt: {id}",
            "Pedido enviado: {id}",
            "Ordine inviato: {id}",
            "Bestelling verzonden: {id}",
            "Encomenda enviada: {id}",
        ],
    ),
    (
        "ui_preview_outside",
        [
            "Extérieur\u{a0}: dos de couverture, dos, couverture",
            "Outside: back, spine, front",
            "Außen: Rückseite, Rücken, Vorderseite",
            "Exterior: contracubierta, lomo, cubierta",
            "Esterno: quarta, dorso, copertina",
            "Buitenkant: achterzijde, rug, voorzijde",
            "Exterior: contracapa, lombada, capa",
        ],
    ),
    (
        "ui_preview_inside",
        ["Intérieur", "Inside", "Innen", "Interior", "Interno", "Binnenkant", "Interior"],
    ),
    (
        "ui_preview_updating",
        [
            "Mise à jour de l’aperçu…",
            "Updating the preview…",
            "Vorschau wird aktualisiert …",
            "Actualizando la vista previa…",
            "Aggiornamento dell’anteprima…",
            "Voorbeeld wordt bijgewerkt…",
            "A atualizar a pré-visualização…",
        ],
    ),
    (
        "ui_preview_empty",
        [
            "L’aperçu apparaît dès que le titre est saisi.",
            "The preview appears as soon as the title is entered.",
            "Die Vorschau erscheint, sobald der Titel eingegeben ist.",
            "La vista previa aparece en cuanto se introduce el título.",
            "L’anteprima appare appena inserito il titolo.",
            "Het voorbeeld verschijnt zodra de titel is ingevuld.",
            "A pré-visualização aparece assim que o título é introduzido.",
        ],
    ),
    (
        "ui_preview_failed",
        [
            "L’aperçu n’a pas pu être dessiné\u{a0}: {reason}",
            "The preview could not be drawn: {reason}",
            "Die Vorschau konnte nicht gezeichnet werden: {reason}",
            "No se pudo dibujar la vista previa: {reason}",
            "Non è stato possibile disegnare l’anteprima: {reason}",
            "Het voorbeeld kon niet worden getekend: {reason}",
            "Não foi possível desenhar a pré-visualização: {reason}",
        ],
    ),
    // ---- Warnings of the composer ------------------------------------------------------
    (
        "warn_cover_without_rights",
        [
            "Aucune déclaration n’a été choisie pour l’image\u{a0}: elle n’est pas imprimée.",
            "No statement was chosen for the image: it is not printed.",
            "Für das Bild wurde keine Erklärung gewählt: Es wird nicht gedruckt.",
            "No se ha elegido ninguna declaración para la imagen: no se imprime.",
            "Non è stata scelta alcuna dichiarazione per l’immagine: non viene stampata.",
            "Er is geen verklaring gekozen voor de afbeelding: ze wordt niet geprint.",
            "Não foi escolhida nenhuma declaração para a imagem: não é impressa.",
        ],
    ),
    (
        "warn_low_resolution",
        [
            "L’image s’imprime à {ppi}\u{a0}ppp\u{a0}; {threshold}\u{a0}ppp sont recommandés.",
            "The image prints at {ppi} ppi; {threshold} ppi is recommended.",
            "Das Bild wird mit {ppi} ppi gedruckt; empfohlen sind {threshold} ppi.",
            "La imagen se imprime a {ppi} ppp; se recomiendan {threshold} ppp.",
            "L’immagine viene stampata a {ppi} ppi; se ne consigliano {threshold}.",
            "De afbeelding wordt geprint op {ppi} ppi; {threshold} ppi is aanbevolen.",
            "A imagem é impressa a {ppi} ppp; recomendam-se {threshold} ppp.",
        ],
    ),
    (
        "warn_resolution_below_minimum",
        [
            "L’image s’imprime à {ppi}\u{a0}ppp, sous le minimum de {threshold}\u{a0}ppp\u{a0}: elle sera floue.",
            "The image prints at {ppi} ppi, below the {threshold} ppi minimum: it will look blurred.",
            "Das Bild wird mit {ppi} ppi gedruckt, unter dem Minimum von {threshold} ppi: Es wird unscharf.",
            "La imagen se imprime a {ppi} ppp, por debajo del mínimo de {threshold} ppp: saldrá borrosa.",
            "L’immagine viene stampata a {ppi} ppi, sotto il minimo di {threshold} ppi: risulterà sfocata.",
            "De afbeelding wordt geprint op {ppi} ppi, onder het minimum van {threshold} ppi: ze wordt onscherp.",
            "A imagem é impressa a {ppi} ppp, abaixo do mínimo de {threshold} ppp: ficará desfocada.",
        ],
    ),
    (
        "warn_text_clamped",
        [
            "Un texte était trop long et a été coupé ({field}).",
            "A text was too long and was cut ({field}).",
            "Ein Text war zu lang und wurde gekürzt ({field}).",
            "Un texto era demasiado largo y se ha cortado ({field}).",
            "Un testo era troppo lungo ed è stato tagliato ({field}).",
            "Een tekst was te lang en is ingekort ({field}).",
            "Um texto era demasiado longo e foi cortado ({field}).",
        ],
    ),
    (
        "warn_missing_glyphs",
        [
            "Certains caractères ne peuvent pas être imprimés et ont été omis\u{a0}: {characters}",
            "Some characters cannot be printed and were left out: {characters}",
            "Einige Zeichen können nicht gedruckt werden und wurden weggelassen: {characters}",
            "Algunos caracteres no pueden imprimirse y se han omitido: {characters}",
            "Alcuni caratteri non si possono stampare e sono stati omessi: {characters}",
            "Sommige tekens kunnen niet worden geprint en zijn weggelaten: {characters}",
            "Alguns caracteres não podem ser impressos e foram omitidos: {characters}",
        ],
    ),
    (
        "warn_guides_included",
        [
            "Les repères sont affichés\u{a0}: ce fichier est une épreuve, pas le fichier à imprimer.",
            "The guides are shown: this file is a proof, not the file to print.",
            "Die Hilfslinien sind eingeblendet: Diese Datei ist ein Probedruck, nicht die Druckdatei.",
            "Las guías están visibles: este archivo es una prueba, no el archivo para imprimir.",
            "Le guide sono visibili: questo file è una bozza, non il file da stampare.",
            "De hulplijnen zijn zichtbaar: dit bestand is een proef, niet het bestand om te printen.",
            "As guias estão visíveis: este ficheiro é uma prova, não o ficheiro a imprimir.",
        ],
    ),
    (
        "warn_fallback_output_profile",
        [
            "Le profil colorimétrique pour l’imprimeur n’a pas pu être chargé\u{a0}; un profil générique a été utilisé.",
            "The colour profile for the printer could not be loaded; a generic one was used.",
            "Das Farbprofil für die Druckerei konnte nicht geladen werden; es wurde ein allgemeines verwendet.",
            "No se pudo cargar el perfil de color para la imprenta; se ha usado uno genérico.",
            "Non è stato possibile caricare il profilo colore per la tipografia; ne è stato usato uno generico.",
            "Het kleurprofiel voor de drukker kon niet worden geladen; er is een algemeen profiel gebruikt.",
            "Não foi possível carregar o perfil de cor para a gráfica; foi usado um genérico.",
        ],
    ),
    (
        "warn_cover_not_converted",
        [
            "L’image est incorporée en sRVB\u{a0}; l’imprimeur la convertit vers ses encres.",
            "The image is embedded as sRGB; the printer converts it to its inks.",
            "Das Bild ist als sRGB eingebettet; die Druckerei wandelt es in ihre Druckfarben um.",
            "La imagen se incrusta en sRGB; la imprenta la convierte a sus tintas.",
            "L’immagine è incorporata in sRGB; la tipografia la converte nei propri inchiostri.",
            "De afbeelding is als sRGB ingesloten; de drukker zet ze om naar zijn inkten.",
            "A imagem é incorporada em sRGB; a gráfica converte-a para as suas tintas.",
        ],
    ),
    (
        "warn_cover_credit_missing",
        [
            "Aucun crédit n’a été indiqué pour l’image\u{a0}: une mention générique est imprimée.",
            "No credit line was given for the image: a generic one is printed.",
            "Für das Bild wurde kein Nachweis angegeben: Es wird ein allgemeiner gedruckt.",
            "No se ha indicado ningún crédito para la imagen: se imprime uno genérico.",
            "Non è stato indicato alcun credito per l’immagine: ne viene stampato uno generico.",
            "Er is geen bronvermelding opgegeven voor de afbeelding: er wordt een algemene geprint.",
            "Não foi indicado nenhum crédito para a imagem: é impressa uma menção genérica.",
        ],
    ),
    (
        "warn_spine_choice_ignored",
        [
            "Ce style de dos demande une image imprimable\u{a0}: le dos standard a été utilisé.",
            "This spine style needs a printable image: the standard spine was used.",
            "Dieser Rückenstil braucht ein druckbares Bild: Der Standardrücken wurde verwendet.",
            "Este estilo de lomo necesita una imagen imprimible: se ha usado el lomo estándar.",
            "Questo stile di dorso richiede un’immagine stampabile: è stato usato il dorso standard.",
            "Deze rugstijl vraagt een printbare afbeelding: de standaardrug is gebruikt.",
            "Este estilo de lombada exige uma imagem imprimível: foi usada a lombada padrão.",
        ],
    ),
    (
        "warn_dark_across_creases",
        [
            "Encre foncée sur les plis (jusqu’à {ink}\u{a0}% d’encre)\u{a0}: demandez à l’imprimeur un pelliculage mat ou soft touch et des fibres parallèles au dos, sinon les plis peuvent se fissurer.",
            "Dark ink across the folds (up to {ink} % ink): ask the printer for matt or soft-touch lamination and the paper grain parallel to the spine, or the folds may crack.",
            "Dunkle Farbe auf den Falzen (bis {ink} % Farbauftrag): Bitten Sie die Druckerei um eine matte oder Soft-Touch-Kaschierung und die Laufrichtung parallel zum Rücken, sonst können die Falze brechen.",
            "Tinta oscura en los pliegues (hasta {ink} % de tinta): pida a la imprenta un laminado mate o soft touch y la fibra del papel paralela al lomo, o los pliegues pueden agrietarse.",
            "Inchiostro scuro sulle pieghe (fino al {ink} % di copertura): chiedete alla tipografia una plastificazione opaca o soft touch e la fibra della carta parallela al dorso, altrimenti le pieghe possono creparsi.",
            "Donkere inkt op de vouwen (tot {ink} % inkt): vraag de drukker om matte of soft-touch laminering en de looprichting van het papier evenwijdig aan de rug, anders kunnen de vouwen barsten.",
            "Tinta escura nas dobras (até {ink} % de tinta): peça à gráfica uma plastificação mate ou soft touch e a fibra do papel paralela à lombada, ou as dobras podem estalar.",
        ],
    ),
    // ---- Errors ------------------------------------------------------------------------
    (
        "err_invalid_code",
        [
            "Ce code contient une faute de frappe.",
            "This code contains a typing mistake.",
            "Dieser Code enthält einen Tippfehler.",
            "Este código contiene un error de escritura.",
            "Questo codice contiene un errore di battitura.",
            "Deze code bevat een typefout.",
            "Este código contém um erro de digitação.",
        ],
    ),
    (
        "err_invalid_base_url",
        [
            "L’adresse du site doit commencer par https:// et ne contenir que le nom du site.",
            "The site address must start with https:// and hold the site’s name only.",
            "Die Adresse der Website muss mit https:// beginnen und nur den Namen der Website enthalten.",
            "La dirección del sitio debe empezar por https:// y contener solo el nombre del sitio.",
            "L’indirizzo del sito deve iniziare con https:// e contenere solo il nome del sito.",
            "Het adres van de site moet met https:// beginnen en alleen de naam van de site bevatten.",
            "O endereço do sítio deve começar por https:// e conter apenas o nome do sítio.",
        ],
    ),
    (
        "err_format_too_small",
        [
            "Ce format est trop petit pour le contenu de la carte.",
            "This size is too small for what the card must carry.",
            "Dieses Format ist zu klein für den Inhalt der Karte.",
            "Este formato es demasiado pequeño para el contenido de la tarjeta.",
            "Questo formato è troppo piccolo per il contenuto del biglietto.",
            "Dit formaat is te klein voor de inhoud van de kaart.",
            "Este formato é demasiado pequeno para o conteúdo do cartão.",
        ],
    ),
    (
        "err_text_too_long",
        [
            "Un texte est trop long.",
            "A text is too long.",
            "Ein Text ist zu lang.",
            "Un texto es demasiado largo.",
            "Un testo è troppo lungo.",
            "Een tekst is te lang.",
            "Um texto é demasiado longo.",
        ],
    ),
];

fn locale_of(tag: &str) -> Result<Locale, ApiError> {
    if tag.len() > 35 {
        return Err(ApiError::new("unknown_locale", "not a language tag"));
    }
    Locale::from_tag(tag)
        .ok_or_else(|| ApiError::new("unknown_locale", "expected fr, en, de, es, it, nl or pt"))
}

fn column(locale: Locale) -> usize {
    Locale::ALL.iter().position(|l| *l == locale).unwrap_or(0)
}

fn ui_string(locale: Locale, key: &str) -> Option<&'static str> {
    UI_STRINGS.iter().find(|(k, _)| *k == key).map(|(_, texts)| texts[column(locale)])
}

/// One string of the page: a key of `ebc-i18n` or of [`UI_STRINGS`], in `locale`
/// (`"fr"`, `"en-GB"`, …).
///
/// # Errors
/// `unknown_locale`, `unknown_key`.
pub fn i18n_t(locale: &str, key: &str) -> Result<String, ApiError> {
    let locale = locale_of(locale)?;
    if let Some(msg) = Msg::from_key(key) {
        return Ok(t(locale, msg).to_owned());
    }
    ui_string(locale, key)
        .map(str::to_owned)
        .ok_or_else(|| ApiError::new("unknown_key", "no string has this key"))
}

/// Every string of the page in `locale`, as a JSON object `{key: text}`.
///
/// # Errors
/// `unknown_locale`.
pub fn i18n_catalog(locale: &str) -> Result<String, ApiError> {
    let locale = locale_of(locale)?;
    let mut all: BTreeMap<&str, &str> =
        Msg::ALL.iter().map(|msg| (msg.key(), t(locale, *msg))).collect();
    let column = column(locale);
    for (key, texts) in UI_STRINGS {
        all.insert(key, texts[column]);
    }
    to_json(&all)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use ebc_i18n::placeholders_in;

    use super::*;

    #[test]
    fn the_columns_follow_the_locales() {
        assert_eq!(Locale::ALL.len(), LOCALES);
        assert_eq!(Locale::ALL.map(Locale::code), ["fr", "en", "de", "es", "it", "nl", "pt"]);
        assert_eq!(i18n_t("en", "ui_step_book").expect("en"), "The book");
        assert_eq!(i18n_t("fr-CA", "ui_step_book").expect("fr"), "Le livre");
        assert_eq!(i18n_t("pt", "ui_step_card").expect("pt"), "O cartão");
        // Keys of ebc-i18n are served too.
        assert_eq!(i18n_t("fr", "lookup_cta").expect("fr"), "Rechercher");
    }

    #[test]
    fn every_string_exists_in_every_language_with_the_same_placeholders() {
        let mut keys = BTreeSet::new();
        for (key, texts) in UI_STRINGS {
            assert!(keys.insert(*key), "duplicate key {key}");
            assert!(
                key.starts_with("ui_") || key.starts_with("warn_") || key.starts_with("err_"),
                "{key} must not shadow a key of ebc-i18n"
            );
            assert!(Msg::from_key(key).is_none(), "{key} shadows ebc-i18n");
            let expected: BTreeSet<&str> = placeholders_in(texts[0]).into_iter().collect();
            for (locale, text) in Locale::ALL.iter().zip(texts) {
                assert!(!text.trim().is_empty(), "{key} is empty in {locale:?}");
                assert_eq!(text.trim(), *text, "{key} has stray white space in {locale:?}");
                let found: BTreeSet<&str> = placeholders_in(text).into_iter().collect();
                assert_eq!(found, expected, "{key}: placeholders differ in {locale:?}");
                assert!(!text.contains('<') && !text.contains('>'), "{key}: no markup in strings");
            }
        }
        assert!(UI_STRINGS.len() > 100);
    }

    #[test]
    fn the_catalogue_is_complete_for_every_locale() {
        let expected = Msg::ALL.len() + UI_STRINGS.len();
        for locale in Locale::ALL {
            let json = i18n_catalog(locale.code()).expect("catalogue");
            let map: BTreeMap<String, String> = serde_json::from_str(&json).expect("an object");
            assert_eq!(map.len(), expected, "{locale:?}");
            assert!(map.values().all(|text| !text.is_empty()));
            assert_eq!(map["ui_step_spine"], i18n_t(locale.code(), "ui_step_spine").expect("key"));
        }
        assert_eq!(i18n_catalog("tlh").expect_err("klingon").code, "unknown_locale");
        assert_eq!(i18n_catalog(&"f".repeat(1000)).expect_err("long").code, "unknown_locale");
        assert_eq!(i18n_t("fr", "no_such_key").expect_err("key").code, "unknown_key");
    }

    /// The wording rules of `ebc-i18n`: the gift is one prepaid book, never a balance.
    #[test]
    fn no_stored_value_words() {
        let banned: [&[&str]; LOCALES] = [
            &[
                "carte cadeau",
                "carte-cadeau",
                "chèque cadeau",
                "bon cadeau",
                "bon d’achat",
                "valable jusqu",
            ],
            &["gift card", "voucher", "token", "coupon", "valid until"],
            &["gutschein", "geschenkkarte"],
            &["tarjeta regalo", "tarjeta de regalo", "cheque regalo", " bono", "cupón"],
            &["buono", "buoni", "carta regalo", "gift card"],
            &["cadeaubon", "cadeaukaart", "boekenbon", "tegoedbon"],
            &["cartão oferta", "cartão de oferta", "cartão presente", "cheque", " vale"],
        ];
        for (key, texts) in UI_STRINGS {
            for (column, text) in texts.iter().enumerate() {
                let lower = text.to_lowercase();
                for word in banned[column] {
                    assert!(!lower.contains(word), "{key} ({column}) contains {word:?}: {text}");
                }
                assert!(!lower.contains("qr"), "{key}: the page says \"code\", never QR");
            }
        }
    }

    /// The page never presents itself as a way to print a book's cover: what may be
    /// printed is "an image" the user has the right to print.
    #[test]
    fn the_cover_step_offers_an_image_not_the_books_cover() {
        for key in ["ui_cover_own", "ui_cover_own_help", "ui_choose_image", "ui_rights_legend"] {
            let (_, texts) = UI_STRINGS.iter().find(|(k, _)| *k == key).expect("key");
            for word in
                ["couverture", "cover", "umschlag", "cubierta", "copertina", "omslag", "capa"]
            {
                for text in texts {
                    assert!(!text.to_lowercase().contains(word), "{key}: {text}");
                }
            }
        }
        // French typography: a no-break space before double punctuation.
        for (key, texts) in UI_STRINGS {
            for mark in [" ?", " !", " :", " ;"] {
                assert!(!texts[0].contains(mark), "{key}: use U+00A0 before {mark:?} in French");
            }
        }
    }
}
