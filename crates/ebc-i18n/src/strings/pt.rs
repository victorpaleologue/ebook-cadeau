//! Portuguese (pt-PT). Register: the formal third person, the retail default in
//! Portugal. European vocabulary ("telemóvel", "ficheiro", "descarregar", "prenda").

use crate::Msg;

pub(super) fn text(msg: Msg) -> &'static str {
    match msg {
        Msg::ProductName => "eBook presente",
        Msg::ProductTagline => "Um eBook para oferecer, como um livro a sério.",
        Msg::CheckoutOption => "Oferecer este eBook",
        Msg::CheckoutOptionHelp => {
            "Preparamos um cartão com o aspeto do livro. O código QR permite receber o eBook num eReader, tablet ou telemóvel."
        }
        Msg::CheckoutPrintAtHome => "Cartão para imprimir em casa",
        Msg::CheckoutPrintedByPost => "Cartão impresso e enviado por correio",

        Msg::CardHeadline => "Este livro é uma prenda",
        Msg::CardFrom => "De {name}",
        Msg::CardTo => "Para {name}",
        Msg::CardHowTo => "Como ler",
        Msg::CardStep1 => "Aponte a câmara do telemóvel para o código QR.",
        Msg::CardStep2 => "Escolha onde ler: eReader, tablet ou telemóvel.",
        Msg::CardStep3 => "Receba o seu livro. Boa leitura!",
        Msg::CardStep2Reseller => "Inicie sessão na sua conta {store} ou crie uma.",
        Msg::CardStep3Reseller => "Encontre o livro na sua biblioteca, no eReader e na aplicação.",
        Msg::CardNoPhone => {
            "Sem telemóvel? No eReader ou num computador, abra {url} e introduza o código."
        }
        Msg::CodeLabel => "Código",
        Msg::CardAlreadyPaid => "Este eBook já está pago: só falta lê-lo.",
        Msg::CardValidUntil => "Disponível até ao dia {date}.",
        Msg::CardNoExpiry => "Sem data limite.",
        Msg::CardHelp => "Precisa de ajuda? {url}",
        Msg::CardBackDefault => {
            "Este livro é digital. Abra este cartão para o receber no seu eReader, tablet ou telemóvel."
        }
        Msg::FoldHere => "Dobrar aqui",
        Msg::CutHere => "Cortar aqui",
        Msg::PrintedWith => "Cartão criado com eBook Cadeau",
        Msg::EbookWord => "eBook",
        Msg::EreaderWord => "eReader",

        Msg::RedeemTitle => "Receba o seu livro",
        Msg::RedeemIntro => "Introduza o código impresso no interior do cartão.",
        Msg::RedeemCta => "Receber o meu livro",
        Msg::RedeemGiftFrom => "{name} oferece-lhe este livro.",
        Msg::RedeemGiftAnonymous => "Este livro é uma prenda para si.",
        Msg::ChooseDevice => "Onde quer ler?",
        Msg::DeviceThis => "Neste dispositivo",
        Msg::DeviceEreader => "Num eReader",
        Msg::DevicePhoneTablet => "Num telemóvel ou tablet",
        Msg::DeviceComputer => "Num computador",
        Msg::DownloadCta => "Descarregar o livro",
        Msg::ReadNowCta => "Ler agora",
        Msg::SendToEreaderCta => "Enviar para o meu eReader",
        Msg::AddToLibraryCta => "Adicionar à minha biblioteca",
        Msg::ContinueAtStore => "Continuar em {store}",
        Msg::EreaderBrowserHint => {
            "No eReader, abra o navegador, vá a {url} e introduza o código do cartão."
        }
        Msg::EmailLabel => "Endereço de e-mail",
        Msg::SendCta => "Enviar",
        Msg::SentConfirmation => "Enviado. O livro chegará ao eReader dentro de alguns minutos.",
        Msg::AlreadyRedeemed => "Este livro já foi recebido. Pode descarregá-lo novamente.",
        Msg::EnjoyReading => "Boa leitura!",
        Msg::ErrorCodeTypo => {
            "Este código contém um erro de digitação. Verifique-o e tente novamente."
        }
        Msg::ErrorCodeFormat => "O código tem {count} caracteres: letras e algarismos.",
        Msg::ErrorCodeUnknown => "Este código não é reconhecido. Verifique-o no cartão.",
        Msg::ErrorGiftExpired => "Esta prenda já não está disponível: o prazo terminou.",
        Msg::ErrorGiftRevoked => "Esta prenda foi cancelada. Contacte quem lha ofereceu.",
        Msg::ErrorGiftPending => {
            "Esta prenda ainda não está pronta. Tente novamente dentro de alguns minutos."
        }
        Msg::ErrorTooManyAttempts => {
            "Demasiadas tentativas. Tente novamente dentro de alguns minutos."
        }
        Msg::ErrorGeneric => "Ocorreu um erro. Tente novamente mais tarde.",

        Msg::CreateTitle => "Criar um eBook presente",
        Msg::FieldIsbn => "ISBN do livro",
        Msg::FieldTitle => "Título",
        Msg::FieldAuthor => "Autor",
        Msg::FieldGiver => "De",
        Msg::FieldRecipient => "Para",
        Msg::FieldMessage => "Mensagem pessoal",
        Msg::FieldCover => "Imagem de capa",
        Msg::FieldFile => "Ficheiro do livro (EPUB ou PDF)",
        Msg::LookupCta => "Procurar",
        Msg::GenerateCta => "Criar o cartão",
        Msg::DownloadPdfCta => "Descarregar o PDF",
        Msg::PrintAtHomeCta => "Imprimir em casa",
        Msg::OrderPrintCta => "Encomendar a impressão",
        Msg::PreviewLabel => "Pré-visualização",
        Msg::SpineLabel => "Lombada",
        Msg::FrontLabel => "Capa",
        Msg::BackLabel => "Contracapa",
        Msg::InsideLabel => "Interior",
        Msg::RightsBasisLabel => "A que título distribui este livro?",
        Msg::RightsOwnWork => "É uma obra minha",
        Msg::RightsPublicDomain => "Está no domínio público",
        Msg::RightsOpenLicence => "Tem uma licença livre",
        Msg::RightsLicensedDistributor => "Sou o seu distribuidor autorizado",
        Msg::RightsConfirm => {
            "Confirmo que tenho o direito de distribuir este ficheiro e de utilizar esta imagem de capa."
        }
        Msg::UploadPurchasedNotice => {
            "Um eBook comprado numa loja é uma licença pessoal: não pode ser transmitido aqui. Para o oferecer, utilize a opção de oferta da loja."
        }
        Msg::CoverNotFound => {
            "Nenhuma capa encontrada. Carregue uma imagem ou utilize a capa tipográfica."
        }
        Msg::IsbnInvalid => "Este ISBN não é válido.",
        Msg::BookNotFound => "Nenhum livro encontrado para este ISBN.",
        Msg::Loading => "A carregar…",
        Msg::LanguageLabel => "Idioma",
        Msg::DemoNotice => "Demonstração — loja fictícia, sem ligação a qualquer retalhista real.",
    }
}
