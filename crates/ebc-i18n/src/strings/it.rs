//! Italian (it-IT). Register: *tu*, the standard register of instructions and retail.

use crate::Msg;

pub(super) fn text(msg: Msg) -> &'static str {
    match msg {
        Msg::ProductName => "eBook regalo",
        Msg::ProductTagline => "Un eBook da regalare, come un libro vero.",
        Msg::CheckoutOption => "Regala questo eBook",
        Msg::CheckoutOptionHelp => {
            "Prepariamo un biglietto con l’aspetto del libro. Il suo codice QR permette di ricevere l’eBook su eReader, tablet o telefono."
        }
        Msg::CheckoutPrintAtHome => "Biglietto da stampare a casa",
        Msg::CheckoutPrintedByPost => "Biglietto stampato e spedito per posta",

        Msg::CardHeadline => "Questo libro è un regalo",
        Msg::CardFrom => "Da parte di {name}",
        Msg::CardTo => "Per {name}",
        Msg::CardHowTo => "Come leggerlo",
        Msg::CardStep1 => "Inquadra il codice QR con il telefono.",
        Msg::CardStep2 => "Scegli dove leggere: eReader, tablet o telefono.",
        Msg::CardStep3 => "Ricevi il tuo libro. Buona lettura!",
        Msg::CardStep2Reseller => "Accedi al tuo account {store} o creane uno.",
        Msg::CardStep3Reseller => "Trova il libro nella tua libreria, sull’eReader e nell’app.",
        Msg::CardNoPhone => {
            "Senza telefono? Dall’eReader o da un computer, apri {url} e inserisci il codice."
        }
        Msg::CodeLabel => "Codice",
        Msg::CardAlreadyPaid => "Questo eBook è già pagato: non resta che leggerlo.",
        Msg::CardValidUntil => "Disponibile fino al giorno {date}.",
        Msg::CardNoExpiry => "Senza scadenza.",
        Msg::CardHelp => "Serve aiuto? {url}",
        Msg::CardBackDefault => {
            "Questo libro è digitale. Apri questo biglietto per riceverlo su eReader, tablet o telefono."
        }
        Msg::FoldHere => "Piegare qui",
        Msg::CutHere => "Tagliare qui",
        Msg::PrintedWith => "Biglietto creato con eBook Cadeau",
        Msg::EbookWord => "eBook",
        Msg::EreaderWord => "eReader",

        Msg::RedeemTitle => "Ricevi il tuo libro",
        Msg::RedeemIntro => "Inserisci il codice stampato all’interno del biglietto.",
        Msg::RedeemCta => "Ricevi il libro",
        Msg::RedeemGiftFrom => "{name} ti regala questo libro.",
        Msg::RedeemGiftAnonymous => "Questo libro è un regalo per te.",
        Msg::ChooseDevice => "Dove vuoi leggere?",
        Msg::DeviceThis => "Su questo dispositivo",
        Msg::DeviceEreader => "Su un eReader",
        Msg::DevicePhoneTablet => "Su telefono o tablet",
        Msg::DeviceComputer => "Su un computer",
        Msg::DownloadCta => "Scarica il libro",
        Msg::ReadNowCta => "Leggi ora",
        Msg::SendToEreaderCta => "Invia al mio eReader",
        Msg::AddToLibraryCta => "Aggiungi alla mia libreria",
        Msg::ContinueAtStore => "Continua su {store}",
        Msg::EreaderBrowserHint => {
            "Sull’eReader, apri il browser, vai su {url} e inserisci il codice del biglietto."
        }
        Msg::EmailLabel => "Indirizzo e-mail",
        Msg::SendCta => "Invia",
        Msg::SentConfirmation => "Inviato. Il libro arriverà sull’eReader tra pochi minuti.",
        Msg::AlreadyRedeemed => "Questo libro è già stato ritirato. Puoi scaricarlo di nuovo.",
        Msg::EnjoyReading => "Buona lettura!",
        Msg::ErrorCodeTypo => {
            "Questo codice contiene un errore di battitura. Controllalo e riprova."
        }
        Msg::ErrorCodeFormat => "Il codice ha {count} caratteri: lettere e cifre.",
        Msg::ErrorCodeUnknown => "Questo codice non è riconosciuto. Controllalo sul biglietto.",
        Msg::ErrorGiftExpired => "Questo regalo non è più disponibile: il termine è scaduto.",
        Msg::ErrorGiftRevoked => "Questo regalo è stato annullato. Contatta chi te lo ha regalato.",
        Msg::ErrorGiftPending => "Questo regalo non è ancora pronto. Riprova tra qualche minuto.",
        Msg::ErrorTooManyAttempts => "Troppi tentativi. Riprova tra qualche minuto.",
        Msg::ErrorGeneric => "Si è verificato un errore. Riprova più tardi.",

        Msg::CreateTitle => "Crea un eBook regalo",
        Msg::FieldIsbn => "ISBN del libro",
        Msg::FieldTitle => "Titolo",
        Msg::FieldAuthor => "Autore",
        Msg::FieldGiver => "Da parte di",
        Msg::FieldRecipient => "Per",
        Msg::FieldMessage => "Messaggio personale",
        Msg::FieldCover => "Immagine di copertina",
        Msg::FieldFile => "File del libro (EPUB o PDF)",
        Msg::LookupCta => "Cerca",
        Msg::GenerateCta => "Crea il biglietto",
        Msg::DownloadPdfCta => "Scarica il PDF",
        Msg::PrintAtHomeCta => "Stampa a casa",
        Msg::OrderPrintCta => "Ordina la stampa",
        Msg::PreviewLabel => "Anteprima",
        Msg::SpineLabel => "Dorso",
        Msg::FrontLabel => "Copertina",
        Msg::BackLabel => "Quarta di copertina",
        Msg::InsideLabel => "Interno",
        Msg::RightsBasisLabel => "A che titolo distribuisci questo libro?",
        Msg::RightsOwnWork => "È un’opera mia",
        Msg::RightsPublicDomain => "È di pubblico dominio",
        Msg::RightsOpenLicence => "È distribuito con licenza libera",
        Msg::RightsLicensedDistributor => "Ne sono il distributore autorizzato",
        Msg::RightsConfirm => {
            "Confermo di avere il diritto di distribuire questo file e di usare questa immagine di copertina."
        }
        Msg::UploadPurchasedNotice => {
            "Un eBook acquistato in un negozio è una licenza personale: non può essere ceduto qui. Per regalarlo, usa l’opzione regalo del negozio."
        }
        Msg::CoverNotFound => {
            "Nessuna copertina trovata. Carica un’immagine o usa la copertina tipografica."
        }
        Msg::IsbnInvalid => "Questo ISBN non è valido.",
        Msg::BookNotFound => "Nessun libro trovato per questo ISBN.",
        Msg::Loading => "Caricamento…",
        Msg::LanguageLabel => "Lingua",
        Msg::DemoNotice => "Demo: negozio fittizio, non affiliato ad alcun rivenditore reale.",
    }
}
