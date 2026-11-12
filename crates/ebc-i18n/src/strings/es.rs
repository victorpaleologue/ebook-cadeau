//! Spanish (es-ES). Register: *tú*, the neutral register of retail in Spain.

use crate::Msg;

pub(super) fn text(msg: Msg) -> &'static str {
    match msg {
        Msg::ProductName => "eBook regalo",
        Msg::ProductTagline => "Un eBook para regalar, como un libro de verdad.",
        Msg::CheckoutOption => "Regalar este eBook",
        Msg::CheckoutOptionHelp => {
            "Preparamos una tarjeta con el aspecto del libro. Su código QR permite recibir el eBook en un eReader, una tableta o un móvil."
        }
        Msg::CheckoutPrintAtHome => "Tarjeta para imprimir en casa",
        Msg::CheckoutPrintedByPost => "Tarjeta impresa y enviada por correo",

        Msg::CardHeadline => "Este libro es un regalo",
        Msg::CardFrom => "De parte de {name}",
        Msg::CardTo => "Para {name}",
        Msg::CardHowTo => "Cómo leerlo",
        Msg::CardStep1 => "Escanea el código QR con tu móvil.",
        Msg::CardStep2 => "Elige dónde leer: eReader, tableta o móvil.",
        Msg::CardStep3 => "Recoge tu libro. ¡Feliz lectura!",
        Msg::CardStep2Reseller => "Inicia sesión en tu cuenta de {store} o crea una.",
        Msg::CardStep3Reseller => {
            "Encuentra el libro en tu biblioteca, en el eReader y en la aplicación."
        }
        Msg::CardNoPhone => {
            "¿Sin móvil? En tu eReader o en un ordenador, abre {url} e introduce el código."
        }
        Msg::CodeLabel => "Código",
        Msg::CardAlreadyPaid => "Este eBook ya está pagado: solo queda leerlo.",
        Msg::CardValidUntil => "Disponible hasta el {date}.",
        Msg::CardNoExpiry => "Sin fecha límite.",
        Msg::CardHelp => "¿Necesitas ayuda? {url}",
        Msg::CardBackDefault => {
            "Este libro es digital. Abre esta tarjeta para recibirlo en tu eReader, tableta o móvil."
        }
        Msg::FoldHere => "Doblar aquí",
        Msg::CutHere => "Cortar aquí",
        Msg::PrintedWith => "Tarjeta creada con eBook Cadeau",
        Msg::EbookWord => "eBook",
        Msg::EreaderWord => "eReader",

        Msg::RedeemTitle => "Recibe tu libro",
        Msg::RedeemIntro => "Introduce el código impreso en el interior de tu tarjeta.",
        Msg::RedeemCta => "Conseguir mi libro",
        Msg::RedeemGiftFrom => "{name} te regala este libro.",
        Msg::RedeemGiftAnonymous => "Este libro es un regalo para ti.",
        Msg::ChooseDevice => "¿Dónde quieres leer?",
        Msg::DeviceThis => "En este dispositivo",
        Msg::DeviceEreader => "En un eReader",
        Msg::DevicePhoneTablet => "En un móvil o una tableta",
        Msg::DeviceComputer => "En un ordenador",
        Msg::DownloadCta => "Descargar el libro",
        Msg::ReadNowCta => "Leer ahora",
        Msg::SendToEreaderCta => "Enviar a mi eReader",
        Msg::AddToLibraryCta => "Añadir a mi biblioteca",
        Msg::ContinueAtStore => "Continuar en {store}",
        Msg::EreaderBrowserHint => {
            "En tu eReader, abre el navegador, entra en {url} e introduce el código de la tarjeta."
        }
        Msg::EmailLabel => "Correo electrónico",
        Msg::SendCta => "Enviar",
        Msg::SentConfirmation => "Enviado. El libro llegará al eReader en unos minutos.",
        Msg::AlreadyRedeemed => "Este libro ya se ha recogido. Puedes volver a descargarlo.",
        Msg::EnjoyReading => "¡Feliz lectura!",
        Msg::ErrorCodeTypo => {
            "Este código contiene un error de escritura. Compruébalo e inténtalo de nuevo."
        }
        Msg::ErrorCodeFormat => "El código tiene {count} caracteres: letras y cifras.",
        Msg::ErrorCodeUnknown => "No reconocemos este código. Compruébalo en tu tarjeta.",
        Msg::ErrorGiftExpired => "Este regalo ya no está disponible: ha vencido su fecha límite.",
        Msg::ErrorGiftRevoked => {
            "Este regalo se ha cancelado. Ponte en contacto con quien te lo regaló."
        }
        Msg::ErrorGiftPending => {
            "Este regalo aún no está listo. Inténtalo de nuevo en unos minutos."
        }
        Msg::ErrorTooManyAttempts => "Demasiados intentos. Inténtalo de nuevo en unos minutos.",
        Msg::ErrorGeneric => "Se ha producido un error. Inténtalo de nuevo más tarde.",

        Msg::CreateTitle => "Crear un eBook regalo",
        Msg::FieldIsbn => "ISBN del libro",
        Msg::FieldTitle => "Título",
        Msg::FieldAuthor => "Autor",
        Msg::FieldGiver => "De parte de",
        Msg::FieldRecipient => "Para",
        Msg::FieldMessage => "Mensaje personal",
        Msg::FieldCover => "Imagen de cubierta",
        Msg::FieldFile => "Archivo del libro (EPUB o PDF)",
        Msg::LookupCta => "Buscar",
        Msg::GenerateCta => "Crear la tarjeta",
        Msg::DownloadPdfCta => "Descargar el PDF",
        Msg::PrintAtHomeCta => "Imprimir en casa",
        Msg::OrderPrintCta => "Encargar la impresión",
        Msg::PreviewLabel => "Vista previa",
        Msg::SpineLabel => "Lomo",
        Msg::FrontLabel => "Cubierta",
        Msg::BackLabel => "Contracubierta",
        Msg::InsideLabel => "Interior",
        Msg::RightsBasisLabel => "¿En calidad de qué distribuyes este libro?",
        Msg::RightsOwnWork => "Es una obra mía",
        Msg::RightsPublicDomain => "Es de dominio público",
        Msg::RightsOpenLicence => "Tiene una licencia libre",
        Msg::RightsLicensedDistributor => "Soy su distribuidor autorizado",
        Msg::RightsConfirm => {
            "Confirmo que tengo derecho a distribuir este archivo y a utilizar esta imagen de cubierta."
        }
        Msg::UploadPurchasedNotice => {
            "Un eBook comprado en una tienda es una licencia personal: no se puede transferir aquí. Para regalarlo, usa la opción de regalo de la tienda."
        }
        Msg::CoverNotFound => {
            "No se ha encontrado ninguna cubierta. Sube una imagen o usa la cubierta tipográfica."
        }
        Msg::IsbnInvalid => "Este ISBN no es válido.",
        Msg::BookNotFound => "No se ha encontrado ningún libro con este ISBN.",
        Msg::Loading => "Cargando…",
        Msg::LanguageLabel => "Idioma",
        Msg::DemoNotice => "Demostración: tienda ficticia, sin relación con ningún comercio real.",
    }
}
