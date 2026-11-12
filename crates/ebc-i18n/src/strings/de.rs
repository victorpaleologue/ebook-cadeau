//! German (de-DE). Register: the pronoun-free infinitive of instructions, so that the
//! card chooses neither *du* nor *Sie*. No "ß", so the strings also work in Switzerland.

use crate::Msg;

pub(super) fn text(msg: Msg) -> &'static str {
    match msg {
        Msg::ProductName => "E-Book-Geschenk",
        Msg::ProductTagline => "Ein E-Book zum Verschenken – wie ein echtes Buch.",
        Msg::CheckoutOption => "Dieses E-Book verschenken",
        Msg::CheckoutOptionHelp => {
            "Wir gestalten eine Karte, die aussieht wie das Buch. Über den QR-Code kommt das E-Book auf E-Reader, Tablet oder Smartphone."
        }
        Msg::CheckoutPrintAtHome => "Karte zum Selbstausdrucken",
        Msg::CheckoutPrintedByPost => "Gedruckte Karte per Post",

        Msg::CardHeadline => "Dieses Buch ist ein Geschenk",
        Msg::CardFrom => "Von {name}",
        Msg::CardTo => "Für {name}",
        Msg::CardHowTo => "So geht’s",
        Msg::CardStep1 => "QR-Code mit dem Smartphone scannen.",
        Msg::CardStep2 => "Lesegerät wählen: E-Reader, Tablet oder Smartphone.",
        Msg::CardStep3 => "Buch abholen. Viel Freude beim Lesen!",
        Msg::CardStep2Reseller => "Bei {store} anmelden oder ein Konto erstellen.",
        Msg::CardStep3Reseller => {
            "Das Buch in der Bibliothek öffnen – auf dem E-Reader und in der App."
        }
        Msg::CardNoPhone => {
            "Kein Smartphone? Auf dem E-Reader oder am Computer {url} öffnen und den Code eingeben."
        }
        Msg::CodeLabel => "Code",
        Msg::CardAlreadyPaid => "Dieses E-Book ist bereits bezahlt – einfach lesen.",
        Msg::CardValidUntil => "Verfügbar bis zum {date}.",
        Msg::CardNoExpiry => "Ohne zeitliche Begrenzung.",
        Msg::CardHelp => "Hilfe: {url}",
        Msg::CardBackDefault => {
            "Dieses Buch ist digital. Karte öffnen und auf E-Reader, Tablet oder Smartphone lesen."
        }
        Msg::FoldHere => "Hier falten",
        Msg::CutHere => "Hier schneiden",
        Msg::PrintedWith => "Karte erstellt mit eBook Cadeau",
        Msg::EbookWord => "E-Book",
        Msg::EreaderWord => "E-Reader",

        Msg::RedeemTitle => "Buch abholen",
        Msg::RedeemIntro => "Bitte den Code von der Innenseite der Karte eingeben.",
        Msg::RedeemCta => "Mein Buch abholen",
        Msg::RedeemGiftFrom => "Dieses Buch ist ein Geschenk von {name}.",
        Msg::RedeemGiftAnonymous => "Dieses Buch ist ein Geschenk.",
        Msg::ChooseDevice => "Wo soll das Buch gelesen werden?",
        Msg::DeviceThis => "Auf diesem Gerät",
        Msg::DeviceEreader => "Auf einem E-Reader",
        Msg::DevicePhoneTablet => "Auf Smartphone oder Tablet",
        Msg::DeviceComputer => "Auf einem Computer",
        Msg::DownloadCta => "Buch herunterladen",
        Msg::ReadNowCta => "Jetzt lesen",
        Msg::SendToEreaderCta => "An meinen E-Reader senden",
        Msg::AddToLibraryCta => "Zu meiner Bibliothek hinzufügen",
        Msg::ContinueAtStore => "Weiter bei {store}",
        Msg::EreaderBrowserHint => {
            "Auf dem E-Reader den Browser öffnen, {url} aufrufen und den Code von der Karte eingeben."
        }
        Msg::EmailLabel => "E-Mail-Adresse",
        Msg::SendCta => "Senden",
        Msg::SentConfirmation => {
            "Gesendet. Das Buch erscheint in wenigen Minuten auf dem E-Reader."
        }
        Msg::AlreadyRedeemed => {
            "Dieses Buch wurde bereits abgeholt. Es kann erneut heruntergeladen werden."
        }
        Msg::EnjoyReading => "Viel Freude beim Lesen!",
        Msg::ErrorCodeTypo => {
            "Dieser Code enthält einen Tippfehler. Bitte prüfen und erneut eingeben."
        }
        Msg::ErrorCodeFormat => "Der Code besteht aus {count} Zeichen: Buchstaben und Ziffern.",
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

        Msg::CreateTitle => "E-Book-Geschenk erstellen",
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
            "Ein im Shop gekauftes E-Book ist eine persönliche Lizenz und kann hier nicht weitergegeben werden. Zum Verschenken bitte die Geschenkfunktion des Shops nutzen."
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
