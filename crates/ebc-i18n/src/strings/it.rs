//! Italian (it-IT). The printed steps use the infinitive; where a pronoun is unavoidable
//! (screens, the "already paid" line) the register is *tu*, the standard register of
//! retail.

use crate::Msg;

pub(super) fn text(msg: Msg) -> &'static str {
    match msg {
        Msg::ProductName => "eBook Regalo",
        Msg::ProductTagline => "Un eBook da regalare, come un libro vero.",
        Msg::CheckoutOption => "Regala questo eBook",
        Msg::CheckoutOptionHelp => {
            "Un biglietto a forma di libro, da stampare a casa o da ricevere per posta. Chi lo riceve inquadra il codice e legge il libro su eReader, tablet o smartphone."
        }
        Msg::CheckoutPrintAtHome => "Biglietto da stampare a casa (gratis)",
        Msg::CheckoutPrintedByPost => "Biglietto stampato e spedito",

        Msg::CardHeadline => "Questo libro è un regalo",
        Msg::CardFrom => "Da parte di {name}",
        Msg::CardTo => "Per {name}",
        Msg::CardHowTo => "Come leggerlo",
        Msg::CardStep1 => "Inquadrare questo codice con la fotocamera di un telefono.",
        Msg::CardStep2 => "Scegliere dove leggere: eReader, tablet o smartphone.",
        Msg::CardStep3 => "Il libro arriva. Buona lettura!",
        Msg::CardStep2Reseller => "Accedere all’account {store}, o crearne uno.",
        Msg::CardStep3Reseller => "Il libro compare nella libreria, sull’eReader e nell’app.",
        Msg::CardNoPhone => {
            "Senza telefono? Dall’eReader o da un computer, aprire il browser, andare su {url} e inserire il codice di questo biglietto."
        }
        Msg::CodeLabel => "Codice",
        Msg::CardAlreadyPaid => {
            "Questo eBook è già pagato. Leggilo su eReader, tablet o smartphone."
        }
        Msg::CardValidUntil => "Da ritirare entro il giorno {date}.",
        Msg::CardNoExpiry => "Senza scadenza.",
        Msg::CardHelp => "Serve aiuto? {url}",
        Msg::CardBackDefault => {
            "Questo libro è digitale. Aprire questo biglietto per riceverlo su eReader, tablet o smartphone."
        }
        Msg::CardLegalLine => {
            "{title}, {author}. Una copia digitale, già pagata: niente da pagare."
        }
        Msg::CardLegalLineNoAuthor => "{title}. Una copia digitale, già pagata: niente da pagare.",
        Msg::CardSingleUse => "Utilizzabile una sola volta.",
        Msg::FoldHere => "Piegare qui",
        Msg::CutHere => "Tagliare qui",
        Msg::PrintedWith => "Biglietto creato con eBook Cadeau",
        Msg::EbookWord => "eBook",
        Msg::EreaderWord => "eReader",
        Msg::QrWord => "codice QR",

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
        Msg::ErrorCodeFormat => "Il codice ha {count} lettere, senza cifre né vocali.",
        Msg::ErrorCodeUnknown => "Questo codice non è riconosciuto. Controllalo sul biglietto.",
        Msg::ErrorGiftExpired => "Questo regalo non è più disponibile: il termine è scaduto.",
        Msg::ErrorGiftRevoked => "Questo regalo è stato annullato. Contatta chi te lo ha regalato.",
        Msg::ErrorGiftPending => "Questo regalo non è ancora pronto. Riprova tra qualche minuto.",
        Msg::ErrorTooManyAttempts => "Troppi tentativi. Riprova tra qualche minuto.",
        Msg::ErrorGeneric => "Si è verificato un errore. Riprova più tardi.",

        Msg::CreateTitle => "Crea un eBook Regalo",
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
