//! Strings that only the server's pages use, in the seven supported locales.
//!
//! Wording shared with the card and the web app comes from `ebc-i18n` ([`ebc_i18n::Msg`]);
//! this table holds the rest, so that `ebc-i18n` did not have to change while it was
//! being written. Same rules as there: the gift is *a book*, never a balance or a
//! voucher; what people type is a *code*; French says *vous*. The strings were written by
//! one author and have **not** been reviewed by native speakers.
//!
//! Placeholders (`{name}`) are filled by [`sf`]; values are escaped by the templates, not
//! here.

use ebc_model::Locale;

macro_rules! strings {
    ($( $(#[$doc:meta])* $name:ident => [$fr:expr, $en:expr, $de:expr, $es:expr, $it:expr, $nl:expr, $pt:expr $(,)?]; )+) => {
        /// A server-only message.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub(crate) enum S {
            $( $(#[$doc])* $name, )+
        }

        impl S {
            /// Every message (for the completeness test).
            #[cfg(test)]
            pub(crate) const ALL: &'static [S] = &[ $( S::$name, )+ ];

            /// The translations in the order fr, en, de, es, it, nl, pt.
            const fn texts(self) -> [&'static str; 7] {
                match self {
                    $( S::$name => [$fr, $en, $de, $es, $it, $nl, $pt], )+
                }
            }
        }
    };
}

strings! {
    /// Under the code field.
    CodeHint => [
        "12 lettres, par exemple BCDF-GHJK-LMNL. Majuscules, minuscules et tirets sont indifférents.",
        "12 letters, for example BCDF-GHJK-LMNL. Capitals, small letters and hyphens make no difference.",
        "12 Buchstaben, zum Beispiel BCDF-GHJK-LMNL. Groß- und Kleinschreibung und Bindestriche spielen keine Rolle.",
        "12 letras, por ejemplo BCDF-GHJK-LMNL. Da igual escribir mayúsculas, minúsculas o guiones.",
        "12 lettere, per esempio BCDF-GHJK-LMNL. Maiuscole, minuscole e trattini non fanno differenza.",
        "12 letters, bijvoorbeeld BCDF-GHJK-LMNL. Hoofdletters, kleine letters en streepjes maken niet uit.",
        "12 letras, por exemplo BCDF-GHJK-LMNL. Maiúsculas, minúsculas e hífenes são indiferentes.",
    ];
    /// Footer link to the help page.
    Help => ["Aide", "Help", "Hilfe", "Ayuda", "Aiuto", "Hulp", "Ajuda"];
    /// Footer link to the home page.
    EnterCode => [
        "Saisir un code", "Enter a code", "Code eingeben", "Introducir un código",
        "Inserire un codice", "Een code invoeren", "Introduzir um código",
    ];
    /// Title of the help page.
    HelpTitle => [
        "Comment recevoir votre livre", "How to get your book", "So erhalten Sie Ihr Buch",
        "Cómo recibir tu libro", "Come ricevere il tuo libro", "Zo ontvang je je boek",
        "Como receber o seu livro",
    ];
    /// Help: heading of the phone route.
    HelpPhoneHead => [
        "Avec un téléphone ou une tablette", "With a phone or tablet", "Mit Smartphone oder Tablet",
        "Con un teléfono o una tableta", "Con un telefono o un tablet", "Met een telefoon of tablet",
        "Com um telemóvel ou tablet",
    ];
    /// Help: the phone route.
    HelpPhoneText => [
        "Scannez le QR code imprimé dans la carte avec l’appareil photo, puis choisissez où lire.",
        "Scan the QR code printed inside the card with the camera, then choose where to read.",
        "QR-Code in der Karte mit der Kamera scannen, dann auswählen, wo gelesen wird.",
        "Escanea con la cámara el código QR impreso en la tarjeta y elige dónde leer.",
        "Inquadra con la fotocamera il codice QR stampato nel biglietto, poi scegli dove leggere.",
        "Scan de QR-code in de kaart met de camera en kies waar je wilt lezen.",
        "Leia com a câmara o código QR impresso no cartão e escolha onde ler.",
    ];
    /// Heading of the Kindle route.
    OnKindle => [
        "Sur un Kindle", "On a Kindle", "Auf einem Kindle", "En un Kindle", "Su un Kindle",
        "Op een Kindle", "Num Kindle",
    ];
    /// Why a Kindle needs another route, and which.
    KindleExplain => [
        "Un Kindle n’ouvre pas directement ce format de fichier. Envoyez-lui le livre avec «\u{a0}Send to Kindle\u{a0}»\u{a0}: depuis un téléphone, téléchargez le livre puis partagez-le vers l’application Kindle\u{a0}; ou utilisez l’envoi par e-mail.",
        "A Kindle does not open this file format directly. Send it the book with “Send to Kindle”: on a phone, download the book and share it to the Kindle app; or use the e-mail form.",
        "Ein Kindle öffnet dieses Dateiformat nicht direkt. Das Buch mit „Send to Kindle“ übertragen: auf dem Smartphone herunterladen und mit der Kindle-App teilen oder per E-Mail senden.",
        "Un Kindle no abre directamente este formato de archivo. Envíale el libro con «Send to Kindle»: en un teléfono, descarga el libro y compártelo con la aplicación Kindle; o usa el envío por correo electrónico.",
        "Un Kindle non apre direttamente questo formato di file. Inviagli il libro con «Send to Kindle»: da un telefono, scarica il libro e condividilo con l’app Kindle; oppure usa l’invio per e-mail.",
        "Een Kindle opent dit bestandsformaat niet rechtstreeks. Stuur het boek met ‘Send to Kindle’: download het op een telefoon en deel het met de Kindle-app, of gebruik het e-mailformulier.",
        "Um Kindle não abre diretamente este formato de ficheiro. Envie-lhe o livro com «Send to Kindle»: num telemóvel, descarregue o livro e partilhe-o com a aplicação Kindle; ou utilize o envio por e-mail.",
    ];
    /// The USB route.
    UsbHint => [
        "Téléchargez le fichier, branchez la liseuse à l’ordinateur avec son câble USB, puis copiez le fichier sur la liseuse.",
        "Download the file, connect the e-reader to the computer with its USB cable, then copy the file onto the e-reader.",
        "Datei herunterladen, den E-Reader per USB-Kabel mit dem Computer verbinden und die Datei auf den E-Reader kopieren.",
        "Descarga el archivo, conecta el lector al ordenador con su cable USB y copia el archivo en el lector.",
        "Scarica il file, collega l’e-reader al computer con il cavo USB e copia il file sull’e-reader.",
        "Download het bestand, sluit de e-reader met de USB-kabel aan op de computer en kopieer het bestand naar de e-reader.",
        "Descarregue o ficheiro, ligue o leitor ao computador com o cabo USB e copie o ficheiro para o leitor.",
    ];
    /// Claim count: nobody has claimed yet.
    NeverClaimed => [
        "Ce livre n’a pas encore été récupéré.", "This book has not been collected yet.",
        "Dieses Buch wurde noch nicht abgeholt.", "Este libro aún no se ha recogido.",
        "Questo libro non è ancora stato ritirato.", "Dit boek is nog niet opgehaald.",
        "Este livro ainda não foi levantado.",
    ];
    /// Claim count. Placeholders: `count`, `date`.
    ClaimedTimes => [
        "Déjà récupéré {count} fois, la dernière fois le {date}.",
        "Already collected {count} time(s), most recently on {date}.",
        "Bereits {count}-mal abgeholt, zuletzt am {date}.",
        "Ya recogido {count} vez/veces, la última el {date}.",
        "Già ritirato {count} volta/e, l’ultima il {date}.",
        "Al {count} keer opgehaald, voor het laatst op {date}.",
        "Já levantado {count} vez(es), a última em {date}.",
    ];
    /// Primary action on a phone.
    ReadOnPhone => [
        "Lire sur ce téléphone ou envoyer vers une application",
        "Read on this phone or send to an app",
        "Auf diesem Smartphone lesen oder an eine App senden",
        "Leer en este teléfono o enviar a una aplicación",
        "Leggere su questo telefono o inviare a un’app",
        "Op deze telefoon lezen of naar een app sturen",
        "Ler neste telemóvel ou enviar para uma aplicação",
    ];
    /// Primary action of a store gift. Placeholder: `store`.
    AddToStoreLibrary => [
        "Ajouter à ma bibliothèque {store}", "Add to my {store} library",
        "Zu meiner {store}-Bibliothek hinzufügen", "Añadir a mi biblioteca de {store}",
        "Aggiungere alla mia biblioteca {store}", "Toevoegen aan mijn {store}-bibliotheek",
        "Adicionar à minha biblioteca {store}",
    ];
    /// Heading of the list of alternatives.
    OtherWays => [
        "Autres façons de lire", "Other ways to read", "Weitere Möglichkeiten",
        "Otras formas de leer", "Altri modi per leggere", "Andere manieren om te lezen",
        "Outras formas de ler",
    ];
    /// The code to type on the e-reader. Placeholder: `code`.
    YourCode => [
        "Code de la carte\u{a0}: {code}", "Code on the card: {code}", "Code auf der Karte: {code}",
        "Código de la tarjeta: {code}", "Codice del biglietto: {code}", "Code op de kaart: {code}",
        "Código do cartão: {code}",
    ];
    /// Offer of a pair code.
    PairIntro => [
        "Code trop long à taper sur la liseuse\u{a0}? Obtenez un code court de 6 lettres, valable 15 minutes.",
        "Code too long to type on the e-reader? Get a short 6-letter code, valid for 15 minutes.",
        "Code zu lang für den E-Reader? Kurzcode mit 6 Buchstaben anfordern, 15 Minuten gültig.",
        "¿El código es demasiado largo para el lector? Obtén un código corto de 6 letras, válido 15 minutos.",
        "Codice troppo lungo da scrivere sull’e-reader? Ottieni un codice breve di 6 lettere, valido 15 minuti.",
        "Code te lang om op de e-reader te typen? Vraag een korte code van 6 letters aan, 15 minuten geldig.",
        "Código demasiado longo para o leitor? Obtenha um código curto de 6 letras, válido durante 15 minutos.",
    ];
    /// Button: create a pair code.
    PairCta => [
        "Obtenir un code court", "Get a short code", "Kurzcode anfordern", "Obtener un código corto",
        "Ottenere un codice breve", "Korte code aanvragen", "Obter um código curto",
    ];
    /// Send to Kindle: the sender to approve first. Placeholder: `sender`.
    ApproveSender => [
        "Avant d’envoyer, ajoutez {sender} aux expéditeurs approuvés de votre compte Amazon (Contenu et appareils, Préférences, Documents personnels).",
        "Before sending, add {sender} to the approved senders of your Amazon account (Content and Devices, Preferences, Personal Document Settings).",
        "Vor dem Senden {sender} im Amazon-Konto als zugelassenen Absender eintragen (Inhalte und Geräte, Einstellungen, Persönliche Dokumente).",
        "Antes de enviar, añade {sender} a los remitentes aprobados de tu cuenta de Amazon (Contenido y dispositivos, Preferencias, Documentos personales).",
        "Prima di inviare, aggiungi {sender} ai mittenti approvati del tuo account Amazon (Contenuti e dispositivi, Preferenze, Documenti personali).",
        "Voeg vóór het versturen {sender} toe aan de goedgekeurde afzenders van je Amazon-account (Inhoud en apparaten, Voorkeuren, Persoonlijke documenten).",
        "Antes de enviar, adicione {sender} aos remetentes aprovados da sua conta Amazon (Conteúdo e dispositivos, Preferências, Documentos pessoais).",
    ];
    /// Label of the device address field. Placeholder: `domains`.
    DeviceEmailLabel => [
        "Adresse e-mail de l’appareil ({domains})", "The device’s e-mail address ({domains})",
        "E-Mail-Adresse des Geräts ({domains})", "Dirección de correo del dispositivo ({domains})",
        "Indirizzo e-mail del dispositivo ({domains})", "E-mailadres van het apparaat ({domains})",
        "Endereço de e-mail do dispositivo ({domains})",
    ];
    /// Button: open the OPDS feed.
    OpdsCta => [
        "Ouvrir dans une application de lecture (OPDS)", "Open in a reading app (OPDS)",
        "In einer Lese-App öffnen (OPDS)", "Abrir en una aplicación de lectura (OPDS)",
        "Aprire in un’app di lettura (OPDS)", "Openen in een lees-app (OPDS)",
        "Abrir numa aplicação de leitura (OPDS)",
    ];
    /// Device chooser: phone or tablet.
    Phone => [
        "Téléphone ou tablette", "Phone or tablet", "Smartphone oder Tablet", "Teléfono o tableta",
        "Telefono o tablet", "Telefoon of tablet", "Telemóvel ou tablet",
    ];
    /// Device chooser: computer.
    Computer => [
        "Ordinateur", "Computer", "Computer", "Ordenador", "Computer", "Computer", "Computador",
    ];
    /// Under a download button.
    RetryDownload => [
        "Le téléchargement n’a pas abouti\u{a0}? Appuyez de nouveau sur le bouton.",
        "Did the download fail? Press the button again.",
        "Download fehlgeschlagen? Schaltfläche erneut antippen.",
        "¿La descarga ha fallado? Pulsa de nuevo el botón.",
        "Il download non è riuscito? Premi di nuovo il pulsante.",
        "Download mislukt? Druk nogmaals op de knop.",
        "A transferência falhou? Prima novamente o botão.",
    ];
    /// The store confirmed the book is in an account. Placeholder: `store`.
    Fulfilled => [
        "Ce livre est déjà dans une bibliothèque {store}. Synchronisez la liseuse ou ouvrez l’application pour le lire.",
        "This book is already in a {store} library. Sync the e-reader or open the app to read it.",
        "Dieses Buch befindet sich bereits in einer {store}-Bibliothek. E-Reader synchronisieren oder die App öffnen, um es zu lesen.",
        "Este libro ya está en una biblioteca de {store}. Sincroniza el lector o abre la aplicación para leerlo.",
        "Questo libro è già in una biblioteca {store}. Sincronizza l’e-reader o apri l’app per leggerlo.",
        "Dit boek staat al in een {store}-bibliotheek. Synchroniseer de e-reader of open de app om het te lezen.",
        "Este livro já está numa biblioteca {store}. Sincronize o leitor ou abra a aplicação para o ler.",
    ];
    /// The download limit is reached.
    Exhausted => [
        "Ce livre a atteint son nombre maximal de téléchargements.",
        "This book has reached its maximum number of downloads.",
        "Für dieses Buch ist die Höchstzahl an Downloads erreicht.",
        "Este libro ha alcanzado su número máximo de descargas.",
        "Questo libro ha raggiunto il numero massimo di download.",
        "Voor dit boek is het maximale aantal downloads bereikt.",
        "Este livro atingiu o número máximo de transferências.",
    ];
    /// Title of a gift that is still wrapped.
    WrappedTitle => [
        "Un cadeau vous attend", "A gift is waiting for you", "Ein Geschenk wartet auf Sie",
        "Te espera un regalo", "Un regalo ti aspetta", "Er wacht een cadeau op je",
        "Um presente espera por si",
    ];
    /// When a wrapped gift opens. Placeholder: `date`.
    RevealOn => [
        "Il pourra être ouvert à partir du {date}.", "It can be opened from {date}.",
        "Es kann ab dem {date} geöffnet werden.", "Podrá abrirse a partir del {date}.",
        "Potrà essere aperto dal {date}.", "Het kan worden geopend vanaf {date}.",
        "Poderá ser aberto a partir de {date}.",
    ];
    /// Store gifts on an e-reader: do it on a phone.
    ResellerEinkHint => [
        "Le plus simple\u{a0}: faites cette étape sur un téléphone ou un ordinateur. Le livre apparaîtra ensuite sur la liseuse à la prochaine synchronisation.",
        "Easiest: do this step on a phone or a computer. The book then appears on the e-reader at its next sync.",
        "Am einfachsten: diesen Schritt auf Smartphone oder Computer erledigen. Das Buch erscheint danach bei der nächsten Synchronisierung auf dem E-Reader.",
        "Lo más sencillo: haz este paso en un teléfono o un ordenador. El libro aparecerá después en el lector en la próxima sincronización.",
        "La via più semplice: esegui questo passaggio su un telefono o un computer. Il libro comparirà poi sull’e-reader alla prossima sincronizzazione.",
        "Het makkelijkst: doe deze stap op een telefoon of computer. Het boek verschijnt daarna op de e-reader bij de volgende synchronisatie.",
        "O mais simples: faça este passo num telemóvel ou num computador. O livro aparecerá depois no leitor na próxima sincronização.",
    ];
    /// Wrong length or character in a gift code.
    CodeFormat => [
        "Un code comporte 12 lettres, sans chiffre ni voyelle.",
        "A code has 12 letters, with no digits and no vowels.",
        "Ein Code besteht aus 12 Buchstaben, ohne Ziffern und ohne Vokale.",
        "Un código tiene 12 letras, sin cifras ni vocales.",
        "Un codice ha 12 lettere, senza cifre né vocali.",
        "Een code heeft 12 letters, zonder cijfers en zonder klinkers.",
        "Um código tem 12 letras, sem algarismos nem vogais.",
    ];
    /// A typo is not an attempt.
    NotCounted => [
        "Cet essai n’est pas compté.", "This try is not counted.", "Dieser Versuch wird nicht gezählt.",
        "Este intento no cuenta.", "Questo tentativo non viene contato.", "Deze poging telt niet mee.",
        "Esta tentativa não é contada.",
    ];
    /// Attempts left. Placeholder: `count`.
    AttemptsLeft => [
        "Essais restants\u{a0}: {count}.", "Tries left: {count}.", "Verbleibende Versuche: {count}.",
        "Intentos restantes: {count}.", "Tentativi rimasti: {count}.", "Resterende pogingen: {count}.",
        "Tentativas restantes: {count}.",
    ];
    /// How long to wait. Placeholder: `minutes`.
    WaitMinutes => [
        "Réessayez dans {minutes} min.", "Try again in {minutes} min.",
        "In {minutes} Min. erneut versuchen.", "Vuelve a intentarlo dentro de {minutes} min.",
        "Riprova tra {minutes} min.", "Probeer het over {minutes} min. opnieuw.",
        "Tente novamente dentro de {minutes} min.",
    ];
    /// The code was replaced by a new one.
    Reissued => [
        "Ce code a été remplacé par un nouveau. Demandez la nouvelle carte à la personne qui vous offre le livre.",
        "This code has been replaced by a new one. Ask the person who gave you the book for the new card.",
        "Dieser Code wurde durch einen neuen ersetzt. Bitte die schenkende Person um die neue Karte bitten.",
        "Este código se ha sustituido por uno nuevo. Pide la nueva tarjeta a quien te regala el libro.",
        "Questo codice è stato sostituito da uno nuovo. Chiedi il nuovo biglietto a chi ti regala il libro.",
        "Deze code is vervangen door een nieuwe. Vraag de gever om de nieuwe kaart.",
        "Este código foi substituído por um novo. Peça o novo cartão a quem lhe oferece o livro.",
    ];
    /// Title of the page showing a pair code.
    PairTitle => [
        "Votre code court", "Your short code", "Ihr Kurzcode", "Tu código corto", "Il tuo codice breve",
        "Je korte code", "O seu código curto",
    ];
    /// What to do with the pair code. Placeholder: `url`.
    PairSteps => [
        "Sur la liseuse, ouvrez le navigateur web, allez sur {url} et saisissez ce code.",
        "On the e-reader, open the web browser, go to {url} and type this code.",
        "Auf dem E-Reader den Webbrowser öffnen, {url} aufrufen und diesen Code eingeben.",
        "En el lector, abre el navegador web, ve a {url} y escribe este código.",
        "Sull’e-reader apri il browser web, vai su {url} e inserisci questo codice.",
        "Open op de e-reader de webbrowser, ga naar {url} en typ deze code.",
        "No leitor, abra o navegador web, vá a {url} e introduza este código.",
    ];
    /// Expiry of the pair code. Placeholders: `time`, `minutes`.
    PairExpires => [
        "Valable {minutes} minutes, jusqu’à {time} UTC, pour une seule utilisation.",
        "Valid for {minutes} minutes, until {time} UTC, for a single use.",
        "{minutes} Minuten gültig, bis {time} UTC, nur einmal verwendbar.",
        "Válido {minutes} minutos, hasta las {time} UTC, para un solo uso.",
        "Valido {minutes} minuti, fino alle {time} UTC, per un solo utilizzo.",
        "{minutes} minuten geldig, tot {time} UTC, voor eenmalig gebruik.",
        "Válido durante {minutes} minutos, até às {time} UTC, para uma única utilização.",
    ];
    /// Title of the pair-code form on the e-reader.
    PairFormTitle => [
        "Saisir le code court", "Enter the short code", "Kurzcode eingeben",
        "Introducir el código corto", "Inserire il codice breve", "De korte code invoeren",
        "Introduzir o código curto",
    ];
    /// Label of the pair-code field.
    PairLabel => [
        "Code de 6 lettres affiché sur le téléphone", "6-letter code shown on the phone",
        "Code mit 6 Buchstaben vom Smartphone", "Código de 6 letras que muestra el teléfono",
        "Codice di 6 lettere mostrato sul telefono", "Code van 6 letters op de telefoon",
        "Código de 6 letras apresentado no telemóvel",
    ];
    /// The pair code is unknown, used or expired.
    PairUnknown => [
        "Ce code court est inconnu, déjà utilisé ou expiré. Demandez-en un nouveau sur le téléphone, ou saisissez le code de la carte.",
        "This short code is unknown, already used or expired. Get a new one on the phone, or type the code on the card.",
        "Dieser Kurzcode ist unbekannt, bereits verwendet oder abgelaufen. Auf dem Smartphone einen neuen anfordern oder den Code der Karte eingeben.",
        "Este código corto es desconocido, ya se ha usado o ha caducado. Pide uno nuevo en el teléfono o escribe el código de la tarjeta.",
        "Questo codice breve è sconosciuto, già usato o scaduto. Richiedine uno nuovo sul telefono oppure inserisci il codice del biglietto.",
        "Deze korte code is onbekend, al gebruikt of verlopen. Vraag op de telefoon een nieuwe aan of typ de code van de kaart.",
        "Este código curto é desconhecido, já foi utilizado ou expirou. Peça um novo no telemóvel ou introduza o código do cartão.",
    ];
    /// Wrong length or character in a pair code.
    PairFormat => [
        "Un code court comporte 6 lettres, sans chiffre ni voyelle.",
        "A short code has 6 letters, with no digits and no vowels.",
        "Ein Kurzcode besteht aus 6 Buchstaben, ohne Ziffern und ohne Vokale.",
        "Un código corto tiene 6 letras, sin cifras ni vocales.",
        "Un codice breve ha 6 lettere, senza cifre né vocali.",
        "Een korte code heeft 6 letters, zonder cijfers en zonder klinkers.",
        "Um código curto tem 6 letras, sem algarismos nem vogais.",
    ];
    /// Link from the pair form to the code form.
    UseCardCode => [
        "Saisir le code de la carte", "Type the code on the card", "Code der Karte eingeben",
        "Escribir el código de la tarjeta", "Inserire il codice del biglietto",
        "De code van de kaart typen", "Introduzir o código do cartão",
    ];
    /// The device address is not acceptable. Placeholder: `domains`.
    SendBadAddress => [
        "Cette adresse n’est pas acceptée. Seules les adresses d’appareil se terminant par {domains} le sont.",
        "This address is not accepted. Only device addresses ending in {domains} are.",
        "Diese Adresse wird nicht akzeptiert. Zulässig sind nur Geräteadressen mit der Endung {domains}.",
        "Esta dirección no se acepta. Solo se admiten direcciones de dispositivo que terminan en {domains}.",
        "Questo indirizzo non è accettato. Sono ammessi solo gli indirizzi di dispositivo che terminano con {domains}.",
        "Dit adres wordt niet geaccepteerd. Alleen apparaatadressen die eindigen op {domains} zijn toegestaan.",
        "Este endereço não é aceite. Só são aceites endereços de dispositivo terminados em {domains}.",
    ];
    /// The mail server refused or is unreachable.
    SendFailed => [
        "L’envoi a échoué. Réessayez plus tard.", "Sending failed. Try again later.",
        "Der Versand ist fehlgeschlagen. Später erneut versuchen.",
        "El envío ha fallado. Vuelve a intentarlo más tarde.", "L’invio non è riuscito. Riprova più tardi.",
        "Het versturen is mislukt. Probeer het later opnieuw.", "O envio falhou. Tente novamente mais tarde.",
    ];
    /// Too many sendings for this gift today.
    SendLimit => [
        "Ce livre a déjà été envoyé plusieurs fois aujourd’hui. Réessayez demain.",
        "This book has already been sent several times today. Try again tomorrow.",
        "Dieses Buch wurde heute bereits mehrfach gesendet. Morgen erneut versuchen.",
        "Este libro ya se ha enviado varias veces hoy. Vuelve a intentarlo mañana.",
        "Questo libro è già stato inviato più volte oggi. Riprova domani.",
        "Dit boek is vandaag al meerdere keren verstuurd. Probeer het morgen opnieuw.",
        "Este livro já foi enviado várias vezes hoje. Tente novamente amanhã.",
    ];
    /// The partner's file is too large to be sent by e-mail.
    SendTooLarge => [
        "Ce fichier est trop volumineux pour être envoyé par e-mail. Téléchargez-le plutôt.",
        "This file is too large to be sent by e-mail. Download it instead.",
        "Diese Datei ist zu groß für den Versand per E-Mail. Bitte stattdessen herunterladen.",
        "Este archivo es demasiado grande para enviarlo por correo. Descárgalo en su lugar.",
        "Questo file è troppo grande per essere inviato per e-mail. Scaricalo invece.",
        "Dit bestand is te groot om per e-mail te versturen. Download het in plaats daarvan.",
        "Este ficheiro é demasiado grande para ser enviado por e-mail. Transfira-o em vez disso.",
    ];
    /// Footer link to the service's contact address.
    Contact => ["Contact", "Contact", "Kontakt", "Contacto", "Contatti", "Contact", "Contacto"];
    /// Help page: the contact point. Placeholder: `email`.
    ContactText => [
        "Une question sur un cadeau, ou un contenu illicite à signaler : écrivez à {email}.",
        "A question about a gift, or illegal content to report: write to {email}.",
        "Eine Frage zu einem Geschenk oder rechtswidrige Inhalte zu melden: Schreiben Sie an {email}.",
        "¿Una pregunta sobre un regalo o un contenido ilícito que denunciar? Escribe a {email}.",
        "Una domanda su un regalo o un contenuto illecito da segnalare: scrivi a {email}.",
        "Een vraag over een cadeau, of illegale inhoud melden: schrijf naar {email}.",
        "Uma pergunta sobre um presente, ou conteúdo ilícito a denunciar: escreva para {email}.",
    ];
    /// Link back to the gift page.
    BackToGift => [
        "Revenir au livre", "Back to the book", "Zurück zum Buch", "Volver al libro",
        "Tornare al libro", "Terug naar het boek", "Voltar ao livro",
    ];
    /// A download link that no longer works.
    SessionExpired => [
        "Ce lien de téléchargement a expiré. Saisissez de nouveau le code de la carte.",
        "This download link has expired. Type the code on the card again.",
        "Dieser Download-Link ist abgelaufen. Den Code der Karte erneut eingeben.",
        "Este enlace de descarga ha caducado. Escribe de nuevo el código de la tarjeta.",
        "Questo link di download è scaduto. Inserisci di nuovo il codice del biglietto.",
        "Deze downloadlink is verlopen. Typ de code van de kaart opnieuw.",
        "Esta ligação de transferência expirou. Introduza novamente o código do cartão.",
    ];
    /// No such page.
    NotFound => [
        "Cette page n’existe pas.", "This page does not exist.", "Diese Seite gibt es nicht.",
        "Esta página no existe.", "Questa pagina non esiste.", "Deze pagina bestaat niet.",
        "Esta página não existe.",
    ];
    /// The store could not deliver right now.
    StoreUnavailable => [
        "La boutique n’a pas pu délivrer le livre pour l’instant. Réessayez dans quelques minutes.",
        "The store could not deliver the book right now. Try again in a few minutes.",
        "Der Shop konnte das Buch gerade nicht bereitstellen. In einigen Minuten erneut versuchen.",
        "La tienda no ha podido entregar el libro en este momento. Vuelve a intentarlo en unos minutos.",
        "Il negozio non ha potuto consegnare il libro in questo momento. Riprova tra qualche minuto.",
        "De winkel kon het boek nu niet leveren. Probeer het over een paar minuten opnieuw.",
        "A loja não conseguiu entregar o livro neste momento. Tente novamente dentro de alguns minutos.",
    ];
    /// Format line of an open file. Placeholder: `format`.
    FormatOpen => [
        "Fichier {format} sans DRM, pour liseuses, tablettes et téléphones.",
        "{format} file without DRM, for e-readers, tablets and phones.",
        "{format}-Datei ohne DRM, für E-Reader, Tablets und Smartphones.",
        "Archivo {format} sin DRM, para lectores, tabletas y teléfonos.",
        "File {format} senza DRM, per e-reader, tablet e telefoni.",
        "{format}-bestand zonder DRM, voor e-readers, tablets en telefoons.",
        "Ficheiro {format} sem DRM, para leitores, tablets e telemóveis.",
    ];
    /// Format line of a store gift. Placeholder: `store`.
    FormatStore => [
        "À lire avec un compte {store}.", "To be read with a {store} account.",
        "Zu lesen mit einem {store}-Konto.", "Para leer con una cuenta de {store}.",
        "Da leggere con un account {store}.", "Te lezen met een {store}-account.",
        "Para ler com uma conta {store}.",
    ];
    /// The requested action does not exist for this gift.
    NotAvailable => [
        "Cette action n’est pas disponible pour ce livre.", "This action is not available for this book.",
        "Diese Aktion ist für dieses Buch nicht verfügbar.", "Esta acción no está disponible para este libro.",
        "Questa azione non è disponibile per questo libro.", "Deze actie is niet beschikbaar voor dit boek.",
        "Esta ação não está disponível para este livro.",
    ];
    /// Body of the e-mail carrying the book. Placeholder: `title`.
    MailBody => [
        "Voici votre livre\u{a0}: {title}. Bonne lecture\u{a0}!", "Here is your book: {title}. Enjoy!",
        "Hier ist Ihr Buch: {title}. Viel Freude beim Lesen!", "Aquí tienes tu libro: {title}. ¡Buena lectura!",
        "Ecco il tuo libro: {title}. Buona lettura!", "Hier is je boek: {title}. Veel leesplezier!",
        "Aqui está o seu livro: {title}. Boa leitura!",
    ];
}

/// The text of `msg` in `locale`.
pub(crate) fn s(locale: Locale, msg: S) -> &'static str {
    let index = match locale {
        Locale::Fr => 0,
        Locale::En => 1,
        Locale::De => 2,
        Locale::Es => 3,
        Locale::It => 4,
        Locale::Nl => 5,
        Locale::Pt => 6,
    };
    msg.texts()[index]
}

/// The text of `msg` in `locale` with `{name}` placeholders replaced (single pass, values
/// inserted verbatim).
pub(crate) fn sf(locale: Locale, msg: S, args: &[(&str, &str)]) -> String {
    ebc_i18n::format_template(s(locale, msg), args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_message_is_translated_with_the_same_placeholders_everywhere() {
        for msg in S::ALL {
            let mut reference: Vec<&str> = ebc_i18n::placeholders_in(s(Locale::Fr, *msg));
            reference.sort_unstable();
            for locale in Locale::ALL {
                let text = s(locale, *msg);
                assert!(!text.trim().is_empty(), "{msg:?} is empty in {locale}");
                let mut found = ebc_i18n::placeholders_in(text);
                found.sort_unstable();
                assert_eq!(found, reference, "{msg:?} in {locale}");
            }
        }
    }

    #[test]
    fn no_string_uses_a_stored_value_word_or_markup() {
        let banned = [
            "carte cadeau",
            "bon d",
            "gift card",
            "voucher",
            "gutschein",
            "cadeaubon",
            "tarjeta regalo",
            "buono",
            "vale",
            "<",
            ">",
        ];
        for msg in S::ALL {
            for locale in Locale::ALL {
                let text = s(locale, *msg).to_lowercase();
                for word in banned {
                    assert!(!text.contains(word), "{msg:?} in {locale} contains {word:?}");
                }
            }
        }
    }

    #[test]
    fn placeholders_are_filled_once_and_never_re_expanded() {
        assert_eq!(sf(Locale::En, S::AttemptsLeft, &[("count", "3")]), "Tries left: 3.");
        assert_eq!(
            sf(Locale::En, S::AddToStoreLibrary, &[("store", "{store}{count}")]),
            "Add to my {store}{count} library"
        );
    }
}
