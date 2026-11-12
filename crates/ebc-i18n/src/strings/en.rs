//! English (en-GB). "eBook" for the book, hyphenated "e-reader" for the device.

use crate::Msg;

pub(super) fn text(msg: Msg) -> &'static str {
    match msg {
        Msg::ProductName => "eBook Gift",
        Msg::ProductTagline => "An eBook to give, like a real book.",
        Msg::CheckoutOption => "Give as a gift",
        Msg::CheckoutOptionHelp => {
            "A book-shaped card to print at home or have sent by post. The person who receives it scans the code and reads the book on an e-reader, tablet or phone."
        }
        Msg::CheckoutPrintAtHome => "Card to print at home (free)",
        Msg::CheckoutPrintedByPost => "Printed card, sent by post",

        Msg::CardHeadline => "This book is a gift",
        Msg::CardFrom => "From {name}",
        Msg::CardTo => "For {name}",
        Msg::CardHowTo => "How to read it",
        Msg::CardStep1 => "Scan this code with a phone camera.",
        Msg::CardStep2 => "Choose where to read: e-reader, tablet or phone.",
        Msg::CardStep3 => "The book arrives. Happy reading!",
        Msg::CardStep2Reseller => "Sign in to a {store} account, or create one.",
        Msg::CardStep3Reseller => {
            "The book appears in the library, on the e-reader and in the app."
        }
        Msg::CardNoPhone => {
            "No phone? On an e-reader or a computer, open the browser, go to {url} and enter the code on this card."
        }
        Msg::CodeLabel => "Code",
        Msg::CardAlreadyPaid => {
            "This eBook is already paid for. Read it on your e-reader, tablet or phone."
        }
        Msg::CardValidUntil => "To be collected before {date}.",
        Msg::CardNoExpiry => "No time limit.",
        Msg::CardHelp => "Need help? {url}",
        Msg::CardBackDefault => {
            "This book is digital. Open this card to receive it on an e-reader, tablet or phone."
        }
        Msg::CardLegalLine => {
            "{title}, {author}. One digital copy, already paid for: nothing to pay."
        }
        Msg::CardLegalLineNoAuthor => {
            "{title}. One digital copy, already paid for: nothing to pay."
        }
        Msg::CardSingleUse => "Single use.",
        Msg::FoldHere => "Fold here",
        Msg::CutHere => "Cut here",
        Msg::PrintedWith => "Card made with eBook Cadeau",
        Msg::EbookWord => "eBook",
        Msg::EreaderWord => "e-reader",
        Msg::QrWord => "QR code",

        Msg::PrintDuplexShortEdge => {
            "Print at 100% (actual size), on both sides, flip on the short edge."
        }
        Msg::PrintDuplexLongEdge => {
            "Print at 100% (actual size), on both sides, flip on the long edge."
        }
        Msg::PrintCutFold => "Cut along the solid marks, fold along the dashed ones.",
        Msg::CoverCredit => "Cover: {credit}",
        Msg::CoverCreditPublisher => "Cover: © {publisher}",
        Msg::CoverCreditPublicDomain => "Cover: public domain",
        Msg::CoverCreditUnknown => "Cover: all rights reserved",
        Msg::CardFormatLine => "Format {width} × {height} mm, spine {spine} mm",
        Msg::OutsideLabel => "Outside",

        Msg::RedeemTitle => "Get your book",
        Msg::RedeemIntro => "Enter the code printed inside your card.",
        Msg::RedeemCta => "Get my book",
        Msg::RedeemGiftFrom => "{name} is giving you this book.",
        Msg::RedeemGiftAnonymous => "This book is a gift for you.",
        Msg::ChooseDevice => "Where would you like to read?",
        Msg::DeviceThis => "On this device",
        Msg::DeviceEreader => "On an e-reader",
        Msg::DevicePhoneTablet => "On a phone or tablet",
        Msg::DeviceComputer => "On a computer",
        Msg::DownloadCta => "Download the book",
        Msg::ReadNowCta => "Read now",
        Msg::SendToEreaderCta => "Send to my e-reader",
        Msg::AddToLibraryCta => "Add to my library",
        Msg::ContinueAtStore => "Continue at {store}",
        Msg::EreaderBrowserHint => {
            "On your e-reader, open the web browser, go to {url} and enter the code from the card."
        }
        Msg::EmailLabel => "Email address",
        Msg::SendCta => "Send",
        Msg::SentConfirmation => "Sent. The book will reach the e-reader within a few minutes.",
        Msg::AlreadyRedeemed => "This book has already been collected. You can download it again.",
        Msg::EnjoyReading => "Happy reading!",
        Msg::ErrorCodeTypo => "This code contains a typing mistake. Check it and try again.",
        Msg::ErrorCodeFormat => "The code has {count} letters, with no digits or vowels.",
        Msg::ErrorCodeUnknown => "This code is not recognised. Check it against your card.",
        Msg::ErrorGiftExpired => "This gift is no longer available: its time limit has passed.",
        Msg::ErrorGiftRevoked => {
            "This gift has been cancelled. Please contact the person who gave it to you."
        }
        Msg::ErrorGiftPending => "This gift is not ready yet. Try again in a few minutes.",
        Msg::ErrorTooManyAttempts => "Too many attempts. Try again in a few minutes.",
        Msg::ErrorGeneric => "Something went wrong. Please try again later.",

        Msg::CreateTitle => "Create an eBook Gift",
        Msg::FieldIsbn => "Book ISBN",
        Msg::FieldTitle => "Title",
        Msg::FieldAuthor => "Author",
        Msg::FieldGiver => "From",
        Msg::FieldRecipient => "For",
        Msg::FieldMessage => "Personal message",
        Msg::FieldCover => "Cover image",
        Msg::FieldFile => "Book file (EPUB or PDF)",
        Msg::LookupCta => "Look up",
        Msg::GenerateCta => "Create the card",
        Msg::DownloadPdfCta => "Download the PDF",
        Msg::PrintAtHomeCta => "Print at home",
        Msg::OrderPrintCta => "Order a print",
        Msg::PreviewLabel => "Preview",
        Msg::SpineLabel => "Spine",
        Msg::FrontLabel => "Front cover",
        Msg::BackLabel => "Back cover",
        Msg::InsideLabel => "Inside",
        Msg::RightsBasisLabel => "On what basis do you distribute this book?",
        Msg::RightsOwnWork => "It is my own work",
        Msg::RightsPublicDomain => "It is in the public domain",
        Msg::RightsOpenLicence => "It is under an open licence",
        Msg::RightsLicensedDistributor => "I am its authorised distributor",
        Msg::RightsConfirm => {
            "I confirm that I have the right to distribute this file and to use this cover image."
        }
        Msg::UploadPurchasedNotice => {
            "An eBook bought from a store is a personal licence: it cannot be passed on here. To give it, use the store’s own gift option."
        }
        Msg::CoverNotFound => "No cover found. Upload an image or use the typographic cover.",
        Msg::IsbnInvalid => "This ISBN is not valid.",
        Msg::BookNotFound => "No book found for this ISBN.",
        Msg::Loading => "Loading…",
        Msg::LanguageLabel => "Language",
        Msg::DemoNotice => "Demo — fictional store, not affiliated with any real retailer.",
    }
}
