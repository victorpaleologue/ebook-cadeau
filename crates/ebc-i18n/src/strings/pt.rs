//! Portuguese (pt-PT). The printed steps use the infinitive; screens use the formal third
//! person, the retail default in Portugal. European vocabulary ("telemóvel", "ficheiro",
//! "descarregar"), but "presente" rather than "prenda" for the gift: it reads the same in
//! Portugal and Brazil. Provisional: the least reliable strings of the catalogue.

use crate::Msg;

pub(super) fn text(msg: Msg) -> &'static str {
    match msg {
        Msg::ProductName => "eBook para Oferecer",
        Msg::ProductTagline => "Um eBook para oferecer, como um livro a sério.",
        Msg::CheckoutOption => "Oferecer este eBook",
        Msg::CheckoutOptionHelp => {
            "Um cartão em forma de livro, para imprimir em casa ou receber pelo correio. Quem o recebe aponta a câmara para o código e lê o livro no eReader, tablet ou telemóvel."
        }
        Msg::CheckoutPrintAtHome => "Cartão para imprimir em casa (grátis)",
        Msg::CheckoutPrintedByPost => "Cartão impresso e enviado pelo correio",

        Msg::CardHeadline => "Este livro é um presente",
        Msg::CardFrom => "De {name}",
        Msg::CardTo => "Para {name}",
        Msg::CardHowTo => "Como ler",
        Msg::CardStep1 => "Apontar a câmara de um telemóvel para este código.",
        Msg::CardStep2 => "Escolher onde ler: eReader, tablet ou telemóvel.",
        Msg::CardStep3 => "O livro chega. Boa leitura!",
        Msg::CardStep2Reseller => "Iniciar sessão na conta {store}, ou criar uma.",
        Msg::CardStep3Reseller => "O livro aparece na biblioteca, no eReader e na aplicação.",
        Msg::CardNoPhone => {
            "Sem telemóvel? No eReader ou num computador, abrir o navegador, ir a {url} e introduzir o código deste cartão."
        }
        Msg::CodeLabel => "Código",
        Msg::CardAlreadyPaid => {
            "Este eBook já está pago. Leia-o no seu eReader, tablet ou telemóvel."
        }
        Msg::CardValidUntil => "Receber até {date}.",
        Msg::CardNoExpiry => "Sem data limite.",
        Msg::CardHelp => "Precisa de ajuda? {url}",
        Msg::CardBackDefault => {
            "Este livro é digital. Abrir este cartão para o receber no eReader, tablet ou telemóvel."
        }
        Msg::CardLegalLine => "{title}, {author}. Um exemplar digital, já pago: nada a pagar.",
        Msg::CardLegalLineNoAuthor => "{title}. Um exemplar digital, já pago: nada a pagar.",
        Msg::CardSingleUse => "Utilização única.",
        Msg::FoldHere => "Dobrar aqui",
        Msg::CutHere => "Cortar aqui",
        Msg::PrintedWith => "Cartão criado com eBook Cadeau",
        Msg::EbookWord => "eBook",
        Msg::EreaderWord => "eReader",
        Msg::QrWord => "código QR",

        Msg::PrintDuplexShortEdge => {
            "Imprimir a 100\u{a0}% (tamanho real), frente e verso, virar pela margem curta."
        }
        Msg::PrintDuplexLongEdge => {
            "Imprimir a 100\u{a0}% (tamanho real), frente e verso, virar pela margem longa."
        }
        Msg::PrintCutFold => "Cortar pelas marcas contínuas, dobrar pelas tracejadas.",
        Msg::CoverCredit => "Capa: {credit}",
        Msg::CoverCreditPublisher => "Capa: © {publisher}",
        Msg::CoverCreditPublicDomain => "Capa: domínio público",
        Msg::CoverCreditUnknown => "Capa: todos os direitos reservados",
        Msg::CardFormatLine => "Formato {width} × {height} mm, lombada {spine} mm",
        Msg::OutsideLabel => "Exterior",

        Msg::RedeemTitle => "Receba o seu livro",
        Msg::RedeemIntro => "Introduza o código impresso no interior do cartão.",
        Msg::RedeemCta => "Receber o meu livro",
        Msg::RedeemGiftFrom => "{name} oferece-lhe este livro.",
        Msg::RedeemGiftAnonymous => "Este livro é um presente para si.",
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
        Msg::ErrorCodeFormat => "O código tem {count} letras, sem algarismos nem vogais.",
        Msg::ErrorCodeUnknown => "Este código não é reconhecido. Verifique-o no cartão.",
        Msg::ErrorGiftExpired => "Este presente já não está disponível: o prazo terminou.",
        Msg::ErrorGiftRevoked => "Este presente foi cancelado. Contacte quem lho ofereceu.",
        Msg::ErrorGiftPending => {
            "Este presente ainda não está pronto. Tente novamente dentro de alguns minutos."
        }
        Msg::ErrorTooManyAttempts => {
            "Demasiadas tentativas. Tente novamente dentro de alguns minutos."
        }
        Msg::ErrorGeneric => "Ocorreu um erro. Tente novamente mais tarde.",

        Msg::CreateTitle => "Criar um eBook para Oferecer",
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
