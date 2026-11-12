//! French (fr-FR). The printed card uses the infinitive, so that it chooses neither *tu*
//! nor *vous* on behalf of the giver; screens address the reader with *vous*. Running text
//! says « livre numérique », the checkout option says « ebook » (the trade form).
//! No-break space (U+00A0) before `? ! : ;`.

use crate::Msg;

pub(super) fn text(msg: Msg) -> &'static str {
    match msg {
        Msg::ProductName => "eBook Cadeau",
        Msg::ProductTagline => "Un livre numérique à offrir, comme un vrai livre.",
        Msg::CheckoutOption => "Offrir cet ebook",
        Msg::CheckoutOptionHelp => {
            "Une carte en forme de livre, à imprimer chez soi ou envoyée par la poste. La personne qui la reçoit scanne le code et lit le livre sur liseuse, tablette ou téléphone."
        }
        Msg::CheckoutPrintAtHome => "Carte à imprimer chez moi (gratuit)",
        Msg::CheckoutPrintedByPost => "Carte imprimée et envoyée par la poste",

        Msg::CardHeadline => "Ce livre est un cadeau",
        Msg::CardFrom => "De la part {de:name}",
        Msg::CardTo => "Pour {name}",
        Msg::CardHowTo => "Pour le lire",
        Msg::CardStep1 => "Scanner ce code avec l’appareil photo d’un téléphone.",
        Msg::CardStep2 => "Choisir où lire\u{a0}: liseuse, tablette ou téléphone.",
        Msg::CardStep3 => "Le livre arrive. Bonne lecture\u{a0}!",
        Msg::CardStep2Reseller => "Se connecter à son compte {store}, ou en créer un.",
        Msg::CardStep3Reseller => {
            "Le livre apparaît dans la bibliothèque, sur la liseuse et l’application."
        }
        Msg::CardNoPhone => {
            "Sans téléphone\u{a0}? Sur la liseuse ou un ordinateur, ouvrir le navigateur, aller sur {url} et saisir le code de cette carte."
        }
        Msg::CodeLabel => "Code",
        Msg::CardAlreadyPaid => {
            "Ce livre numérique est déjà payé\u{a0}: il ne reste qu’à le lire, sur liseuse, tablette ou téléphone."
        }
        Msg::CardValidUntil => "À récupérer avant le {date}.",
        Msg::CardNoExpiry => "Sans date limite.",
        Msg::CardHelp => "Une aide\u{a0}? {url}",
        Msg::CardBackDefault => {
            "Ce livre est numérique. Ouvrir cette carte pour le recevoir sur liseuse, tablette ou téléphone."
        }
        Msg::CardLegalLine => {
            "{title}, {author}. Un exemplaire numérique, déjà payé\u{a0}: rien à régler."
        }
        Msg::CardLegalLineNoAuthor => {
            "{title}. Un exemplaire numérique, déjà payé\u{a0}: rien à régler."
        }
        Msg::CardSingleUse => "Usage unique.",
        Msg::FoldHere => "Plier ici",
        Msg::CutHere => "Couper ici",
        Msg::PrintedWith => "Carte créée avec eBook Cadeau",
        Msg::EbookWord => "livre numérique",
        Msg::EreaderWord => "liseuse",
        Msg::QrWord => "QR code",

        Msg::RedeemTitle => "Recevoir votre livre",
        Msg::RedeemIntro => "Saisissez le code imprimé à l’intérieur de votre carte.",
        Msg::RedeemCta => "Récupérer mon livre",
        Msg::RedeemGiftFrom => "{name} vous offre ce livre.",
        Msg::RedeemGiftAnonymous => "Ce livre vous est offert.",
        Msg::ChooseDevice => "Où souhaitez-vous lire\u{a0}?",
        Msg::DeviceThis => "Sur cet appareil",
        Msg::DeviceEreader => "Sur une liseuse",
        Msg::DevicePhoneTablet => "Sur un téléphone ou une tablette",
        Msg::DeviceComputer => "Sur un ordinateur",
        Msg::DownloadCta => "Télécharger le livre",
        Msg::ReadNowCta => "Lire maintenant",
        Msg::SendToEreaderCta => "Envoyer vers ma liseuse",
        Msg::AddToLibraryCta => "Ajouter à ma bibliothèque",
        Msg::ContinueAtStore => "Continuer sur {store}",
        Msg::EreaderBrowserHint => {
            "Sur votre liseuse, ouvrez le navigateur web, allez sur {url} et saisissez le code de la carte."
        }
        Msg::EmailLabel => "Adresse e-mail",
        Msg::SendCta => "Envoyer",
        Msg::SentConfirmation => {
            "C’est envoyé. Le livre arrivera sur la liseuse d’ici quelques minutes."
        }
        Msg::AlreadyRedeemed => {
            "Ce livre a déjà été récupéré. Vous pouvez le télécharger à nouveau."
        }
        Msg::EnjoyReading => "Bonne lecture\u{a0}!",
        Msg::ErrorCodeTypo => "Ce code contient une faute de frappe. Vérifiez-le et réessayez.",
        Msg::ErrorCodeFormat => "Le code comporte {count} lettres, sans chiffre ni voyelle.",
        Msg::ErrorCodeUnknown => "Ce code est inconnu. Vérifiez-le sur votre carte.",
        Msg::ErrorGiftExpired => {
            "Ce cadeau n’est plus disponible\u{a0}: sa date limite est dépassée."
        }
        Msg::ErrorGiftRevoked => {
            "Ce cadeau a été annulé. Contactez la personne qui vous l’a offert."
        }
        Msg::ErrorGiftPending => {
            "Ce cadeau n’est pas encore prêt. Réessayez dans quelques minutes."
        }
        Msg::ErrorTooManyAttempts => "Trop de tentatives. Réessayez dans quelques minutes.",
        Msg::ErrorGeneric => "Une erreur est survenue. Réessayez plus tard.",

        Msg::CreateTitle => "Créer un eBook Cadeau",
        Msg::FieldIsbn => "ISBN du livre",
        Msg::FieldTitle => "Titre",
        Msg::FieldAuthor => "Auteur",
        Msg::FieldGiver => "De la part de",
        Msg::FieldRecipient => "Pour",
        Msg::FieldMessage => "Message personnel",
        Msg::FieldCover => "Image de couverture",
        Msg::FieldFile => "Fichier du livre (EPUB ou PDF)",
        Msg::LookupCta => "Rechercher",
        Msg::GenerateCta => "Créer la carte",
        Msg::DownloadPdfCta => "Télécharger le PDF",
        Msg::PrintAtHomeCta => "Imprimer chez moi",
        Msg::OrderPrintCta => "Commander l’impression",
        Msg::PreviewLabel => "Aperçu",
        Msg::SpineLabel => "Dos",
        Msg::FrontLabel => "Première de couverture",
        Msg::BackLabel => "Quatrième de couverture",
        Msg::InsideLabel => "Intérieur",
        Msg::RightsBasisLabel => "À quel titre distribuez-vous ce livre\u{a0}?",
        Msg::RightsOwnWork => "C’est mon propre ouvrage",
        Msg::RightsPublicDomain => "Il est dans le domaine public",
        Msg::RightsOpenLicence => "Il est sous licence libre",
        Msg::RightsLicensedDistributor => "J’en suis le distributeur autorisé",
        Msg::RightsConfirm => {
            "Je confirme avoir le droit de distribuer ce fichier et d’utiliser cette image de couverture."
        }
        Msg::UploadPurchasedNotice => {
            "Un livre numérique acheté en boutique est une licence personnelle\u{a0}: il ne peut pas être transmis ici. Pour l’offrir, utilisez la fonction cadeau de la boutique."
        }
        Msg::CoverNotFound => {
            "Aucune couverture trouvée. Importez une image ou utilisez la couverture typographique."
        }
        Msg::IsbnInvalid => "Cet ISBN n’est pas valide.",
        Msg::BookNotFound => "Aucun livre trouvé pour cet ISBN.",
        Msg::Loading => "Chargement…",
        Msg::LanguageLabel => "Langue",
        Msg::DemoNotice => "Démonstration — boutique fictive, sans lien avec une enseigne réelle.",
    }
}
