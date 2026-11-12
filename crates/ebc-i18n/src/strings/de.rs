//! German (de-DE). Register: the pronoun-free infinitive of instructions, so that the
//! card chooses neither *du* nor *Sie*. No "ß", so the strings also work in Switzerland.
//! Trade spellings „eBook" and „eReader" (`docs/research/06-terminology.md` §1.6).

use crate::Msg;

pub(super) fn text(msg: Msg) -> &'static str {
    match msg {
        Msg::ProductName => "eBook-Geschenk",
        Msg::ProductTagline => "Ein eBook zum Verschenken – wie ein echtes Buch.",
        Msg::CheckoutOption => "eBook verschenken",
        Msg::CheckoutOptionHelp => {
            "Eine Karte in Buchform, zum Selbstausdrucken oder per Post verschickt. Wer sie bekommt, scannt den Code und liest das Buch auf eReader, Tablet oder Smartphone."
        }
        Msg::CheckoutPrintAtHome => "Karte zum Selbstausdrucken (kostenlos)",
        Msg::CheckoutPrintedByPost => "Gedruckte Karte per Post",

        Msg::CardHeadline => "Dieses Buch ist ein Geschenk",
        Msg::CardFrom => "Von {name}",
        Msg::CardTo => "Für {name}",
        Msg::CardHowTo => "So geht’s",
        Msg::CardStep1 => "Diesen Code mit der Kamera eines Smartphones scannen.",
        Msg::CardStep2 => "Auswählen, wo gelesen wird: eReader, Tablet oder Smartphone.",
        Msg::CardStep3 => "Das Buch ist da. Viel Freude beim Lesen!",
        Msg::CardStep2Reseller => "Im {store}-Konto anmelden oder ein Konto anlegen.",
        Msg::CardStep3Reseller => {
            "Das Buch erscheint in der Bibliothek, auf dem eReader und in der App."
        }
        Msg::CardNoPhone => {
            "Kein Smartphone? Auf dem eReader oder am Computer den Browser öffnen, {url} aufrufen und den Code dieser Karte eingeben."
        }
        Msg::CodeLabel => "Code",
        Msg::CardAlreadyPaid => {
            "Dieses eBook ist bereits bezahlt – einfach lesen: auf eReader, Tablet oder Smartphone."
        }
        Msg::CardValidUntil => "Abholen bis {date}.",
        Msg::CardNoExpiry => "Zeitlich unbegrenzt.",
        Msg::CardHelp => "Hilfe: {url}",
        Msg::CardBackDefault => {
            "Dieses Buch ist digital. Karte öffnen und auf eReader, Tablet oder Smartphone lesen."
        }
        Msg::CardLegalLine => {
            "{title}, {author}. Ein digitales Exemplar, bereits bezahlt: nichts mehr zu zahlen."
        }
        Msg::CardLegalLineNoAuthor => {
            "{title}. Ein digitales Exemplar, bereits bezahlt: nichts mehr zu zahlen."
        }
        Msg::CardSingleUse => "Einmalig verwendbar.",
        Msg::FoldHere => "Hier falten",
        Msg::CutHere => "Hier schneiden",
        Msg::PrintedWith => "Karte erstellt mit eBook Cadeau",
        Msg::EbookWord => "eBook",
        Msg::EreaderWord => "eReader",
        Msg::QrWord => "QR-Code",

        Msg::PrintDuplexShortEdge => {
            "Mit 100\u{a0}% (ohne Skalierung) beidseitig drucken, an der kurzen Kante spiegeln."
        }
        Msg::PrintDuplexLongEdge => {
            "Mit 100\u{a0}% (ohne Skalierung) beidseitig drucken, an der langen Kante spiegeln."
        }
        Msg::PrintCutFold => "An den durchgezogenen Marken schneiden, an den gestrichelten falzen.",
        Msg::CoverCredit => "Umschlagabbildung: {credit}",
        Msg::CoverCreditPublisher => "Umschlagabbildung: © {publisher}",
        Msg::CoverCreditPublicDomain => "Umschlagabbildung: gemeinfrei",
        Msg::CoverCreditUnknown => "Umschlagabbildung: alle Rechte vorbehalten",
        Msg::CardFormatLine => "Format {width} × {height} mm, Rücken {spine} mm",
        Msg::OutsideLabel => "Vorder- und Rückseite",

        Msg::RedeemTitle => "Buch abholen",
        Msg::RedeemIntro => "Bitte den Code von der Innenseite der Karte eingeben.",
        Msg::RedeemCta => "Mein Buch abholen",
        Msg::RedeemGiftFrom => "Dieses Buch ist ein Geschenk von {name}.",
        Msg::RedeemGiftAnonymous => "Dieses Buch ist ein Geschenk.",
        Msg::ChooseDevice => "Wo soll das Buch gelesen werden?",
        Msg::DeviceThis => "Auf diesem Gerät",
        Msg::DeviceEreader => "Auf einem eReader",
        Msg::DevicePhoneTablet => "Auf Smartphone oder Tablet",
        Msg::DeviceComputer => "Auf einem Computer",
        Msg::DownloadCta => "Buch herunterladen",
        Msg::ReadNowCta => "Jetzt lesen",
        Msg::SendToEreaderCta => "An meinen eReader senden",
        Msg::AddToLibraryCta => "Zu meiner Bibliothek hinzufügen",
        Msg::ContinueAtStore => "Weiter bei {store}",
        Msg::EreaderBrowserHint => {
            "Auf dem eReader den Browser öffnen, {url} aufrufen und den Code von der Karte eingeben."
        }
        Msg::EmailLabel => "E-Mail-Adresse",
        Msg::SendCta => "Senden",
        Msg::SentConfirmation => "Gesendet. Das Buch erscheint in wenigen Minuten auf dem eReader.",
        Msg::AlreadyRedeemed => {
            "Dieses Buch wurde bereits abgeholt. Es kann erneut heruntergeladen werden."
        }
        Msg::EnjoyReading => "Viel Freude beim Lesen!",
        Msg::ErrorCodeTypo => {
            "Dieser Code enthält einen Tippfehler. Bitte prüfen und erneut eingeben."
        }
        Msg::ErrorCodeFormat => {
            "Der Code besteht aus {count} Buchstaben, ohne Ziffern und ohne Vokale."
        }
        Msg::ErrorCodeUnknown => "Dieser Code ist unbekannt. Bitte mit der Karte vergleichen.",
        Msg::ErrorGiftExpired => {
            "Dieses Geschenk ist nicht mehr verfügbar: Die Frist ist abgelaufen."
        }
        Msg::ErrorGiftRevoked => {
            "Dieses Geschenk wurde storniert. Bitte an die Person wenden, von der es stammt."
        }
        Msg::ErrorGiftPending => {
            "Dieses Geschenk ist noch nicht bereit. Bitte in einigen Minuten erneut versuchen."
        }
        Msg::ErrorTooManyAttempts => {
            "Zu viele Versuche. Bitte in einigen Minuten erneut versuchen."
        }
        Msg::ErrorGeneric => "Ein Fehler ist aufgetreten. Bitte später erneut versuchen.",

        Msg::CreateTitle => "eBook-Geschenk erstellen",
        Msg::FieldIsbn => "ISBN des Buchs",
        Msg::FieldTitle => "Titel",
        Msg::FieldAuthor => "Autor/in",
        Msg::FieldGiver => "Von",
        Msg::FieldRecipient => "Für",
        Msg::FieldMessage => "Persönliche Nachricht",
        Msg::FieldCover => "Coverbild",
        Msg::FieldFile => "Buchdatei (EPUB oder PDF)",
        Msg::LookupCta => "Suchen",
        Msg::GenerateCta => "Karte erstellen",
        Msg::DownloadPdfCta => "PDF herunterladen",
        Msg::PrintAtHomeCta => "Selbst ausdrucken",
        Msg::OrderPrintCta => "Druck bestellen",
        Msg::PreviewLabel => "Vorschau",
        Msg::SpineLabel => "Buchrücken",
        Msg::FrontLabel => "Vorderseite",
        Msg::BackLabel => "Rückseite",
        Msg::InsideLabel => "Innenseite",
        Msg::RightsBasisLabel => "Auf welcher Grundlage wird dieses Buch weitergegeben?",
        Msg::RightsOwnWork => "Es ist mein eigenes Werk",
        Msg::RightsPublicDomain => "Es ist gemeinfrei",
        Msg::RightsOpenLicence => "Es steht unter einer freien Lizenz",
        Msg::RightsLicensedDistributor => "Ich bin zum Vertrieb berechtigt",
        Msg::RightsConfirm => {
            "Ich bestätige, dass ich diese Datei weitergeben und dieses Coverbild verwenden darf."
        }
        Msg::UploadPurchasedNotice => {
            "Ein im Shop gekauftes eBook ist eine persönliche Lizenz und kann hier nicht weitergegeben werden. Zum Verschenken bitte die Geschenkfunktion des Shops nutzen."
        }
        Msg::CoverNotFound => {
            "Kein Cover gefunden. Bitte ein Bild hochladen oder das typografische Cover verwenden."
        }
        Msg::IsbnInvalid => "Diese ISBN ist ungültig.",
        Msg::BookNotFound => "Kein Buch zu dieser ISBN gefunden.",
        Msg::Loading => "Wird geladen…",
        Msg::LanguageLabel => "Sprache",
        Msg::DemoNotice => "Demo – fiktiver Shop, keine Verbindung zu einem echten Händler.",
    }
}
