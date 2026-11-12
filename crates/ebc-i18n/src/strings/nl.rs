//! Dutch (nl-NL). Register: *je*, the norm in Dutch retail. "e-book" and "e-reader" are
//! hyphenated and lower case.

use crate::Msg;

pub(super) fn text(msg: Msg) -> &'static str {
    match msg {
        Msg::ProductName => "e-book cadeau",
        Msg::ProductTagline => "Een e-book om cadeau te doen, net als een echt boek.",
        Msg::CheckoutOption => "Dit e-book cadeau doen",
        Msg::CheckoutOptionHelp => {
            "We maken een kaart die eruitziet als het boek. Met de QR-code komt het e-book op een e-reader, tablet of telefoon."
        }
        Msg::CheckoutPrintAtHome => "Kaart om zelf te printen",
        Msg::CheckoutPrintedByPost => "Gedrukte kaart per post",

        Msg::CardHeadline => "Dit boek is een cadeau",
        Msg::CardFrom => "Van {name}",
        Msg::CardTo => "Voor {name}",
        Msg::CardHowTo => "Zo lees je het",
        Msg::CardStep1 => "Scan de QR-code met je telefoon.",
        Msg::CardStep2 => "Kies waar je wilt lezen: e-reader, tablet of telefoon.",
        Msg::CardStep3 => "Haal je boek op. Veel leesplezier!",
        Msg::CardStep2Reseller => "Log in op je account bij {store} of maak er een aan.",
        Msg::CardStep3Reseller => "Open het boek in je bibliotheek, op je e-reader en in de app.",
        Msg::CardNoPhone => {
            "Geen telefoon? Open {url} op je e-reader of een computer en vul de code in."
        }
        Msg::CodeLabel => "Code",
        Msg::CardAlreadyPaid => "Dit e-book is al betaald: je hoeft het alleen nog te lezen.",
        Msg::CardValidUntil => "Beschikbaar tot en met {date}.",
        Msg::CardNoExpiry => "Zonder einddatum.",
        Msg::CardHelp => "Hulp nodig? {url}",
        Msg::CardBackDefault => {
            "Dit boek is digitaal. Open deze kaart en lees het op je e-reader, tablet of telefoon."
        }
        Msg::FoldHere => "Hier vouwen",
        Msg::CutHere => "Hier knippen",
        Msg::PrintedWith => "Kaart gemaakt met eBook Cadeau",
        Msg::EbookWord => "e-book",
        Msg::EreaderWord => "e-reader",

        Msg::RedeemTitle => "Haal je boek op",
        Msg::RedeemIntro => "Vul de code in die aan de binnenkant van je kaart staat.",
        Msg::RedeemCta => "Mijn boek ophalen",
        Msg::RedeemGiftFrom => "{name} geeft je dit boek cadeau.",
        Msg::RedeemGiftAnonymous => "Dit boek is een cadeau voor jou.",
        Msg::ChooseDevice => "Waar wil je lezen?",
        Msg::DeviceThis => "Op dit apparaat",
        Msg::DeviceEreader => "Op een e-reader",
        Msg::DevicePhoneTablet => "Op een telefoon of tablet",
        Msg::DeviceComputer => "Op een computer",
        Msg::DownloadCta => "Boek downloaden",
        Msg::ReadNowCta => "Nu lezen",
        Msg::SendToEreaderCta => "Naar mijn e-reader sturen",
        Msg::AddToLibraryCta => "Toevoegen aan mijn bibliotheek",
        Msg::ContinueAtStore => "Verder bij {store}",
        Msg::EreaderBrowserHint => {
            "Open op je e-reader de browser, ga naar {url} en vul de code van de kaart in."
        }
        Msg::EmailLabel => "E-mailadres",
        Msg::SendCta => "Versturen",
        Msg::SentConfirmation => "Verstuurd. Het boek staat binnen enkele minuten op de e-reader.",
        Msg::AlreadyRedeemed => "Dit boek is al opgehaald. Je kunt het opnieuw downloaden.",
        Msg::EnjoyReading => "Veel leesplezier!",
        Msg::ErrorCodeTypo => {
            "Deze code bevat een typefout. Controleer de code en probeer het opnieuw."
        }
        Msg::ErrorCodeFormat => "De code bestaat uit {count} tekens: letters en cijfers.",
        Msg::ErrorCodeUnknown => "Deze code is onbekend. Controleer de code op je kaart.",
        Msg::ErrorGiftExpired => "Dit cadeau is niet meer beschikbaar: de termijn is verstreken.",
        Msg::ErrorGiftRevoked => {
            "Dit cadeau is geannuleerd. Neem contact op met degene van wie je het hebt gekregen."
        }
        Msg::ErrorGiftPending => {
            "Dit cadeau is nog niet klaar. Probeer het over een paar minuten opnieuw."
        }
        Msg::ErrorTooManyAttempts => "Te veel pogingen. Probeer het over een paar minuten opnieuw.",
        Msg::ErrorGeneric => "Er is iets misgegaan. Probeer het later opnieuw.",

        Msg::CreateTitle => "Een e-book cadeau maken",
        Msg::FieldIsbn => "ISBN van het boek",
        Msg::FieldTitle => "Titel",
        Msg::FieldAuthor => "Auteur",
        Msg::FieldGiver => "Van",
        Msg::FieldRecipient => "Voor",
        Msg::FieldMessage => "Persoonlijk bericht",
        Msg::FieldCover => "Omslagafbeelding",
        Msg::FieldFile => "Boekbestand (EPUB of PDF)",
        Msg::LookupCta => "Zoeken",
        Msg::GenerateCta => "Kaart maken",
        Msg::DownloadPdfCta => "PDF downloaden",
        Msg::PrintAtHomeCta => "Zelf printen",
        Msg::OrderPrintCta => "Drukwerk bestellen",
        Msg::PreviewLabel => "Voorbeeld",
        Msg::SpineLabel => "Rug",
        Msg::FrontLabel => "Voorkant",
        Msg::BackLabel => "Achterkant",
        Msg::InsideLabel => "Binnenkant",
        Msg::RightsBasisLabel => "Op welke grond verspreid je dit boek?",
        Msg::RightsOwnWork => "Het is mijn eigen werk",
        Msg::RightsPublicDomain => "Het behoort tot het publieke domein",
        Msg::RightsOpenLicence => "Het heeft een vrije licentie",
        Msg::RightsLicensedDistributor => "Ik ben de erkende distributeur",
        Msg::RightsConfirm => {
            "Ik bevestig dat ik dit bestand mag verspreiden en deze omslagafbeelding mag gebruiken."
        }
        Msg::UploadPurchasedNotice => {
            "Een e-book dat in een winkel is gekocht, is een persoonlijke licentie en kan hier niet worden doorgegeven. Gebruik de cadeau-optie van de winkel om het cadeau te doen."
        }
        Msg::CoverNotFound => {
            "Geen omslag gevonden. Upload een afbeelding of gebruik de typografische omslag."
        }
        Msg::IsbnInvalid => "Dit ISBN is ongeldig.",
        Msg::BookNotFound => "Geen boek gevonden voor dit ISBN.",
        Msg::Loading => "Laden…",
        Msg::LanguageLabel => "Taal",
        Msg::DemoNotice => "Demo – fictieve winkel, niet verbonden aan een echte winkelketen.",
    }
}
