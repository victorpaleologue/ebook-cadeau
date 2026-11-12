//! The shelf's own strings, in the seven supported locales (fr, en, de, es, it, nl, pt).
//!
//! Same rules as the core's: a gift is *a book*, never a balance; French says *vous*.
//! Written by one author, **not reviewed by native speakers**; the legal wording (terms,
//! statements of reasons) is a **draft not reviewed by counsel**. Placeholders (`{name}`)
//! are filled by [`sf`]; values are escaped by the templates, not here.

use ebc_model::Locale;

macro_rules! strings {
    ($( $(#[$doc:meta])* $name:ident => [$fr:expr, $en:expr, $de:expr, $es:expr, $it:expr, $nl:expr, $pt:expr $(,)?]; )+) => {
        /// A shelf message.
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

/// The text of `msg` in `locale`.
pub(crate) fn s(locale: Locale, msg: S) -> &'static str {
    let index = Locale::ALL.iter().position(|l| *l == locale).unwrap_or(0);
    msg.texts()[index]
}

/// [`s`] with `{name}` placeholders replaced.
pub(crate) fn sf(locale: Locale, msg: S, args: &[(&str, &str)]) -> String {
    let mut text = s(locale, msg).to_owned();
    for (name, value) in args {
        text = text.replace(&format!("{{{name}}}"), value);
    }
    text
}

strings! {
    // ----- chrome -----
    /// Navigation: account.
    NavAccount => ["Mon compte", "My account", "Mein Konto", "Mi cuenta", "Il mio account", "Mijn account", "A minha conta"];
    /// Navigation: sign in.
    NavSignIn => ["Se connecter", "Sign in", "Anmelden", "Iniciar sesión", "Accedi", "Inloggen", "Iniciar sessão"];
    /// Button: sign out.
    NavSignOut => ["Se déconnecter", "Sign out", "Abmelden", "Cerrar sesión", "Esci", "Uitloggen", "Terminar sessão"];
    /// Navigation: notice form.
    NavNotice => ["Signaler un contenu", "Report content", "Inhalt melden", "Denunciar un contenido", "Segnala un contenuto", "Inhoud melden", "Denunciar um conteúdo"];
    /// Navigation: terms.
    NavTerms => ["Conditions", "Terms", "Bedingungen", "Condiciones", "Condizioni", "Voorwaarden", "Condições"];
    /// Navigation: legal notice.
    NavLegal => ["Mentions légales", "Legal notice", "Impressum", "Aviso legal", "Note legali", "Colofon", "Informação legal"];
    /// Navigation: contact point.
    NavContact => ["Contact", "Contact", "Kontakt", "Contacto", "Contatti", "Contact", "Contacto"];
    /// Navigation: moderation.
    NavModeration => ["Modération", "Moderation", "Moderation", "Moderación", "Moderazione", "Moderatie", "Moderação"];
    /// Banner of the legal templates.
    Draft => [
        "Brouillon — texte non relu par un avocat.",
        "Draft — not reviewed by counsel.",
        "Entwurf — nicht anwaltlich geprüft.",
        "Borrador — no revisado por un abogado.",
        "Bozza — non rivista da un avvocato.",
        "Concept — niet door een jurist nagelezen.",
        "Rascunho — não revisto por um advogado.",
    ];
    /// Marks optional fields.
    Optional => ["(facultatif)", "(optional)", "(optional)", "(opcional)", "(facoltativo)", "(optioneel)", "(opcional)"];
    /// Generic save button.
    Save => ["Enregistrer", "Save", "Speichern", "Guardar", "Salva", "Opslaan", "Guardar"];
    /// 404.
    NotFound => ["Page introuvable.", "Page not found.", "Seite nicht gefunden.", "Página no encontrada.", "Pagina non trovata.", "Pagina niet gevonden.", "Página não encontrada."];
    /// 403.
    Forbidden => ["Accès refusé.", "Access denied.", "Zugriff verweigert.", "Acceso denegado.", "Accesso negato.", "Toegang geweigerd.", "Acesso recusado."];
    /// 500.
    ServerError => [
        "Une erreur est survenue. Réessayez dans un instant.",
        "Something went wrong. Please try again in a moment.",
        "Ein Fehler ist aufgetreten. Bitte gleich noch einmal versuchen.",
        "Se ha producido un error. Inténtalo de nuevo en un momento.",
        "Si è verificato un errore. Riprova tra un momento.",
        "Er ging iets mis. Probeer het zo opnieuw.",
        "Ocorreu um erro. Tente novamente dentro de momentos.",
    ];
    /// 429.
    TooMany => [
        "Trop de demandes. Réessayez dans {minutes} minutes.",
        "Too many requests. Try again in {minutes} minutes.",
        "Zu viele Anfragen. Bitte in {minutes} Minuten erneut versuchen.",
        "Demasiadas solicitudes. Inténtalo de nuevo dentro de {minutes} minutos.",
        "Troppe richieste. Riprova tra {minutes} minuti.",
        "Te veel verzoeken. Probeer het over {minutes} minuten opnieuw.",
        "Demasiados pedidos. Tente novamente dentro de {minutes} minutos.",
    ];
    /// Stale form token.
    BadForm => [
        "Ce formulaire a expiré. Rechargez la page et recommencez.",
        "This form has expired. Reload the page and try again.",
        "Dieses Formular ist abgelaufen. Seite neu laden und erneut versuchen.",
        "Este formulario ha caducado. Recarga la página y vuelve a intentarlo.",
        "Questo modulo è scaduto. Ricarica la pagina e riprova.",
        "Dit formulier is verlopen. Laad de pagina opnieuw en probeer het nog eens.",
        "Este formulário expirou. Recarregue a página e tente novamente.",
    ];
    /// Size in megabytes.
    Megabytes => ["{n} Mo", "{n} MB", "{n} MB", "{n} MB", "{n} MB", "{n} MB", "{n} MB"];
    /// Size in kilobytes.
    Kilobytes => ["{n} ko", "{n} kB", "{n} kB", "{n} kB", "{n} kB", "{n} kB", "{n} kB"];

    // ----- home -----
    /// Home: what the shelf is.
    HomeIntro => [
        "Ce rayonnage héberge des livres numériques que leurs ayants droit offrent avec une carte eBook Cadeau. Il n’a ni catalogue ni recherche\u{a0}: un livre n’est remis qu’à qui détient une carte.",
        "This shelf hosts eBooks that their rights holders give with an eBook Cadeau card. It has no catalogue and no search: a book is only delivered to whoever holds a card.",
        "Dieses Regal beherbergt E-Books, die ihre Rechteinhaber mit einer eBook-Cadeau-Karte verschenken. Es gibt keinen Katalog und keine Suche: Ein Buch erhält nur, wer eine Karte besitzt.",
        "Esta estantería aloja libros electrónicos que sus titulares de derechos regalan con una tarjeta eBook Cadeau. No tiene catálogo ni buscador: un libro solo se entrega a quien tiene una tarjeta.",
        "Questo scaffale ospita libri digitali che i titolari dei diritti regalano con un biglietto eBook Cadeau. Non ha catalogo né ricerca: un libro viene consegnato solo a chi possiede un biglietto.",
        "Deze plank bewaart e-books die hun rechthebbenden cadeau doen met een eBook Cadeau-kaart. Er is geen catalogus en geen zoekfunctie: een boek gaat alleen naar wie een kaart heeft.",
        "Esta estante aloja livros digitais que os titulares dos direitos oferecem com um cartão eBook Cadeau. Não tem catálogo nem pesquisa: um livro só é entregue a quem tem um cartão.",
    ];
    /// Home, `own` mode.
    HomeOwn => [
        "Il est tenu par son auteur ou son éditeur, pour ses propres livres.",
        "It is run by its author or publisher, for their own books.",
        "Es wird von seiner Autorin oder seinem Verlag für die eigenen Bücher betrieben.",
        "La gestiona su autor o su editorial, para sus propios libros.",
        "È gestito dal suo autore o editore, per i propri libri.",
        "Het wordt beheerd door de auteur of uitgever, voor de eigen boeken.",
        "É gerida pelo seu autor ou editor, para os seus próprios livros.",
    ];
    /// Home, `multi` mode.
    HomeMulti => [
        "Autrices, auteurs, éditeurs et diffuseurs autorisés\u{a0}: créez un compte pour héberger vos livres et en offrir.",
        "Authors, publishers and authorised distributors: create an account to host your books and give them.",
        "Autorinnen, Autoren, Verlage und autorisierte Vertriebe: Legen Sie ein Konto an, um Ihre Bücher hier zu hosten und zu verschenken.",
        "Autoras, autores, editoriales y distribuidores autorizados: cread una cuenta para alojar vuestros libros y regalarlos.",
        "Autrici, autori, editori e distributori autorizzati: create un account per ospitare i vostri libri e regalarli.",
        "Auteurs, uitgevers en erkende distributeurs: maak een account om je boeken hier te bewaren en cadeau te doen.",
        "Autoras, autores, editores e distribuidores autorizados: criem uma conta para alojar e oferecer os vossos livros.",
    ];
    /// The purchased-book warning (research 09 §3.3 item 1, implication 2).
    NotPurchases => [
        "Un livre acheté dans une librairie numérique ne peut pas être déposé ici\u{a0}: c’est une licence personnelle. Pour l’offrir, utilisez l’option cadeau de la librairie.",
        "A book bought from an eBook store cannot be uploaded here: it is a personal licence. To give it, use the store’s gift option.",
        "Ein in einem E-Book-Shop gekauftes Buch kann hier nicht hochgeladen werden: Es ist eine persönliche Lizenz. Zum Verschenken die Geschenkoption des Shops nutzen.",
        "Un libro comprado en una librería digital no puede subirse aquí: es una licencia personal. Para regalarlo, usa la opción de regalo de la librería.",
        "Un libro acquistato in una libreria digitale non può essere caricato qui: è una licenza personale. Per regalarlo, usa l’opzione regalo della libreria.",
        "Een boek dat je in een e-bookwinkel kocht, kan hier niet worden geüpload: het is een persoonlijke licentie. Gebruik de cadeauoptie van de winkel om het te geven.",
        "Um livro comprado numa livraria digital não pode ser carregado aqui: é uma licença pessoal. Para o oferecer, use a opção de oferta da livraria.",
    ];
    /// Home: for recipients.
    HomeRecipient => [
        "Vous avez reçu une carte\u{a0}? Ouvrez l’adresse imprimée dessus et tapez son code\u{a0}: ce n’est pas ici.",
        "Did you receive a card? Open the address printed on it and type its code: it is not here.",
        "Sie haben eine Karte bekommen? Öffnen Sie die darauf gedruckte Adresse und geben Sie den Code ein – nicht hier.",
        "¿Has recibido una tarjeta? Abre la dirección impresa en ella y escribe su código: no es aquí.",
        "Hai ricevuto un biglietto? Apri l’indirizzo stampato e digita il codice: non è qui.",
        "Een kaart gekregen? Open het adres dat erop staat en typ de code: dat is niet hier.",
        "Recebeu um cartão? Abra o endereço nele impresso e escreva o código: não é aqui.",
    ];

    // ----- sign-in -----
    /// Sign-in title.
    SignInTitle => ["Connexion", "Sign in", "Anmeldung", "Iniciar sesión", "Accesso", "Inloggen", "Iniciar sessão"];
    /// Sign-in explanation.
    SignInIntro => [
        "Nous vous envoyons un lien de connexion par e-mail. Il est valable {minutes} minutes et ne sert qu’une fois. Il n’y a pas de mot de passe.",
        "We send you a sign-in link by e-mail. It is valid for {minutes} minutes and works once. There is no password.",
        "Wir senden Ihnen einen Anmeldelink per E-Mail. Er gilt {minutes} Minuten und nur einmal. Es gibt kein Passwort.",
        "Te enviamos un enlace de acceso por correo. Es válido {minutes} minutos y sirve una sola vez. No hay contraseña.",
        "Ti inviamo un link di accesso via e-mail. È valido {minutes} minuti e funziona una sola volta. Non c’è password.",
        "We sturen je een inloglink per e-mail. Die is {minutes} minuten geldig en werkt één keer. Er is geen wachtwoord.",
        "Enviamos-lhe uma ligação de acesso por e-mail. É válida durante {minutes} minutos e só funciona uma vez. Não há palavra-passe.",
    ];
    /// E-mail field.
    EmailLabel => ["Adresse e-mail", "E-mail address", "E-Mail-Adresse", "Correo electrónico", "Indirizzo e-mail", "E-mailadres", "Endereço de e-mail"];
    /// Button: send the link.
    SendLink => ["Recevoir le lien", "Send me the link", "Link zusenden", "Recibir el enlace", "Ricevi il link", "Stuur me de link", "Receber a ligação"];
    /// After asking for a link.
    LinkSentTitle => ["Vérifiez votre messagerie", "Check your mailbox", "Bitte Postfach prüfen", "Revisa tu correo", "Controlla la posta", "Kijk in je mailbox", "Verifique o seu e-mail"];
    /// After asking for a link (no account enumeration).
    LinkSent => [
        "Si cette adresse peut se connecter ici, un lien vient de lui être envoyé. Il expire dans {minutes} minutes.",
        "If this address may sign in here, a link has just been sent to it. It expires in {minutes} minutes.",
        "Falls sich diese Adresse hier anmelden darf, wurde gerade ein Link an sie gesendet. Er läuft in {minutes} Minuten ab.",
        "Si esta dirección puede iniciar sesión aquí, se le acaba de enviar un enlace. Caduca en {minutes} minutos.",
        "Se questo indirizzo può accedere qui, gli è appena stato inviato un link. Scade tra {minutes} minuti.",
        "Als dit adres hier mag inloggen, is er net een link naartoe gestuurd. Die verloopt over {minutes} minuten.",
        "Se este endereço puder iniciar sessão aqui, acabou de lhe ser enviada uma ligação. Expira dentro de {minutes} minutos.",
    ];
    /// `own` mode: sign-up closed.
    SignupClosed => [
        "Ce rayonnage n’ouvre pas de compte\u{a0}: il est tenu par une seule personne pour ses propres livres.",
        "This shelf does not open accounts: one person runs it for their own books.",
        "Dieses Regal eröffnet keine Konten: Eine einzelne Person betreibt es für ihre eigenen Bücher.",
        "Esta estantería no abre cuentas: la gestiona una sola persona para sus propios libros.",
        "Questo scaffale non apre account: lo gestisce una sola persona per i propri libri.",
        "Deze plank opent geen accounts: één persoon beheert haar voor de eigen boeken.",
        "Esta estante não abre contas: é gerida por uma única pessoa para os seus próprios livros.",
    ];
    /// Bad e-mail address.
    InvalidEmail => ["Cette adresse e-mail n’est pas valable.", "This e-mail address is not valid.", "Diese E-Mail-Adresse ist ungültig.", "Esta dirección de correo no es válida.", "Questo indirizzo e-mail non è valido.", "Dit e-mailadres is ongeldig.", "Este endereço de e-mail não é válido."];
    /// Mail could not be sent.
    MailFailed => [
        "Le message n’a pas pu être envoyé. Réessayez plus tard.",
        "The message could not be sent. Please try again later.",
        "Die Nachricht konnte nicht gesendet werden. Bitte später erneut versuchen.",
        "No se ha podido enviar el mensaje. Inténtalo más tarde.",
        "Non è stato possibile inviare il messaggio. Riprova più tardi.",
        "Het bericht kon niet worden verzonden. Probeer het later opnieuw.",
        "Não foi possível enviar a mensagem. Tente novamente mais tarde.",
    ];
    /// Confirmation page of a sign-in link.
    ConfirmIntro => [
        "Appuyez sur le bouton pour vous connecter avec {email}.",
        "Press the button to sign in as {email}.",
        "Drücken Sie die Schaltfläche, um sich als {email} anzumelden.",
        "Pulsa el botón para iniciar sesión como {email}.",
        "Premi il pulsante per accedere come {email}.",
        "Druk op de knop om in te loggen als {email}.",
        "Carregue no botão para iniciar sessão como {email}.",
    ];
    /// Button: confirm sign-in.
    ConfirmButton => ["Me connecter", "Sign me in", "Anmelden", "Entrar", "Entra", "Inloggen", "Entrar"];
    /// Link used or expired.
    LinkInvalid => [
        "Ce lien de connexion n’est plus valable (déjà utilisé ou expiré). Demandez-en un nouveau.",
        "This sign-in link is no longer valid (already used or expired). Ask for a new one.",
        "Dieser Anmeldelink ist nicht mehr gültig (bereits benutzt oder abgelaufen). Fordern Sie einen neuen an.",
        "Este enlace de acceso ya no es válido (usado o caducado). Pide uno nuevo.",
        "Questo link di accesso non è più valido (già usato o scaduto). Richiedine uno nuovo.",
        "Deze inloglink is niet meer geldig (al gebruikt of verlopen). Vraag een nieuwe aan.",
        "Esta ligação de acesso já não é válida (já usada ou expirada). Peça uma nova.",
    ];
    /// Sign-in mail subject.
    MailLoginSubject => ["Votre lien de connexion — {shelf}", "Your sign-in link — {shelf}", "Ihr Anmeldelink — {shelf}", "Tu enlace de acceso — {shelf}", "Il tuo link di accesso — {shelf}", "Je inloglink — {shelf}", "A sua ligação de acesso — {shelf}"];
    /// Sign-in mail body.
    MailLoginBody => [
        "Bonjour,\n\nPour vous connecter à {shelf}, ouvrez ce lien (valable {minutes} minutes, une seule fois)\u{a0}:\n\n{link}\n\nSi vous n’avez rien demandé, ignorez ce message\u{a0}: rien ne se passera.\n",
        "Hello,\n\nTo sign in to {shelf}, open this link (valid for {minutes} minutes, once):\n\n{link}\n\nIf you did not ask for it, ignore this message: nothing will happen.\n",
        "Guten Tag,\n\num sich bei {shelf} anzumelden, öffnen Sie diesen Link ({minutes} Minuten gültig, nur einmal):\n\n{link}\n\nFalls Sie nichts angefordert haben, ignorieren Sie diese Nachricht.\n",
        "Hola:\n\nPara entrar en {shelf}, abre este enlace (válido {minutes} minutos, una sola vez):\n\n{link}\n\nSi no lo has pedido, ignora este mensaje.\n",
        "Buongiorno,\n\nper accedere a {shelf}, apri questo link (valido {minutes} minuti, una sola volta):\n\n{link}\n\nSe non l’hai richiesto, ignora questo messaggio.\n",
        "Hallo,\n\nOpen deze link om in te loggen bij {shelf} ({minutes} minuten geldig, één keer):\n\n{link}\n\nHeb je niets aangevraagd? Negeer dit bericht dan.\n",
        "Olá,\n\nPara entrar em {shelf}, abra esta ligação (válida durante {minutes} minutos, uma só vez):\n\n{link}\n\nSe não pediu nada, ignore esta mensagem.\n",
    ];

    // ----- account -----
    /// Account: who is signed in.
    SignedInAs => ["Connecté avec {email}", "Signed in as {email}", "Angemeldet als {email}", "Sesión iniciada como {email}", "Accesso effettuato come {email}", "Ingelogd als {email}", "Sessão iniciada como {email}"];
    /// Identity missing.
    IdentityMissing => [
        "Avant de déposer un livre, complétez votre identification\u{a0}: l’hébergeur doit pouvoir identifier qui met un contenu en ligne.",
        "Before uploading a book, complete your identification: the host must be able to identify who puts content online.",
        "Bevor Sie ein Buch hochladen, vervollständigen Sie bitte Ihre Angaben: Der Hoster muss feststellen können, wer Inhalte veröffentlicht.",
        "Antes de subir un libro, completa tu identificación: el alojador debe poder identificar a quien publica un contenido.",
        "Prima di caricare un libro, completa la tua identificazione: l’hosting deve poter identificare chi pubblica un contenuto.",
        "Vul eerst je identificatie in voordat je een boek uploadt: de hostingdienst moet kunnen nagaan wie inhoud online zet.",
        "Antes de carregar um livro, complete a sua identificação: o alojamento tem de poder identificar quem publica um conteúdo.",
    ];
    /// Link to the identity form.
    CompleteIdentity => ["Compléter mon identification", "Complete my identification", "Angaben vervollständigen", "Completar mi identificación", "Completa la mia identificazione", "Mijn identificatie invullen", "Completar a minha identificação"];
    /// Link to edit the identity.
    EditIdentity => ["Modifier mon identification", "Edit my identification", "Angaben ändern", "Modificar mi identificación", "Modifica la mia identificazione", "Mijn identificatie wijzigen", "Alterar a minha identificação"];
    /// Suspended account.
    Suspended => [
        "Votre compte est suspendu\u{a0}: vous ne pouvez plus déposer de livre ni créer de cadeau. Les décisions et les voies de recours sont ci-dessous.",
        "Your account is suspended: you can no longer upload books or create gifts. The decisions and how to contest them are below.",
        "Ihr Konto ist gesperrt: Sie können keine Bücher mehr hochladen und keine Geschenke anlegen. Die Entscheidungen und Rechtsbehelfe stehen unten.",
        "Tu cuenta está suspendida: ya no puedes subir libros ni crear regalos. Las decisiones y las vías de recurso están abajo.",
        "Il tuo account è sospeso: non puoi più caricare libri né creare regali. Le decisioni e i mezzi di ricorso sono qui sotto.",
        "Je account is geschorst: je kunt geen boeken meer uploaden en geen cadeaus maken. De beslissingen en bezwaarmogelijkheden staan hieronder.",
        "A sua conta está suspensa: já não pode carregar livros nem criar ofertas. As decisões e as vias de recurso estão abaixo.",
    ];
    /// My books heading.
    MyBooks => ["Mes livres", "My books", "Meine Bücher", "Mis libros", "I miei libri", "Mijn boeken", "Os meus livros"];
    /// No book yet.
    NoBooks => ["Aucun livre pour l’instant.", "No books yet.", "Noch keine Bücher.", "Todavía no hay libros.", "Ancora nessun libro.", "Nog geen boeken.", "Ainda não há livros."];
    /// Upload link / title.
    UploadBook => ["Déposer un livre", "Upload a book", "Buch hochladen", "Subir un libro", "Carica un libro", "Boek uploaden", "Carregar um livro"];
    /// Decisions heading.
    Decisions => ["Décisions me concernant", "Decisions about me", "Entscheidungen zu meinem Konto", "Decisiones que me afectan", "Decisioni che mi riguardano", "Beslissingen over mij", "Decisões que me dizem respeito"];
    /// Storage used.
    StorageUsed => ["Espace utilisé\u{a0}: {used} sur {quota}.", "Storage used: {used} of {quota}.", "Belegter Speicher: {used} von {quota}.", "Espacio usado: {used} de {quota}.", "Spazio usato: {used} su {quota}.", "Gebruikte opslag: {used} van {quota}.", "Espaço usado: {used} de {quota}."];
    /// Close account heading/button.
    CloseAccount => ["Fermer mon compte", "Close my account", "Konto schließen", "Cerrar mi cuenta", "Chiudi il mio account", "Mijn account sluiten", "Encerrar a minha conta"];
    /// Close account explanation.
    CloseAccountHelp => [
        "Vos livres sont effacés et leurs cadeaux annulés\u{a0}: leurs codes ne fonctionneront plus. Vos données d’identification sont conservées pendant la durée légale, puis effacées.",
        "Your books are erased and their gifts cancelled: their codes will stop working. Your identification data is kept for the legal period, then erased.",
        "Ihre Bücher werden gelöscht und ihre Geschenke storniert: Die Codes funktionieren dann nicht mehr. Ihre Identifikationsdaten werden für die gesetzliche Frist aufbewahrt und danach gelöscht.",
        "Tus libros se borran y sus regalos se anulan: sus códigos dejarán de funcionar. Tus datos de identificación se conservan durante el plazo legal y después se borran.",
        "I tuoi libri vengono cancellati e i loro regali annullati: i codici smetteranno di funzionare. I dati di identificazione sono conservati per il periodo di legge, poi cancellati.",
        "Je boeken worden gewist en hun cadeaus geannuleerd: de codes werken dan niet meer. Je identificatiegegevens worden de wettelijke termijn bewaard en daarna gewist.",
        "Os seus livros são apagados e as ofertas anuladas: os códigos deixam de funcionar. Os seus dados de identificação são conservados durante o prazo legal e depois apagados.",
    ];
    /// Close account confirmation checkbox.
    CloseConfirm => ["Je confirme la fermeture de mon compte", "I confirm closing my account", "Ich bestätige die Schließung meines Kontos", "Confirmo el cierre de mi cuenta", "Confermo la chiusura del mio account", "Ik bevestig het sluiten van mijn account", "Confirmo o encerramento da minha conta"];
    /// After closing.
    AccountClosed => ["Votre compte est fermé.", "Your account is closed.", "Ihr Konto ist geschlossen.", "Tu cuenta está cerrada.", "Il tuo account è chiuso.", "Je account is gesloten.", "A sua conta está encerrada."];

    // ----- identity -----
    /// Identity title.
    IdentityTitle => ["Identification", "Identification", "Identifikation", "Identificación", "Identificazione", "Identificatie", "Identificação"];
    /// Identity explanation.
    IdentityIntro => [
        "Ces informations ne sont jamais publiées. L’hébergeur doit les conserver pour pouvoir identifier qui met un contenu en ligne (LCEN, art. 6\u{a0}; décret n° 2021-1362).",
        "This information is never published. The host must keep it to be able to identify who puts content online (French LCEN art. 6; decree 2021-1362).",
        "Diese Angaben werden nie veröffentlicht. Der Hoster muss sie aufbewahren, um feststellen zu können, wer Inhalte veröffentlicht (franz. LCEN Art. 6; Dekret 2021-1362).",
        "Estos datos nunca se publican. El alojador debe conservarlos para poder identificar a quien publica un contenido (LCEN francesa, art. 6; decreto 2021-1362).",
        "Queste informazioni non sono mai pubblicate. L’hosting deve conservarle per poter identificare chi pubblica un contenuto (LCEN francese, art. 6; decreto 2021-1362).",
        "Deze gegevens worden nooit gepubliceerd. De hostingdienst moet ze bewaren om te kunnen nagaan wie inhoud online zet (Franse LCEN art. 6; decreet 2021-1362).",
        "Estas informações nunca são publicadas. O alojamento tem de as conservar para poder identificar quem publica um conteúdo (LCEN francesa, art. 6; decreto 2021-1362).",
    ];
    /// Person or organisation.
    KindLegend => ["Vous êtes", "You are", "Sie sind", "Eres", "Sei", "Je bent", "É"];
    /// Natural person.
    KindPerson => ["une personne", "a person", "eine Person", "una persona", "una persona", "een persoon", "uma pessoa"];
    /// Organisation.
    KindOrganisation => ["une entreprise ou une association", "a company or an association", "ein Unternehmen oder ein Verein", "una empresa o una asociación", "un’impresa o un’associazione", "een bedrijf of vereniging", "uma empresa ou associação"];
    /// Name.
    NameLabel => ["Nom et prénom, ou raison sociale", "Full name, or company name", "Vor- und Nachname oder Firmenname", "Nombre y apellidos, o razón social", "Nome e cognome, o ragione sociale", "Voor- en achternaam, of bedrijfsnaam", "Nome completo, ou denominação social"];
    /// Representative.
    RepresentativeLabel => ["Pour une entreprise\u{a0}: la personne qui agit pour elle", "For a company: the person acting for it", "Für ein Unternehmen: die handelnde Person", "Para una empresa: la persona que actúa en su nombre", "Per un’impresa: la persona che agisce per essa", "Voor een bedrijf: de persoon die namens het bedrijf handelt", "Para uma empresa: a pessoa que age em seu nome"];
    /// Address.
    AddressLabel => ["Adresse postale", "Postal address", "Postanschrift", "Dirección postal", "Indirizzo postale", "Postadres", "Morada"];
    /// Phone.
    PhoneLabel => ["Téléphone", "Telephone", "Telefon", "Teléfono", "Telefono", "Telefoon", "Telefone"];
    /// Birth date.
    BirthDateLabel => ["Date de naissance", "Date of birth", "Geburtsdatum", "Fecha de nacimiento", "Data di nascita", "Geboortedatum", "Data de nascimento"];
    /// Birth place.
    BirthPlaceLabel => ["Lieu de naissance", "Place of birth", "Geburtsort", "Lugar de nacimiento", "Luogo di nascita", "Geboorteplaats", "Local de nascimento"];
    /// Identity form error.
    IdentityInvalid => [
        "Indiquez au moins le nom et l’adresse postale (et, pour une entreprise, la personne qui agit pour elle).",
        "Give at least the name and the postal address (and, for a company, the person acting for it).",
        "Geben Sie mindestens Name und Postanschrift an (und bei einem Unternehmen die handelnde Person).",
        "Indica al menos el nombre y la dirección postal (y, para una empresa, la persona que actúa en su nombre).",
        "Indica almeno nome e indirizzo postale (e, per un’impresa, la persona che agisce per essa).",
        "Vul ten minste naam en postadres in (en, voor een bedrijf, wie namens het bedrijf handelt).",
        "Indique pelo menos o nome e a morada (e, para uma empresa, a pessoa que age em seu nome).",
    ];

    // ----- upload -----
    /// Upload limits.
    UploadLimits => ["EPUB ou PDF sans DRM, {max} au plus.", "EPUB or PDF without DRM, {max} at most.", "EPUB oder PDF ohne DRM, höchstens {max}.", "EPUB o PDF sin DRM, {max} como máximo.", "EPUB o PDF senza DRM, al massimo {max}.", "EPUB of pdf zonder DRM, hoogstens {max}.", "EPUB ou PDF sem DRM, {max} no máximo."];
    /// File field.
    FileLabel => ["Fichier", "File", "Datei", "Archivo", "File", "Bestand", "Ficheiro"];
    /// Title field.
    TitleLabel => ["Titre", "Title", "Titel", "Título", "Titolo", "Titel", "Título"];
    /// Author field.
    AuthorLabel => ["Autrice ou auteur", "Author", "Autor/in", "Autor o autora", "Autore o autrice", "Auteur", "Autor ou autora"];
    /// ISBN field.
    IsbnLabel => ["ISBN du livre numérique", "ISBN of the eBook", "ISBN des E-Books", "ISBN del libro electrónico", "ISBN del libro digitale", "ISBN van het e-book", "ISBN do livro digital"];
    /// Language field.
    LanguageLabel => ["Langue du livre (fr, en…)", "Language of the book (fr, en…)", "Sprache des Buches (fr, en…)", "Idioma del libro (fr, en…)", "Lingua del libro (fr, en…)", "Taal van het boek (fr, en…)", "Língua do livro (fr, en…)"];
    /// Basis legend.
    BasisLegend => ["Sur quelle base pouvez-vous diffuser ce fichier\u{a0}?", "On what basis may you distribute this file?", "Auf welcher Grundlage dürfen Sie diese Datei verbreiten?", "¿Con qué base puedes difundir este archivo?", "Su quale base puoi diffondere questo file?", "Op welke grond mag je dit bestand verspreiden?", "Com que base pode difundir este ficheiro?"];
    /// Own work.
    BasisOwnWork => [
        "J’en suis l’auteur ou l’éditeur et je détiens les droits numériques",
        "I am its author or publisher and hold the digital rights",
        "Ich bin Autor/in oder Verlag und halte die digitalen Rechte",
        "Soy su autor o editorial y tengo los derechos digitales",
        "Ne sono l’autore o l’editore e detengo i diritti digitali",
        "Ik ben de auteur of uitgever en heb de digitale rechten",
        "Sou o autor ou editor e detenho os direitos digitais",
    ];
    /// Public domain.
    BasisPublicDomain => ["L’œuvre est dans le domaine public en France", "The work is in the public domain in France", "Das Werk ist in Frankreich gemeinfrei", "La obra es de dominio público en Francia", "L’opera è di pubblico dominio in Francia", "Het werk is in Frankrijk publiek domein", "A obra está no domínio público em França"];
    /// Open licence.
    BasisOpenLicence => ["L’œuvre est sous licence libre", "The work is under an open licence", "Das Werk steht unter einer freien Lizenz", "La obra tiene una licencia libre", "L’opera è sotto licenza libera", "Het werk valt onder een open licentie", "A obra tem uma licença livre"];
    /// Licensed distributor.
    BasisLicensedDistributor => ["Je suis diffuseur autorisé par l’ayant droit", "I am a distributor licensed by the rights holder", "Ich bin vom Rechteinhaber autorisierter Vertrieb", "Soy distribuidor autorizado por el titular de los derechos", "Sono distributore autorizzato dal titolare dei diritti", "Ik ben distributeur met toestemming van de rechthebbende", "Sou distribuidor autorizado pelo titular dos direitos"];
    /// Public domain: death year.
    DeathYearLabel => [
        "Domaine public\u{a0}: année de décès de l’autrice ou de l’auteur (et du traducteur, s’il y en a un)",
        "Public domain: year the author died (and the translator, if any)",
        "Gemeinfrei: Todesjahr des Autors (und ggf. des Übersetzers)",
        "Dominio público: año de fallecimiento del autor (y del traductor, si lo hay)",
        "Pubblico dominio: anno di morte dell’autore (e del traduttore, se c’è)",
        "Publiek domein: sterfjaar van de auteur (en van de vertaler, als die er is)",
        "Domínio público: ano da morte do autor (e do tradutor, se houver)",
    ];
    /// Open licence choice.
    LicenceLabel => ["Licence libre", "Open licence", "Freie Lizenz", "Licencia libre", "Licenza libera", "Open licentie", "Licença livre"];
    /// Licensed distributor details.
    LicensorLabel => [
        "Diffuseur\u{a0}: ayant droit et référence du contrat",
        "Distributor: rights holder and contract reference",
        "Vertrieb: Rechteinhaber und Vertragsreferenz",
        "Distribuidor: titular de los derechos y referencia del contrato",
        "Distributore: titolare dei diritti e riferimento del contratto",
        "Distributeur: rechthebbende en contractreferentie",
        "Distribuidor: titular dos direitos e referência do contrato",
    ];
    /// The declaration (version: see `routes::books::DECLARATION_VERSION`).
    DeclarationText => [
        "Je déclare sur l’honneur détenir les droits nécessaires pour héberger ce fichier et le remettre à toute personne qui détient une carte cadeau que je crée. Ce n’est pas un livre acheté en librairie. J’accorde à l’hébergeur une licence limitée à cet hébergement et à cette remise. Je sais qu’une fausse déclaration entraîne le retrait du livre et la suspension de mon compte.",
        "I declare on my honour that I hold the rights needed to host this file and deliver it to anyone holding a gift card I create. It is not a book bought from a store. I grant the host a licence limited to this hosting and delivery. I know that a false declaration leads to the book’s removal and my account’s suspension.",
        "Ich erkläre, die nötigen Rechte zu besitzen, um diese Datei hosten und an jede Person ausliefern zu lassen, die eine von mir erstellte Geschenkkarte besitzt. Es ist kein im Handel gekauftes Buch. Ich räume dem Hoster eine auf dieses Hosting und diese Auslieferung beschränkte Lizenz ein. Mir ist bekannt, dass eine falsche Erklärung zur Entfernung des Buches und zur Sperrung meines Kontos führt.",
        "Declaro bajo mi responsabilidad que tengo los derechos necesarios para alojar este archivo y entregarlo a quien tenga una tarjeta regalo que yo cree. No es un libro comprado en una librería. Concedo al alojador una licencia limitada a este alojamiento y entrega. Sé que una declaración falsa conlleva la retirada del libro y la suspensión de mi cuenta.",
        "Dichiaro sul mio onore di detenere i diritti necessari per ospitare questo file e consegnarlo a chiunque possieda un biglietto regalo da me creato. Non è un libro acquistato in libreria. Concedo all’hosting una licenza limitata a questo hosting e a questa consegna. So che una dichiarazione falsa comporta la rimozione del libro e la sospensione del mio account.",
        "Ik verklaar op erewoord dat ik de rechten heb om dit bestand te laten bewaren en te laten bezorgen aan iedereen die een cadeaukaart heeft die ik maak. Het is geen gekocht boek. Ik geef de hostingdienst een licentie die beperkt is tot dit bewaren en bezorgen. Ik weet dat een valse verklaring leidt tot verwijdering van het boek en schorsing van mijn account.",
        "Declaro sob compromisso de honra que detenho os direitos necessários para alojar este ficheiro e entregá-lo a quem tiver um cartão de oferta criado por mim. Não é um livro comprado numa livraria. Concedo ao alojamento uma licença limitada a este alojamento e entrega. Sei que uma declaração falsa leva à remoção do livro e à suspensão da minha conta.",
    ];
    /// Upload button.
    UploadButton => ["Déposer", "Upload", "Hochladen", "Subir", "Carica", "Uploaden", "Carregar"];
    /// No file.
    ErrNoFile => ["Choisissez un fichier.", "Choose a file.", "Wählen Sie eine Datei.", "Elige un archivo.", "Scegli un file.", "Kies een bestand.", "Escolha um ficheiro."];
    /// File too large.
    ErrTooLarge => ["Le fichier dépasse {max}.", "The file is larger than {max}.", "Die Datei ist größer als {max}.", "El archivo supera {max}.", "Il file supera {max}.", "Het bestand is groter dan {max}.", "O ficheiro excede {max}."];
    /// Not EPUB/PDF.
    ErrType => ["Seuls les fichiers EPUB et PDF sont acceptés.", "Only EPUB and PDF files are accepted.", "Nur EPUB- und PDF-Dateien werden angenommen.", "Solo se aceptan archivos EPUB y PDF.", "Sono accettati solo file EPUB e PDF.", "Alleen EPUB- en pdf-bestanden worden aanvaard.", "Só são aceites ficheiros EPUB e PDF."];
    /// Damaged EPUB.
    ErrDamaged => ["Ce fichier EPUB est endommagé.", "This EPUB file is damaged.", "Diese EPUB-Datei ist beschädigt.", "Este archivo EPUB está dañado.", "Questo file EPUB è danneggiato.", "Dit EPUB-bestand is beschadigd.", "Este ficheiro EPUB está danificado."];
    /// DRM refused.
    ErrDrm => [
        "Ce fichier est protégé par DRM\u{a0}: il ne peut pas être hébergé. Un livre acheté s’offre avec l’option cadeau de la librairie.",
        "This file is protected by DRM: it cannot be hosted. A bought book is given with the store’s gift option.",
        "Diese Datei ist DRM-geschützt und kann nicht gehostet werden. Ein gekauftes Buch verschenkt man über die Geschenkoption des Shops.",
        "Este archivo está protegido con DRM: no puede alojarse. Un libro comprado se regala con la opción de regalo de la librería.",
        "Questo file è protetto da DRM: non può essere ospitato. Un libro acquistato si regala con l’opzione regalo della libreria.",
        "Dit bestand is met DRM beveiligd en kan niet worden bewaard. Een gekocht boek geef je met de cadeauoptie van de winkel.",
        "Este ficheiro está protegido por DRM: não pode ser alojado. Um livro comprado oferece-se com a opção de oferta da livraria.",
    ];
    /// Declaration missing.
    ErrDeclaration => ["Choisissez une base de droits et cochez la déclaration.", "Choose a rights basis and tick the declaration.", "Wählen Sie eine Rechtegrundlage und bestätigen Sie die Erklärung.", "Elige una base de derechos y marca la declaración.", "Scegli una base dei diritti e spunta la dichiarazione.", "Kies een rechtsgrond en vink de verklaring aan.", "Escolha uma base de direitos e assinale a declaração."];
    /// Title/author missing.
    ErrTitle => ["Indiquez le titre et l’autrice ou l’auteur.", "Give the title and the author.", "Geben Sie Titel und Autor/in an.", "Indica el título y el autor.", "Indica titolo e autore.", "Vul titel en auteur in.", "Indique o título e o autor."];
    /// Bad ISBN.
    ErrIsbn => ["Cet ISBN n’est pas valable.", "This ISBN is not valid.", "Diese ISBN ist ungültig.", "Este ISBN no es válido.", "Questo ISBN non è valido.", "Dit ISBN is ongeldig.", "Este ISBN não é válido."];
    /// Blocked hash.
    ErrBlockedFile => [
        "Ce fichier a été retiré après un signalement\u{a0}: il ne peut pas être déposé à nouveau.",
        "This file was removed after a notice: it cannot be uploaded again.",
        "Diese Datei wurde nach einer Meldung entfernt und kann nicht erneut hochgeladen werden.",
        "Este archivo fue retirado tras una denuncia: no puede volver a subirse.",
        "Questo file è stato rimosso dopo una segnalazione: non può essere ricaricato.",
        "Dit bestand is na een melding verwijderd en kan niet opnieuw worden geüpload.",
        "Este ficheiro foi removido após uma denúncia: não pode ser carregado de novo.",
    ];
    /// Blocked ISBN.
    ErrBlockedIsbn => [
        "Ce livre (ISBN {isbn}) a été retiré après un signalement\u{a0}: il ne peut pas être déposé à nouveau.",
        "This book (ISBN {isbn}) was removed after a notice: it cannot be uploaded again.",
        "Dieses Buch (ISBN {isbn}) wurde nach einer Meldung entfernt und kann nicht erneut hochgeladen werden.",
        "Este libro (ISBN {isbn}) fue retirado tras una denuncia: no puede volver a subirse.",
        "Questo libro (ISBN {isbn}) è stato rimosso dopo una segnalazione: non può essere ricaricato.",
        "Dit boek (ISBN {isbn}) is na een melding verwijderd en kan niet opnieuw worden geüpload.",
        "Este livro (ISBN {isbn}) foi removido após uma denúncia: não pode ser carregado de novo.",
    ];
    /// Account quota.
    ErrQuotaAccount => ["Votre espace est plein ({quota}).", "Your storage is full ({quota}).", "Ihr Speicher ist voll ({quota}).", "Tu espacio está lleno ({quota}).", "Il tuo spazio è pieno ({quota}).", "Je opslag is vol ({quota}).", "O seu espaço está cheio ({quota})."];
    /// Global quota.
    ErrQuotaTotal => ["Le rayonnage est plein. Réessayez plus tard.", "The shelf is full. Please try again later.", "Das Regal ist voll. Bitte später erneut versuchen.", "La estantería está llena. Inténtalo más tarde.", "Lo scaffale è pieno. Riprova più tardi.", "De plank is vol. Probeer het later opnieuw.", "A estante está cheia. Tente mais tarde."];
    /// Public domain not proven.
    ErrNotPublicDomain => [
        "En France, une œuvre entre dans le domaine public le 1er janvier qui suit le 70e anniversaire de la mort de son auteur. Indiquez une année de décès antérieure à {year}.",
        "In France a work enters the public domain on the 1 January after the 70th anniversary of its author’s death. Give a year of death before {year}.",
        "In Frankreich wird ein Werk am 1. Januar nach dem 70. Todestag seines Autors gemeinfrei. Geben Sie ein Todesjahr vor {year} an.",
        "En Francia, una obra pasa al dominio público el 1 de enero siguiente al 70.º aniversario de la muerte de su autor. Indica un año de fallecimiento anterior a {year}.",
        "In Francia un’opera entra nel pubblico dominio il 1° gennaio successivo al 70° anniversario della morte dell’autore. Indica un anno di morte anteriore al {year}.",
        "In Frankrijk wordt een werk publiek domein op 1 januari na de 70e verjaardag van het overlijden van de auteur. Geef een sterfjaar vóór {year}.",
        "Em França, uma obra entra no domínio público a 1 de janeiro seguinte ao 70.º aniversário da morte do autor. Indique um ano de morte anterior a {year}.",
    ];
    /// Licence missing.
    ErrLicence => ["Choisissez la licence.", "Choose the licence.", "Wählen Sie die Lizenz.", "Elige la licencia.", "Scegli la licenza.", "Kies de licentie.", "Escolha a licença."];
    /// Licensor missing.
    ErrLicensor => ["Indiquez l’ayant droit et la référence du contrat.", "Give the rights holder and the contract reference.", "Geben Sie Rechteinhaber und Vertragsreferenz an.", "Indica el titular de los derechos y la referencia del contrato.", "Indica il titolare dei diritti e il riferimento del contratto.", "Vul de rechthebbende en de contractreferentie in.", "Indique o titular dos direitos e a referência do contrato."];
    /// Too many concurrent uploads.
    ErrBusy => ["Trop de dépôts en cours. Réessayez dans un instant.", "Too many uploads in progress. Try again in a moment.", "Zu viele laufende Uploads. Bitte gleich erneut versuchen.", "Demasiadas subidas en curso. Inténtalo en un momento.", "Troppi caricamenti in corso. Riprova tra un momento.", "Te veel uploads tegelijk. Probeer het zo opnieuw.", "Demasiados carregamentos em curso. Tente daqui a pouco."];
    /// Cannot upload (suspended / identity).
    ErrCannotUpload => ["Votre compte ne peut pas déposer de livre pour l’instant.", "Your account cannot upload books at the moment.", "Ihr Konto kann derzeit keine Bücher hochladen.", "Tu cuenta no puede subir libros por ahora.", "Il tuo account al momento non può caricare libri.", "Je account kan op dit moment geen boeken uploaden.", "A sua conta não pode carregar livros de momento."];

    // ----- book page -----
    /// Book file line.
    BookFile => ["Fichier\u{a0}: {name}, {size}, SHA-256 {hash}", "File: {name}, {size}, SHA-256 {hash}", "Datei: {name}, {size}, SHA-256 {hash}", "Archivo: {name}, {size}, SHA-256 {hash}", "File: {name}, {size}, SHA-256 {hash}", "Bestand: {name}, {size}, SHA-256 {hash}", "Ficheiro: {name}, {size}, SHA-256 {hash}"];
    /// Declaration line.
    BookDeclared => ["Déclaration du {date}\u{a0}: {basis}", "Declaration of {date}: {basis}", "Erklärung vom {date}: {basis}", "Declaración del {date}: {basis}", "Dichiarazione del {date}: {basis}", "Verklaring van {date}: {basis}", "Declaração de {date}: {basis}"];
    /// Removed book.
    BookRemoved => ["Ce livre n’est plus hébergé\u{a0}: {reason}.", "This book is no longer hosted: {reason}.", "Dieses Buch wird nicht mehr gehostet: {reason}.", "Este libro ya no está alojado: {reason}.", "Questo libro non è più ospitato: {reason}.", "Dit boek wordt niet meer bewaard: {reason}.", "Este livro já não está alojado: {reason}."];
    /// Removal reason: deleted.
    RemovedDeleted => ["supprimé par son ayant droit", "deleted by its rights holder", "vom Rechteinhaber gelöscht", "borrado por su titular", "cancellato dal titolare", "verwijderd door de rechthebbende", "apagado pelo titular"];
    /// Removal reason: takedown.
    RemovedTakenDown => ["retiré après un signalement", "removed after a notice", "nach einer Meldung entfernt", "retirado tras una denuncia", "rimosso dopo una segnalazione", "verwijderd na een melding", "removido após uma denúncia"];
    /// Removal reason: expired.
    RemovedExpired => ["plus aucun cadeau actif", "no active gift any more", "kein aktives Geschenk mehr", "ya no hay regalos activos", "nessun regalo più attivo", "geen actief cadeau meer", "já não há ofertas ativas"];
    /// Removal reason: account closed.
    RemovedClosed => ["compte fermé", "account closed", "Konto geschlossen", "cuenta cerrada", "account chiuso", "account gesloten", "conta encerrada"];
    /// Gifts heading.
    Gifts => ["Cadeaux", "Gifts", "Geschenke", "Regalos", "Regali", "Cadeaus", "Ofertas"];
    /// No gift.
    NoGifts => ["Aucun cadeau pour l’instant.", "No gifts yet.", "Noch keine Geschenke.", "Todavía no hay regalos.", "Ancora nessun regalo.", "Nog geen cadeaus.", "Ainda não há ofertas."];
    /// New gifts heading.
    NewGifts => ["Créer des cadeaux", "Create gifts", "Geschenke anlegen", "Crear regalos", "Crea regali", "Cadeaus maken", "Criar ofertas"];
    /// Count field.
    CountLabel => ["Nombre de cartes (1 à {max})", "Number of cards (1 to {max})", "Anzahl Karten (1 bis {max})", "Número de tarjetas (1 a {max})", "Numero di biglietti (da 1 a {max})", "Aantal kaarten (1 tot {max})", "Número de cartões (1 a {max})"];
    /// Giver field.
    GiverLabel => ["De la part de", "From", "Von", "De parte de", "Da parte di", "Van", "Da parte de"];
    /// Recipient field.
    RecipientLabel => ["Pour", "To", "Für", "Para", "Per", "Voor", "Para"];
    /// Message field.
    MessageLabel => ["Message", "Message", "Nachricht", "Mensaje", "Messaggio", "Bericht", "Mensagem"];
    /// Card language.
    CardLanguageLabel => ["Langue de la carte", "Language of the card", "Sprache der Karte", "Idioma de la tarjeta", "Lingua del biglietto", "Taal van de kaart", "Língua do cartão"];
    /// Reveal date.
    RevealLabel => ["À ouvrir à partir du (AAAA-MM-JJ)", "To be opened from (YYYY-MM-DD)", "Zu öffnen ab (JJJJ-MM-TT)", "Para abrir a partir del (AAAA-MM-DD)", "Da aprire dal (AAAA-MM-GG)", "Te openen vanaf (JJJJ-MM-DD)", "Para abrir a partir de (AAAA-MM-DD)"];
    /// Create button.
    CreateGifts => ["Créer", "Create", "Anlegen", "Crear", "Crea", "Maken", "Criar"];
    /// Code label.
    CodeLabel => ["Code", "Code", "Code", "Código", "Codice", "Code", "Código"];
    /// Gift live.
    GiftLive => ["actif", "active", "aktiv", "activo", "attivo", "actief", "ativo"];
    /// Gift pending.
    GiftPending => ["en attente du service eBook Cadeau", "waiting for the eBook Cadeau service", "wartet auf den eBook-Cadeau-Dienst", "a la espera del servicio eBook Cadeau", "in attesa del servizio eBook Cadeau", "wacht op de eBook Cadeau-dienst", "à espera do serviço eBook Cadeau"];
    /// Gift cancelled.
    GiftCancelled => ["annulé", "cancelled", "storniert", "anulado", "annullato", "geannuleerd", "anulado"];
    /// Deliveries.
    Deliveries => ["remis {n} fois", "delivered {n} times", "{n}-mal ausgeliefert", "entregado {n} veces", "consegnato {n} volte", "{n} keer bezorgd", "entregue {n} vezes"];
    /// Card for home printing.
    CardHome => ["Carte à imprimer (A4)", "Card to print (A4)", "Karte zum Drucken (A4)", "Tarjeta para imprimir (A4)", "Biglietto da stampare (A4)", "Kaart om te printen (A4)", "Cartão para imprimir (A4)"];
    /// Card for printers.
    CardPro => ["Carte pour imprimeur", "Card for a printer", "Karte für die Druckerei", "Tarjeta para imprenta", "Biglietto per tipografia", "Kaart voor de drukker", "Cartão para gráfica"];
    /// Designer link.
    Designer => ["Personnaliser dans le créateur de cartes", "Customise in the card designer", "Im Kartengestalter anpassen", "Personalizar en el creador de tarjetas", "Personalizza nel creatore di biglietti", "Aanpassen in de kaartontwerper", "Personalizar no criador de cartões"];
    /// Reissue button.
    Reissue => ["Nouveau code (carte perdue)", "New code (lost card)", "Neuer Code (Karte verloren)", "Nuevo código (tarjeta perdida)", "Nuovo codice (biglietto perso)", "Nieuwe code (kaart kwijt)", "Novo código (cartão perdido)"];
    /// Cancel button.
    CancelGift => ["Annuler ce cadeau", "Cancel this gift", "Dieses Geschenk stornieren", "Anular este regalo", "Annulla questo regalo", "Dit cadeau annuleren", "Anular esta oferta"];
    /// Retry button.
    Retry => ["Réessayer", "Try again", "Erneut versuchen", "Reintentar", "Riprova", "Opnieuw proberen", "Tentar novamente"];
    /// Delete book.
    DeleteBook => ["Supprimer ce livre", "Delete this book", "Dieses Buch löschen", "Borrar este libro", "Cancella questo libro", "Dit boek verwijderen", "Apagar este livro"];
    /// Delete book explanation.
    DeleteBookHelp => [
        "Le fichier est effacé et tous ses cadeaux sont annulés\u{a0}: leurs codes ne fonctionneront plus.",
        "The file is erased and all its gifts are cancelled: their codes will stop working.",
        "Die Datei wird gelöscht und alle Geschenke storniert: Die Codes funktionieren dann nicht mehr.",
        "El archivo se borra y todos sus regalos se anulan: sus códigos dejarán de funcionar.",
        "Il file viene cancellato e tutti i suoi regali annullati: i codici smetteranno di funzionare.",
        "Het bestand wordt gewist en alle cadeaus worden geannuleerd: de codes werken dan niet meer.",
        "O ficheiro é apagado e todas as ofertas anuladas: os códigos deixam de funcionar.",
    ];
    /// Gift form error.
    ErrGiftForm => [
        "Vérifiez le nombre de cartes (1 à {max}), la langue et la date (AAAA-MM-JJ). Les textes ont des longueurs limitées.",
        "Check the number of cards (1 to {max}), the language and the date (YYYY-MM-DD). Texts have limited lengths.",
        "Prüfen Sie Anzahl der Karten (1 bis {max}), Sprache und Datum (JJJJ-MM-TT). Die Texte sind in der Länge begrenzt.",
        "Revisa el número de tarjetas (1 a {max}), el idioma y la fecha (AAAA-MM-DD). Los textos tienen una longitud limitada.",
        "Controlla il numero di biglietti (da 1 a {max}), la lingua e la data (AAAA-MM-GG). I testi hanno una lunghezza limitata.",
        "Controleer het aantal kaarten (1 tot {max}), de taal en de datum (JJJJ-MM-DD). Teksten hebben een beperkte lengte.",
        "Verifique o número de cartões (1 a {max}), a língua e a data (AAAA-MM-DD). Os textos têm um comprimento limitado.",
    ];
    /// Core failure.
    ErrCore => [
        "Le service eBook Cadeau n’a pas répondu ({detail}). Ce qui n’a pas abouti reste en attente\u{a0}: réessayez plus tard.",
        "The eBook Cadeau service did not answer ({detail}). What did not go through stays pending: try again later.",
        "Der eBook-Cadeau-Dienst hat nicht geantwortet ({detail}). Was nicht durchging, bleibt offen: Bitte später erneut versuchen.",
        "El servicio eBook Cadeau no ha respondido ({detail}). Lo que no se completó queda pendiente: inténtalo más tarde.",
        "Il servizio eBook Cadeau non ha risposto ({detail}). Ciò che non è andato a buon fine resta in sospeso: riprova più tardi.",
        "De eBook Cadeau-dienst antwoordde niet ({detail}). Wat niet lukte, blijft openstaan: probeer het later opnieuw.",
        "O serviço eBook Cadeau não respondeu ({detail}). O que não foi concluído fica pendente: tente mais tarde.",
    ];

    // ----- notices -----
    /// Notice form title.
    NoticeTitle => ["Signaler un contenu illicite", "Report illegal content", "Rechtswidrige Inhalte melden", "Denunciar un contenido ilícito", "Segnala un contenuto illecito", "Illegale inhoud melden", "Denunciar um conteúdo ilícito"];
    /// Notice form explanation.
    NoticeIntro => [
        "Ce formulaire suit l’article 16 du règlement européen sur les services numériques (DSA). Aucun compte n’est nécessaire. Nous accusons réception et décidons avec diligence, en principe sous 24 heures.",
        "This form follows Article 16 of the EU Digital Services Act (DSA). No account is needed. We acknowledge receipt and decide diligently, normally within 24 hours.",
        "Dieses Formular folgt Artikel 16 des EU-Gesetzes über digitale Dienste (DSA). Ein Konto ist nicht nötig. Wir bestätigen den Eingang und entscheiden sorgfältig, in der Regel binnen 24 Stunden.",
        "Este formulario sigue el artículo 16 del Reglamento de Servicios Digitales de la UE (DSA). No hace falta cuenta. Acusamos recibo y decidimos con diligencia, en principio en 24 horas.",
        "Questo modulo segue l’articolo 16 del regolamento UE sui servizi digitali (DSA). Non serve un account. Confermiamo la ricezione e decidiamo con diligenza, di norma entro 24 ore.",
        "Dit formulier volgt artikel 16 van de Europese digitaledienstenverordening (DSA). Een account is niet nodig. We bevestigen de ontvangst en beslissen zorgvuldig, in principe binnen 24 uur.",
        "Este formulário segue o artigo 16 do Regulamento dos Serviços Digitais da UE (DSA). Não é precisa conta. Acusamos a receção e decidimos com diligência, em princípio em 24 horas.",
    ];
    /// Category legend.
    CategoryLegend => ["Nature du contenu", "Kind of content", "Art des Inhalts", "Tipo de contenido", "Tipo di contenuto", "Soort inhoud", "Tipo de conteúdo"];
    /// Copyright.
    CategoryCopyright => ["atteinte au droit d’auteur ou aux droits voisins", "infringement of copyright or related rights", "Verletzung des Urheberrechts oder verwandter Schutzrechte", "infracción de los derechos de autor o afines", "violazione del diritto d’autore o dei diritti connessi", "inbreuk op het auteursrecht of naburige rechten", "violação dos direitos de autor ou conexos"];
    /// Other illegal content.
    CategoryOther => ["autre contenu illicite", "other illegal content", "sonstiger rechtswidriger Inhalt", "otro contenido ilícito", "altro contenuto illecito", "andere illegale inhoud", "outro conteúdo ilícito"];
    /// Location field.
    LocationLabel => [
        "Où se trouve le contenu\u{a0}? Adresse exacte (URL), identifiant du livre ou du cadeau, ou code imprimé sur une carte",
        "Where is the content? Exact address (URL), book or gift identifier, or the code printed on a card",
        "Wo befindet sich der Inhalt? Genaue Adresse (URL), Buch- oder Geschenkkennung oder der auf einer Karte gedruckte Code",
        "¿Dónde está el contenido? Dirección exacta (URL), identificador del libro o del regalo, o código impreso en una tarjeta",
        "Dove si trova il contenuto? Indirizzo esatto (URL), identificativo del libro o del regalo, o codice stampato su un biglietto",
        "Waar staat de inhoud? Exact adres (URL), kenmerk van het boek of cadeau, of de code op een kaart",
        "Onde está o conteúdo? Endereço exato (URL), identificador do livro ou da oferta, ou código impresso num cartão",
    ];
    /// Explanation field.
    ExplanationLabel => ["Pourquoi ce contenu est-il illicite\u{a0}? Donnez des motifs précis", "Why is this content illegal? Give precise reasons", "Warum ist dieser Inhalt rechtswidrig? Bitte genau begründen", "¿Por qué es ilícito este contenido? Da motivos precisos", "Perché questo contenuto è illecito? Indica motivi precisi", "Waarom is deze inhoud illegaal? Geef precieze redenen", "Porque é ilícito este conteúdo? Indique motivos precisos"];
    /// Notifier's name.
    NotifierNameLabel => ["Votre nom", "Your name", "Ihr Name", "Tu nombre", "Il tuo nome", "Je naam", "O seu nome"];
    /// Exception of art. 16(2)(c).
    ExceptionLabel => [
        "Mon signalement concerne une infraction visée aux articles 3 à 7 de la directive 2011/93/UE (abus sexuels sur enfants)\u{a0}: nom et e-mail facultatifs",
        "My notice concerns an offence under Articles 3 to 7 of Directive 2011/93/EU (child sexual abuse): name and e-mail optional",
        "Meine Meldung betrifft eine Straftat nach Art. 3 bis 7 der Richtlinie 2011/93/EU (sexueller Missbrauch von Kindern): Name und E-Mail freiwillig",
        "Mi denuncia se refiere a un delito de los artículos 3 a 7 de la Directiva 2011/93/UE (abusos sexuales a menores): nombre y correo opcionales",
        "La mia segnalazione riguarda un reato di cui agli articoli da 3 a 7 della direttiva 2011/93/UE (abusi sessuali su minori): nome ed e-mail facoltativi",
        "Mijn melding betreft een strafbaar feit uit artikel 3 tot 7 van Richtlijn 2011/93/EU (seksueel misbruik van kinderen): naam en e-mail optioneel",
        "A minha denúncia diz respeito a uma infração dos artigos 3.º a 7.º da Diretiva 2011/93/UE (abuso sexual de crianças): nome e e-mail facultativos",
    ];
    /// Good-faith statement.
    GoodFaithLabel => [
        "Je déclare de bonne foi que les informations et allégations de ce signalement sont exactes et complètes.",
        "I declare in good faith that the information and allegations in this notice are accurate and complete.",
        "Ich erkläre in gutem Glauben, dass die Angaben und Behauptungen dieser Meldung richtig und vollständig sind.",
        "Declaro de buena fe que la información y las alegaciones de esta denuncia son exactas y completas.",
        "Dichiaro in buona fede che le informazioni e le affermazioni di questa segnalazione sono esatte e complete.",
        "Ik verklaar te goeder trouw dat de gegevens en beweringen in deze melding juist en volledig zijn.",
        "Declaro de boa-fé que as informações e alegações desta denúncia são exatas e completas.",
    ];
    /// Send button.
    SendNotice => ["Envoyer le signalement", "Send the notice", "Meldung senden", "Enviar la denuncia", "Invia la segnalazione", "Melding versturen", "Enviar a denúncia"];
    /// Notice form error.
    NoticeErr => [
        "Remplissez la localisation et l’explication, votre nom et votre e-mail (sauf l’exception ci-dessous), et cochez la déclaration de bonne foi.",
        "Fill in the location and the explanation, your name and e-mail (unless the exception below applies), and tick the good-faith statement.",
        "Füllen Sie Ort und Begründung, Ihren Namen und Ihre E-Mail (außer bei der Ausnahme unten) aus und bestätigen Sie die Erklärung in gutem Glauben.",
        "Rellena la ubicación y la explicación, tu nombre y correo (salvo la excepción de abajo) y marca la declaración de buena fe.",
        "Compila posizione e spiegazione, nome ed e-mail (salvo l’eccezione qui sotto) e spunta la dichiarazione di buona fede.",
        "Vul de plaats en de uitleg in, je naam en e-mail (behalve bij de uitzondering hieronder), en vink de verklaring te goeder trouw aan.",
        "Preencha a localização e a explicação, o seu nome e e-mail (salvo a exceção abaixo) e assinale a declaração de boa-fé.",
    ];
    /// Notice received title.
    NoticeReceivedTitle => ["Signalement reçu", "Notice received", "Meldung erhalten", "Denuncia recibida", "Segnalazione ricevuta", "Melding ontvangen", "Denúncia recebida"];
    /// Notice received.
    NoticeReceived => [
        "Votre signalement est enregistré sous la référence {id}. Un accusé de réception vous est envoyé\u{a0}; nous vous écrirons quand une décision sera prise.",
        "Your notice is recorded under reference {id}. An acknowledgement is on its way; we will write to you when a decision is taken.",
        "Ihre Meldung ist unter der Referenz {id} erfasst. Eine Eingangsbestätigung ist unterwegs; wir schreiben Ihnen, sobald entschieden ist.",
        "Tu denuncia está registrada con la referencia {id}. Te enviamos un acuse de recibo; te escribiremos cuando se tome una decisión.",
        "La tua segnalazione è registrata con il riferimento {id}. Ti inviamo una conferma di ricezione; ti scriveremo quando sarà presa una decisione.",
        "Je melding is geregistreerd onder referentie {id}. Er komt een ontvangstbevestiging; we schrijven je zodra er een beslissing is.",
        "A sua denúncia foi registada com a referência {id}. Enviamos-lhe um aviso de receção; escreveremos quando houver uma decisão.",
    ];
    /// Acknowledgement subject.
    MailNoticeAckSubject => ["Signalement {id} reçu — {shelf}", "Notice {id} received — {shelf}", "Meldung {id} erhalten — {shelf}", "Denuncia {id} recibida — {shelf}", "Segnalazione {id} ricevuta — {shelf}", "Melding {id} ontvangen — {shelf}", "Denúncia {id} recebida — {shelf}"];
    /// Acknowledgement body.
    MailNoticeAckBody => [
        "Bonjour,\n\nNous avons bien reçu votre signalement {id} du {date} concernant\u{a0}:\n{location}\n\nNous l’examinons et vous informerons de notre décision et des voies de recours.\n",
        "Hello,\n\nWe have received your notice {id} of {date} about:\n{location}\n\nWe are examining it and will tell you our decision and how to contest it.\n",
        "Guten Tag,\n\nwir haben Ihre Meldung {id} vom {date} erhalten zu:\n{location}\n\nWir prüfen sie und teilen Ihnen unsere Entscheidung und die Rechtsbehelfe mit.\n",
        "Hola:\n\nHemos recibido tu denuncia {id} del {date} sobre:\n{location}\n\nLa estamos examinando y te informaremos de nuestra decisión y de las vías de recurso.\n",
        "Buongiorno,\n\nabbiamo ricevuto la tua segnalazione {id} del {date} relativa a:\n{location}\n\nLa stiamo esaminando e ti comunicheremo la decisione e i mezzi di ricorso.\n",
        "Hallo,\n\nWe hebben je melding {id} van {date} ontvangen over:\n{location}\n\nWe bekijken haar en laten je onze beslissing en de bezwaarmogelijkheden weten.\n",
        "Olá,\n\nRecebemos a sua denúncia {id} de {date} sobre:\n{location}\n\nEstamos a analisá-la e informaremos da nossa decisão e das vias de recurso.\n",
    ];
    /// Decision subject (to the notifier).
    MailNoticeDecisionSubject => ["Décision sur le signalement {id} — {shelf}", "Decision on notice {id} — {shelf}", "Entscheidung zur Meldung {id} — {shelf}", "Decisión sobre la denuncia {id} — {shelf}", "Decisione sulla segnalazione {id} — {shelf}", "Beslissing over melding {id} — {shelf}", "Decisão sobre a denúncia {id} — {shelf}"];
    /// Upheld (to the notifier).
    MailNoticeUpheld => [
        "Bonjour,\n\nÀ la suite de votre signalement {id}, le contenu a été retiré\u{a0}: le fichier est effacé et les codes de ses cartes ne fonctionnent plus. Il ne pourra pas être déposé à nouveau.\n",
        "Hello,\n\nFollowing your notice {id}, the content was removed: the file is erased and the codes of its cards no longer work. It cannot be uploaded again.\n",
        "Guten Tag,\n\naufgrund Ihrer Meldung {id} wurde der Inhalt entfernt: Die Datei ist gelöscht, die Codes ihrer Karten funktionieren nicht mehr. Sie kann nicht erneut hochgeladen werden.\n",
        "Hola:\n\nTras tu denuncia {id}, el contenido se ha retirado: el archivo está borrado y los códigos de sus tarjetas ya no funcionan. No podrá volver a subirse.\n",
        "Buongiorno,\n\na seguito della tua segnalazione {id}, il contenuto è stato rimosso: il file è cancellato e i codici dei biglietti non funzionano più. Non potrà essere ricaricato.\n",
        "Hallo,\n\nNa je melding {id} is de inhoud verwijderd: het bestand is gewist en de codes van de kaarten werken niet meer. Het kan niet opnieuw worden geüpload.\n",
        "Olá,\n\nNa sequência da sua denúncia {id}, o conteúdo foi removido: o ficheiro foi apagado e os códigos dos cartões deixaram de funcionar. Não poderá ser carregado de novo.\n",
    ];
    /// Rejected (to the notifier).
    MailNoticeRejected => [
        "Bonjour,\n\nNous n’avons pas retiré le contenu visé par votre signalement {id}.\nMotifs\u{a0}: {reason}\n",
        "Hello,\n\nWe did not remove the content of your notice {id}.\nReasons: {reason}\n",
        "Guten Tag,\n\nwir haben den Inhalt Ihrer Meldung {id} nicht entfernt.\nGründe: {reason}\n",
        "Hola:\n\nNo hemos retirado el contenido de tu denuncia {id}.\nMotivos: {reason}\n",
        "Buongiorno,\n\nnon abbiamo rimosso il contenuto della tua segnalazione {id}.\nMotivi: {reason}\n",
        "Hallo,\n\nWe hebben de inhoud van je melding {id} niet verwijderd.\nRedenen: {reason}\n",
        "Olá,\n\nNão removemos o conteúdo da sua denúncia {id}.\nMotivos: {reason}\n",
    ];
    /// Redress for the notifier.
    MailNotifierRedress => [
        "\nVous pouvez contester cette décision en répondant à ce message, vous adresser à un organisme de règlement extrajudiciaire des litiges certifié (DSA, art. 21) ou saisir le juge.\n",
        "\nYou may contest this decision by replying to this message, turn to a certified out-of-court dispute settlement body (DSA Art. 21) or go to court.\n",
        "\nSie können diese Entscheidung anfechten, indem Sie auf diese Nachricht antworten, sich an eine zertifizierte außergerichtliche Streitbeilegungsstelle wenden (DSA Art. 21) oder ein Gericht anrufen.\n",
        "\nPuedes impugnar esta decisión respondiendo a este mensaje, acudir a un órgano certificado de resolución extrajudicial de litigios (DSA, art. 21) o a los tribunales.\n",
        "\nPuoi contestare questa decisione rispondendo a questo messaggio, rivolgerti a un organismo certificato di risoluzione extragiudiziale delle controversie (DSA, art. 21) o al giudice.\n",
        "\nJe kunt deze beslissing aanvechten door op dit bericht te antwoorden, een erkende instantie voor buitengerechtelijke geschilbeslechting inschakelen (DSA art. 21) of naar de rechter stappen.\n",
        "\nPode contestar esta decisão respondendo a esta mensagem, recorrer a um organismo certificado de resolução extrajudicial de litígios (DSA, art. 21) ou ao tribunal.\n",
    ];
    /// Statement of reasons subject.
    MailSorSubject => ["Décision concernant votre livre « {title} » — {shelf}", "Decision about your book “{title}” — {shelf}", "Entscheidung zu Ihrem Buch „{title}“ — {shelf}", "Decisión sobre tu libro «{title}» — {shelf}", "Decisione sul tuo libro «{title}» — {shelf}", "Beslissing over je boek ‘{title}’ — {shelf}", "Decisão sobre o seu livro «{title}» — {shelf}"];
    /// Statement of reasons body (DSA art. 17).
    MailSorBody => [
        "Bonjour,\n\nExposé des motifs (règlement sur les services numériques, art. 17).\n\nDécision\u{a0}: votre livre « {title} » ({book}) est retiré de {shelf} le {date}. Le fichier est effacé, les codes de ses cartes ne fonctionnent plus, et son empreinte SHA-256 et son ISBN ne peuvent plus être déposés.\nFaits et circonstances\u{a0}: signalement {notice} reçu le {received}. {statement}\nFondement\u{a0}: {ground}\nMoyens automatisés\u{a0}: aucun\u{a0}; la décision a été prise par une personne à la suite d’un signalement.\n{false}\nVoies de recours\u{a0}:\n- une réclamation auprès de nous, dans les 6 mois, depuis votre compte\u{a0}: {link}\n- un organisme de règlement extrajudiciaire des litiges certifié (art. 21)\u{a0};\n- le juge.\n",
        "Hello,\n\nStatement of reasons (Digital Services Act, Art. 17).\n\nDecision: your book “{title}” ({book}) is removed from {shelf} on {date}. The file is erased, the codes of its cards no longer work, and its SHA-256 fingerprint and ISBN can no longer be uploaded.\nFacts and circumstances: notice {notice} received on {received}. {statement}\nGround: {ground}\nAutomated means: none; the decision was taken by a person following a notice.\n{false}\nRedress:\n- a complaint to us, within 6 months, from your account: {link}\n- a certified out-of-court dispute settlement body (Art. 21);\n- the courts.\n",
        "Guten Tag,\n\nBegründung (Gesetz über digitale Dienste, Art. 17).\n\nEntscheidung: Ihr Buch „{title}“ ({book}) wurde am {date} aus {shelf} entfernt. Die Datei ist gelöscht, die Codes der Karten funktionieren nicht mehr, und SHA-256-Fingerabdruck und ISBN können nicht mehr hochgeladen werden.\nSachverhalt: Meldung {notice} vom {received}. {statement}\nGrundlage: {ground}\nAutomatisierte Mittel: keine; ein Mensch hat nach einer Meldung entschieden.\n{false}\nRechtsbehelfe:\n- Beschwerde bei uns binnen 6 Monaten über Ihr Konto: {link}\n- eine zertifizierte außergerichtliche Streitbeilegungsstelle (Art. 21);\n- die Gerichte.\n",
        "Hola:\n\nExposición de motivos (Reglamento de Servicios Digitales, art. 17).\n\nDecisión: tu libro «{title}» ({book}) se retira de {shelf} el {date}. El archivo está borrado, los códigos de sus tarjetas ya no funcionan y su huella SHA-256 y su ISBN ya no pueden subirse.\nHechos: denuncia {notice} recibida el {received}. {statement}\nFundamento: {ground}\nMedios automatizados: ninguno; la decisión la tomó una persona tras una denuncia.\n{false}\nVías de recurso:\n- una reclamación ante nosotros, en 6 meses, desde tu cuenta: {link}\n- un órgano certificado de resolución extrajudicial de litigios (art. 21);\n- los tribunales.\n",
        "Buongiorno,\n\nMotivazione (regolamento sui servizi digitali, art. 17).\n\nDecisione: il tuo libro «{title}» ({book}) è rimosso da {shelf} il {date}. Il file è cancellato, i codici dei biglietti non funzionano più e la sua impronta SHA-256 e il suo ISBN non possono più essere caricati.\nFatti: segnalazione {notice} ricevuta il {received}. {statement}\nFondamento: {ground}\nMezzi automatizzati: nessuno; la decisione è stata presa da una persona a seguito di una segnalazione.\n{false}\nMezzi di ricorso:\n- un reclamo presso di noi, entro 6 mesi, dal tuo account: {link}\n- un organismo certificato di risoluzione extragiudiziale delle controversie (art. 21);\n- il giudice.\n",
        "Hallo,\n\nMotivering (digitaledienstenverordening, art. 17).\n\nBeslissing: je boek ‘{title}’ ({book}) is op {date} van {shelf} verwijderd. Het bestand is gewist, de codes van de kaarten werken niet meer, en de SHA-256-vingerafdruk en het ISBN kunnen niet meer worden geüpload.\nFeiten: melding {notice} ontvangen op {received}. {statement}\nGrond: {ground}\nGeautomatiseerde middelen: geen; een persoon besliste na een melding.\n{false}\nBezwaarmogelijkheden:\n- een klacht bij ons, binnen 6 maanden, via je account: {link}\n- een erkende instantie voor buitengerechtelijke geschilbeslechting (art. 21);\n- de rechter.\n",
        "Olá,\n\nExposição de motivos (Regulamento dos Serviços Digitais, art. 17).\n\nDecisão: o seu livro «{title}» ({book}) foi removido de {shelf} em {date}. O ficheiro foi apagado, os códigos dos cartões deixaram de funcionar e a sua impressão SHA-256 e o ISBN já não podem ser carregados.\nFactos: denúncia {notice} recebida em {received}. {statement}\nFundamento: {ground}\nMeios automatizados: nenhum; a decisão foi tomada por uma pessoa na sequência de uma denúncia.\n{false}\nVias de recurso:\n- uma reclamação junto de nós, no prazo de 6 meses, a partir da sua conta: {link}\n- um organismo certificado de resolução extrajudicial de litígios (art. 21);\n- o tribunal.\n",
    ];
    /// Ground: copyright.
    GroundCopyright => [
        "atteinte au droit d’auteur (Code de la propriété intellectuelle, art. L122-4)\u{a0}; conditions d’utilisation, « Qui peut déposer quoi ».",
        "copyright infringement (French Intellectual Property Code, Art. L122-4); terms of use, “Who may upload what”.",
        "Urheberrechtsverletzung (franz. Gesetzbuch über geistiges Eigentum, Art. L122-4); Nutzungsbedingungen, „Wer was hochladen darf“.",
        "infracción de derechos de autor (Código de la propiedad intelectual francés, art. L122-4); condiciones de uso, «Quién puede subir qué».",
        "violazione del diritto d’autore (Codice francese della proprietà intellettuale, art. L122-4); condizioni d’uso, «Chi può caricare cosa».",
        "inbreuk op het auteursrecht (Franse wet op de intellectuele eigendom, art. L122-4); gebruiksvoorwaarden, ‘Wie mag wat uploaden’.",
        "violação de direitos de autor (Código francês da propriedade intelectual, art. L122-4); condições de utilização, «Quem pode carregar o quê».",
    ];
    /// Ground: other.
    GroundOther => ["contenu illicite\u{a0}; conditions d’utilisation.", "illegal content; terms of use.", "rechtswidriger Inhalt; Nutzungsbedingungen.", "contenido ilícito; condiciones de uso.", "contenuto illecito; condizioni d’uso.", "illegale inhoud; gebruiksvoorwaarden.", "conteúdo ilícito; condições de utilização."];
    /// False declaration noted.
    FalseDeclaration => ["La déclaration de droits faite au dépôt était inexacte.\n", "The rights declaration made at upload was false.\n", "Die beim Hochladen abgegebene Rechteerklärung war falsch.\n", "La declaración de derechos hecha al subir el archivo era falsa.\n", "La dichiarazione dei diritti fatta al caricamento era falsa.\n", "De rechtenverklaring bij het uploaden was onjuist.\n", "A declaração de direitos feita no carregamento era falsa.\n"];
    /// Suspension (repeat-infringer policy).
    MailSuspended => [
        "\nVotre compte est suspendu en application de notre politique envers les contrevenants répétés ({upheld} signalement(s) fondé(s), {false} fausse(s) déclaration(s))\u{a0}: vous ne pouvez plus déposer de livre ni créer de cadeau. Les cadeaux de vos autres livres continuent de fonctionner. Cette suspension peut être contestée par les mêmes voies.\n",
        "\nYour account is suspended under our repeat-infringer policy ({upheld} upheld notice(s), {false} false declaration(s)): you can no longer upload books or create gifts. Gifts of your other books keep working. The suspension can be contested the same ways.\n",
        "\nIhr Konto ist nach unserer Richtlinie zu Wiederholungstätern gesperrt ({upheld} begründete Meldung(en), {false} falsche Erklärung(en)): Sie können keine Bücher mehr hochladen und keine Geschenke anlegen. Geschenke Ihrer anderen Bücher funktionieren weiter. Die Sperre kann auf denselben Wegen angefochten werden.\n",
        "\nTu cuenta queda suspendida según nuestra política de infractores reincidentes ({upheld} denuncia(s) fundada(s), {false} declaración(es) falsa(s)): ya no puedes subir libros ni crear regalos. Los regalos de tus otros libros siguen funcionando. La suspensión puede impugnarse por las mismas vías.\n",
        "\nIl tuo account è sospeso in base alla nostra politica sui trasgressori recidivi ({upheld} segnalazione/i fondata/e, {false} dichiarazione/i falsa/e): non puoi più caricare libri né creare regali. I regali degli altri tuoi libri continuano a funzionare. La sospensione può essere contestata con gli stessi mezzi.\n",
        "\nJe account is geschorst volgens ons beleid voor herhaalde inbreuk ({upheld} gegronde melding(en), {false} valse verklaring(en)): je kunt geen boeken meer uploaden of cadeaus maken. Cadeaus van je andere boeken blijven werken. De schorsing kan op dezelfde manieren worden aangevochten.\n",
        "\nA sua conta está suspensa ao abrigo da nossa política para infratores reincidentes ({upheld} denúncia(s) procedente(s), {false} declaração(ões) falsa(s)): já não pode carregar livros nem criar ofertas. As ofertas dos seus outros livros continuam a funcionar. A suspensão pode ser contestada pelas mesmas vias.\n",
    ];

    // ----- decisions and appeals -----
    /// Decision page title.
    DecisionTitle => ["Décision {id}", "Decision {id}", "Entscheidung {id}", "Decisión {id}", "Decisione {id}", "Beslissing {id}", "Decisão {id}"];
    /// Decision summary.
    DecisionUpheld => ["Livre « {title} » retiré le {date} à la suite d’un signalement.", "Book “{title}” removed on {date} following a notice.", "Buch „{title}“ am {date} nach einer Meldung entfernt.", "Libro «{title}» retirado el {date} tras una denuncia.", "Libro «{title}» rimosso il {date} a seguito di una segnalazione.", "Boek ‘{title}’ op {date} verwijderd na een melding.", "Livro «{title}» removido em {date} na sequência de uma denúncia."];
    /// Reasons.
    Reasons => ["Motifs\u{a0}: {statement}", "Reasons: {statement}", "Gründe: {statement}", "Motivos: {statement}", "Motivi: {statement}", "Redenen: {statement}", "Motivos: {statement}"];
    /// Appeal intro.
    AppealIntro => [
        "Vous pouvez contester cette décision jusqu’au {date}. Expliquez pourquoi elle devrait être annulée\u{a0}; une personne réexaminera le dossier.",
        "You can contest this decision until {date}. Explain why it should be reversed; a person will review the case.",
        "Sie können diese Entscheidung bis zum {date} anfechten. Erklären Sie, warum sie aufgehoben werden sollte; ein Mensch prüft den Fall erneut.",
        "Puedes impugnar esta decisión hasta el {date}. Explica por qué debería anularse; una persona revisará el caso.",
        "Puoi contestare questa decisione fino al {date}. Spiega perché dovrebbe essere annullata; una persona riesaminerà il caso.",
        "Je kunt deze beslissing tot {date} aanvechten. Leg uit waarom ze moet worden herzien; een persoon bekijkt de zaak opnieuw.",
        "Pode contestar esta decisão até {date}. Explique porque deve ser anulada; uma pessoa reexaminará o caso.",
    ];
    /// Appeal field.
    AppealLabel => ["Vos arguments", "Your arguments", "Ihre Begründung", "Tus argumentos", "I tuoi argomenti", "Je argumenten", "Os seus argumentos"];
    /// Appeal button.
    SendAppeal => ["Contester", "Contest", "Anfechten", "Impugnar", "Contesta", "Aanvechten", "Contestar"];
    /// Appeal filed.
    AppealFiled => ["Réclamation envoyée le {date}\u{a0}; en cours d’examen.", "Complaint sent on {date}; under review.", "Beschwerde am {date} gesendet; wird geprüft.", "Reclamación enviada el {date}; en examen.", "Reclamo inviato il {date}; in esame.", "Klacht verstuurd op {date}; in behandeling.", "Reclamação enviada em {date}; em análise."];
    /// Appeal granted.
    AppealGranted => ["Réclamation acceptée le {date}\u{a0}: {answer}", "Complaint upheld on {date}: {answer}", "Beschwerde am {date} stattgegeben: {answer}", "Reclamación aceptada el {date}: {answer}", "Reclamo accolto il {date}: {answer}", "Klacht gegrond verklaard op {date}: {answer}", "Reclamação aceite em {date}: {answer}"];
    /// Appeal dismissed.
    AppealDismissed => ["Réclamation rejetée le {date}\u{a0}: {answer}", "Complaint dismissed on {date}: {answer}", "Beschwerde am {date} abgewiesen: {answer}", "Reclamación rechazada el {date}: {answer}", "Reclamo respinto il {date}: {answer}", "Klacht afgewezen op {date}: {answer}", "Reclamação rejeitada em {date}: {answer}"];
    /// Appeal window over.
    AppealClosed => ["Le délai de réclamation est écoulé.", "The time to complain has passed.", "Die Beschwerdefrist ist abgelaufen.", "El plazo de reclamación ha vencido.", "Il termine per il reclamo è scaduto.", "De termijn voor een klacht is verstreken.", "O prazo de reclamação terminou."];
    /// Appeal outcome subject.
    MailAppealSubject => ["Réponse à votre réclamation — {shelf}", "Answer to your complaint — {shelf}", "Antwort auf Ihre Beschwerde — {shelf}", "Respuesta a tu reclamación — {shelf}", "Risposta al tuo reclamo — {shelf}", "Antwoord op je klacht — {shelf}", "Resposta à sua reclamação — {shelf}"];
    /// Appeal granted body.
    MailAppealGranted => [
        "Bonjour,\n\nVotre réclamation est acceptée\u{a0}: la décision {notice} est annulée et ne compte plus dans notre politique envers les contrevenants répétés. {answer}\nLe livre peut de nouveau être déposé (le fichier effacé ne peut pas être restauré).\n",
        "Hello,\n\nYour complaint is upheld: decision {notice} is reversed and no longer counts under our repeat-infringer policy. {answer}\nThe book may be uploaded again (the erased file cannot be restored).\n",
        "Guten Tag,\n\nIhrer Beschwerde wird stattgegeben: Die Entscheidung {notice} ist aufgehoben und zählt nicht mehr für unsere Richtlinie zu Wiederholungstätern. {answer}\nDas Buch kann erneut hochgeladen werden (die gelöschte Datei lässt sich nicht wiederherstellen).\n",
        "Hola:\n\nTu reclamación se acepta: la decisión {notice} queda anulada y ya no cuenta en nuestra política de reincidencia. {answer}\nEl libro puede volver a subirse (el archivo borrado no se puede restaurar).\n",
        "Buongiorno,\n\nil tuo reclamo è accolto: la decisione {notice} è annullata e non conta più per la nostra politica sui recidivi. {answer}\nIl libro può essere ricaricato (il file cancellato non può essere ripristinato).\n",
        "Hallo,\n\nJe klacht is gegrond: beslissing {notice} is herroepen en telt niet meer mee voor ons beleid voor herhaalde inbreuk. {answer}\nHet boek mag opnieuw worden geüpload (het gewiste bestand kan niet worden hersteld).\n",
        "Olá,\n\nA sua reclamação foi aceite: a decisão {notice} foi anulada e deixa de contar para a nossa política de reincidência. {answer}\nO livro pode voltar a ser carregado (o ficheiro apagado não pode ser restaurado).\n",
    ];
    /// Appeal dismissed body.
    MailAppealDismissed => [
        "Bonjour,\n\nVotre réclamation contre la décision {notice} est rejetée\u{a0}: {answer}\nVous pouvez encore vous adresser à un organisme de règlement extrajudiciaire des litiges certifié (DSA, art. 21) ou saisir le juge.\n",
        "Hello,\n\nYour complaint against decision {notice} is dismissed: {answer}\nYou may still turn to a certified out-of-court dispute settlement body (DSA Art. 21) or go to court.\n",
        "Guten Tag,\n\nIhre Beschwerde gegen die Entscheidung {notice} wird abgewiesen: {answer}\nSie können sich weiterhin an eine zertifizierte außergerichtliche Streitbeilegungsstelle (DSA Art. 21) oder ein Gericht wenden.\n",
        "Hola:\n\nTu reclamación contra la decisión {notice} se rechaza: {answer}\nAún puedes acudir a un órgano certificado de resolución extrajudicial de litigios (DSA, art. 21) o a los tribunales.\n",
        "Buongiorno,\n\nil tuo reclamo contro la decisione {notice} è respinto: {answer}\nPuoi ancora rivolgerti a un organismo certificato di risoluzione extragiudiziale (DSA, art. 21) o al giudice.\n",
        "Hallo,\n\nJe klacht tegen beslissing {notice} is afgewezen: {answer}\nJe kunt nog naar een erkende instantie voor buitengerechtelijke geschilbeslechting (DSA art. 21) of naar de rechter.\n",
        "Olá,\n\nA sua reclamação contra a decisão {notice} foi rejeitada: {answer}\nPode ainda recorrer a um organismo certificado de resolução extrajudicial de litígios (DSA, art. 21) ou ao tribunal.\n",
    ];

    // ----- moderation -----
    /// Open notices.
    OpenNotices => ["Signalements à traiter", "Notices to handle", "Offene Meldungen", "Denuncias pendientes", "Segnalazioni da trattare", "Te behandelen meldingen", "Denúncias por tratar"];
    /// None open.
    NothingOpen => ["Rien en attente.", "Nothing pending.", "Nichts offen.", "Nada pendiente.", "Niente in sospeso.", "Niets openstaand.", "Nada pendente."];
    /// Open appeals.
    OpenAppeals => ["Réclamations à traiter", "Complaints to handle", "Offene Beschwerden", "Reclamaciones pendientes", "Reclami da trattare", "Te behandelen klachten", "Reclamações por tratar"];
    /// Closed notices.
    ClosedNotices => ["Décisions prises", "Decisions taken", "Getroffene Entscheidungen", "Decisiones tomadas", "Decisioni prese", "Genomen beslissingen", "Decisões tomadas"];
    /// Blocklist heading.
    Blocklist => ["Liste de blocage", "Blocklist", "Sperrliste", "Lista de bloqueo", "Lista di blocco", "Blokkeerlijst", "Lista de bloqueio"];
    /// Received on.
    ReceivedOn => ["reçu le {date}", "received on {date}", "erhalten am {date}", "recibida el {date}", "ricevuta il {date}", "ontvangen op {date}", "recebida em {date}"];
    /// Location.
    LocationIs => ["Localisation\u{a0}: {location}", "Location: {location}", "Ort: {location}", "Ubicación: {location}", "Posizione: {location}", "Plaats: {location}", "Localização: {location}"];
    /// Resolved book.
    ResolvedTo => ["Livre concerné\u{a0}: « {title} » ({id})", "Book concerned: “{title}” ({id})", "Betroffenes Buch: „{title}“ ({id})", "Libro afectado: «{title}» ({id})", "Libro interessato: «{title}» ({id})", "Betrokken boek: ‘{title}’ ({id})", "Livro em causa: «{title}» ({id})"];
    /// Unresolved.
    Unresolved => [
        "Le livre n’a pas pu être identifié automatiquement\u{a0}: indiquez son identifiant.",
        "The book could not be identified automatically: give its identifier.",
        "Das Buch konnte nicht automatisch ermittelt werden: Geben Sie seine Kennung an.",
        "No se ha podido identificar el libro automáticamente: indica su identificador.",
        "Il libro non è stato identificato automaticamente: indica il suo identificativo.",
        "Het boek kon niet automatisch worden gevonden: geef het kenmerk op.",
        "Não foi possível identificar o livro automaticamente: indique o identificador.",
    ];
    /// Book id field.
    BookIdLabel => ["Identifiant du livre (b_…)", "Book identifier (b_…)", "Buchkennung (b_…)", "Identificador del libro (b_…)", "Identificativo del libro (b_…)", "Kenmerk van het boek (b_…)", "Identificador do livro (b_…)"];
    /// Notifier line.
    NotifierIs => ["Signalé par\u{a0}: {name} <{email}>", "Reported by: {name} <{email}>", "Gemeldet von: {name} <{email}>", "Denunciado por: {name} <{email}>", "Segnalato da: {name} <{email}>", "Gemeld door: {name} <{email}>", "Denunciado por: {name} <{email}>"];
    /// Notifier anonymous (exception).
    NotifierException => ["Signalé sous l’exception de l’article 16(2)(c)", "Reported under the Article 16(2)(c) exception", "Gemeldet unter der Ausnahme des Art. 16 Abs. 2 Buchst. c", "Denunciado con la excepción del artículo 16(2)(c)", "Segnalato con l’eccezione dell’articolo 16(2)(c)", "Gemeld onder de uitzondering van artikel 16(2)(c)", "Denunciado ao abrigo da exceção do artigo 16.º, n.º 2, alínea c)"];
    /// Uploader line.
    UploaderIs => ["Déposant\u{a0}: {name}, {address} — {email}", "Uploader: {name}, {address} — {email}", "Hochgeladen von: {name}, {address} — {email}", "Depositante: {name}, {address} — {email}", "Caricato da: {name}, {address} — {email}", "Geüpload door: {name}, {address} — {email}", "Depositante: {name}, {address} — {email}"];
    /// Decision legend.
    DecisionLegend => ["Décision", "Decision", "Entscheidung", "Decisión", "Decisione", "Beslissing", "Decisão"];
    /// Uphold.
    Uphold => ["Retirer le contenu", "Remove the content", "Inhalt entfernen", "Retirar el contenido", "Rimuovi il contenuto", "Inhoud verwijderen", "Remover o conteúdo"];
    /// Reject.
    Reject => ["Ne pas retirer", "Do not remove", "Nicht entfernen", "No retirar", "Non rimuovere", "Niet verwijderen", "Não remover"];
    /// Statement field.
    StatementLabel => [
        "Motifs (faits et raisons\u{a0}; envoyés au déposant, ou au notifiant en cas de refus)",
        "Reasons (facts and grounds; sent to the uploader, or to the notifier if not removed)",
        "Begründung (Sachverhalt und Gründe; geht an den Uploader bzw. bei Ablehnung an den Melder)",
        "Motivos (hechos y razones; se envían al depositante, o al denunciante si no se retira)",
        "Motivi (fatti e ragioni; inviati a chi ha caricato, o al segnalante se non si rimuove)",
        "Motivering (feiten en gronden; naar de uploader, of naar de melder als niets wordt verwijderd)",
        "Motivos (factos e razões; enviados ao depositante, ou ao denunciante se não for removido)",
    ];
    /// False declaration checkbox.
    FalseDeclLabel => ["La déclaration de droits était fausse", "The rights declaration was false", "Die Rechteerklärung war falsch", "La declaración de derechos era falsa", "La dichiarazione dei diritti era falsa", "De rechtenverklaring was onjuist", "A declaração de direitos era falsa"];
    /// Decide button.
    Decide => ["Enregistrer la décision", "Record the decision", "Entscheidung speichern", "Registrar la decisión", "Registra la decisione", "Beslissing opslaan", "Registar a decisão"];
    /// Decision form error.
    ErrDecision => [
        "Choisissez une décision et rédigez les motifs\u{a0}; pour un retrait, le livre doit être identifié et encore en ligne.",
        "Choose a decision and write the reasons; to remove, the book must be identified and still online.",
        "Wählen Sie eine Entscheidung und schreiben Sie die Gründe; zum Entfernen muss das Buch ermittelt und noch online sein.",
        "Elige una decisión y redacta los motivos; para retirar, el libro debe estar identificado y aún en línea.",
        "Scegli una decisione e scrivi i motivi; per rimuovere, il libro deve essere identificato e ancora online.",
        "Kies een beslissing en schrijf de motivering; om te verwijderen moet het boek bekend en nog online zijn.",
        "Escolha uma decisão e escreva os motivos; para remover, o livro tem de estar identificado e ainda em linha.",
    ];
    /// Grant appeal.
    GrantAppeal => ["Accepter la réclamation", "Uphold the complaint", "Beschwerde stattgeben", "Aceptar la reclamación", "Accogli il reclamo", "Klacht gegrond verklaren", "Aceitar a reclamação"];
    /// Dismiss appeal.
    DismissAppeal => ["Rejeter la réclamation", "Dismiss the complaint", "Beschwerde abweisen", "Rechazar la reclamación", "Respingi il reclamo", "Klacht afwijzen", "Rejeitar a reclamação"];
    /// Answer field.
    AnswerLabel => ["Réponse motivée", "Reasoned answer", "Begründete Antwort", "Respuesta motivada", "Risposta motivata", "Gemotiveerd antwoord", "Resposta fundamentada"];
    /// Suspended accounts heading.
    SuspendedAccounts => ["Comptes suspendus", "Suspended accounts", "Gesperrte Konten", "Cuentas suspendidas", "Account sospesi", "Geschorste accounts", "Contas suspensas"];

    // ----- legal pages -----
    /// Terms title.
    TermsTitle => ["Conditions d’utilisation", "Terms of use", "Nutzungsbedingungen", "Condiciones de uso", "Condizioni d’uso", "Gebruiksvoorwaarden", "Condições de utilização"];
    /// Terms: what.
    TermsWhatHead => ["Ce que fait ce rayonnage", "What this shelf does", "Was dieses Regal tut", "Qué hace esta estantería", "Cosa fa questo scaffale", "Wat deze plank doet", "O que faz esta estante"];
    /// Terms: what (text).
    TermsWhat => [
        "Il conserve des fichiers EPUB ou PDF déposés par leurs ayants droit et les remet, sans les modifier, à qui détient une carte eBook Cadeau créée par le déposant\u{a0}: le service eBook Cadeau demande pour chaque réclamation un lien de téléchargement de courte durée. Il n’y a ni catalogue, ni recherche, ni page publique de livre\u{a0}; quiconque détient la carte peut télécharger le livre, dans la limite fixée par le service eBook Cadeau. Rien n’est vendu ici.",
        "It stores EPUB or PDF files uploaded by their rights holders and delivers them, unchanged, to whoever holds an eBook Cadeau card the uploader created: the eBook Cadeau service asks for a short-lived download link on each claim. There is no catalogue, no search and no public book page; whoever holds the card may download the book, within the limit set by the eBook Cadeau service. Nothing is sold here.",
        "Es speichert EPUB- oder PDF-Dateien, die ihre Rechteinhaber hochladen, und liefert sie unverändert an jede Person aus, die eine vom Uploader erstellte eBook-Cadeau-Karte besitzt: Der eBook-Cadeau-Dienst fordert bei jedem Abruf einen kurzlebigen Download-Link an. Es gibt keinen Katalog, keine Suche und keine öffentliche Buchseite; wer die Karte hat, kann das Buch im Rahmen des Limits des eBook-Cadeau-Dienstes herunterladen. Hier wird nichts verkauft.",
        "Conserva archivos EPUB o PDF subidos por sus titulares y los entrega, sin modificarlos, a quien tenga una tarjeta eBook Cadeau creada por quien los subió: el servicio eBook Cadeau pide en cada canje un enlace de descarga de corta duración. No hay catálogo, ni buscador, ni página pública del libro; quien tenga la tarjeta puede descargar el libro dentro del límite del servicio eBook Cadeau. Aquí no se vende nada.",
        "Conserva file EPUB o PDF caricati dai titolari dei diritti e li consegna, senza modificarli, a chi possiede un biglietto eBook Cadeau creato da chi li ha caricati: il servizio eBook Cadeau chiede a ogni riscatto un link di download di breve durata. Non ci sono catalogo, ricerca né pagine pubbliche dei libri; chi possiede il biglietto può scaricare il libro entro il limite del servizio eBook Cadeau. Qui non si vende nulla.",
        "Ze bewaart EPUB- of pdf-bestanden die hun rechthebbenden uploaden en bezorgt ze ongewijzigd aan wie een eBook Cadeau-kaart heeft die de uploader maakte: de eBook Cadeau-dienst vraagt bij elke inwisseling een kortlevende downloadlink. Er is geen catalogus, geen zoekfunctie en geen openbare boekpagina; wie de kaart heeft, kan het boek downloaden binnen de limiet van de eBook Cadeau-dienst. Hier wordt niets verkocht.",
        "Conserva ficheiros EPUB ou PDF carregados pelos titulares dos direitos e entrega-os, sem os alterar, a quem tiver um cartão eBook Cadeau criado por quem os carregou: o serviço eBook Cadeau pede em cada resgate uma ligação de descarga de curta duração. Não há catálogo, pesquisa nem página pública do livro; quem tiver o cartão pode descarregar o livro dentro do limite do serviço eBook Cadeau. Nada é vendido aqui.",
    ];
    /// Terms: who.
    TermsWhoHead => ["Qui peut déposer quoi", "Who may upload what", "Wer was hochladen darf", "Quién puede subir qué", "Chi può caricare cosa", "Wie mag wat uploaden", "Quem pode carregar o quê"];
    /// Terms: who (text).
    TermsWho => [
        "Seuls les ayants droit déposent\u{a0}: l’auteur ou l’éditeur qui détient les droits numériques, une œuvre du domaine public en France, une œuvre sous licence libre (CC0, CC BY, CC BY-SA, et les variantes NC et ND pour un usage non commercial), ou un diffuseur autorisé par l’ayant droit. Chaque dépôt porte une déclaration de droits, horodatée et liée au compte, à son identification et à l’empreinte du fichier. Un livre acheté dans une librairie, avec ou sans DRM, ne peut jamais être déposé.",
        "Only rights holders upload: the author or publisher holding the digital rights, a work in the public domain in France, a work under an open licence (CC0, CC BY, CC BY-SA, and the NC and ND variants for non-commercial use), or a distributor licensed by the rights holder. Every upload carries a rights declaration, time-stamped and tied to the account, its identification and the file’s fingerprint. A book bought from a store, with or without DRM, can never be uploaded.",
        "Hochladen dürfen nur Rechteinhaber: Autor/in oder Verlag mit den digitalen Rechten, ein in Frankreich gemeinfreies Werk, ein Werk unter freier Lizenz (CC0, CC BY, CC BY-SA sowie NC- und ND-Varianten für nicht kommerzielle Nutzung) oder ein vom Rechteinhaber autorisierter Vertrieb. Jeder Upload trägt eine Rechteerklärung mit Zeitstempel, verknüpft mit Konto, Identifikation und Fingerabdruck der Datei. Ein im Handel gekauftes Buch, mit oder ohne DRM, darf nie hochgeladen werden.",
        "Solo suben archivos los titulares de derechos: el autor o la editorial con los derechos digitales, una obra de dominio público en Francia, una obra con licencia libre (CC0, CC BY, CC BY-SA y las variantes NC y ND para uso no comercial) o un distribuidor autorizado por el titular. Cada subida lleva una declaración de derechos con fecha y hora, vinculada a la cuenta, a su identificación y a la huella del archivo. Un libro comprado en una librería, con o sin DRM, nunca puede subirse.",
        "Caricano solo i titolari dei diritti: l’autore o l’editore con i diritti digitali, un’opera di pubblico dominio in Francia, un’opera sotto licenza libera (CC0, CC BY, CC BY-SA e le varianti NC e ND per uso non commerciale) o un distributore autorizzato dal titolare. Ogni caricamento reca una dichiarazione dei diritti con data e ora, legata all’account, alla sua identificazione e all’impronta del file. Un libro acquistato in libreria, con o senza DRM, non può mai essere caricato.",
        "Alleen rechthebbenden uploaden: de auteur of uitgever met de digitale rechten, een werk dat in Frankrijk publiek domein is, een werk onder een open licentie (CC0, CC BY, CC BY-SA, en de NC- en ND-varianten voor niet-commercieel gebruik), of een distributeur met toestemming van de rechthebbende. Elke upload draagt een rechtenverklaring met tijdstempel, gekoppeld aan het account, de identificatie en de vingerafdruk van het bestand. Een gekocht boek, met of zonder DRM, mag nooit worden geüpload.",
        "Só os titulares dos direitos carregam: o autor ou editor com os direitos digitais, uma obra no domínio público em França, uma obra sob licença livre (CC0, CC BY, CC BY-SA e as variantes NC e ND para uso não comercial) ou um distribuidor autorizado pelo titular. Cada carregamento leva uma declaração de direitos com data e hora, ligada à conta, à sua identificação e à impressão digital do ficheiro. Um livro comprado numa livraria, com ou sem DRM, nunca pode ser carregado.",
    ];
    /// Terms: checks.
    TermsChecksHead => ["Contrôles au dépôt", "Checks at upload", "Prüfungen beim Hochladen", "Controles al subir", "Controlli al caricamento", "Controles bij het uploaden", "Controlos no carregamento"];
    /// Terms: checks (text).
    TermsChecks => [
        "Le type du fichier est reconnu d’après son contenu\u{a0}; les fichiers protégés par DRM (Adobe, Readium LCP, chiffrement autre que l’obscurcissement des polices) et les PDF chiffrés sont refusés\u{a0}; la taille est limitée à {max} par fichier, avec un espace limité par compte\u{a0}; l’empreinte SHA-256 et les ISBN (déclaré et inscrits dans le fichier) sont comparés à la liste des contenus retirés. Ces contrôles automatiques n’attrapent que les cas évidents\u{a0}: la déclaration et les signalements restent le vrai garde-fou.",
        "The file type is recognised from its content; files protected by DRM (Adobe, Readium LCP, any encryption beyond font obfuscation) and encrypted PDFs are refused; files are limited to {max} each, with limited storage per account; the SHA-256 fingerprint and the ISBNs (declared and written in the file) are compared with the list of removed content. These automatic checks only catch obvious cases: the declaration and the notices remain the real safeguard.",
        "Der Dateityp wird am Inhalt erkannt; DRM-geschützte Dateien (Adobe, Readium LCP, jede Verschlüsselung außer Schriftverschleierung) und verschlüsselte PDFs werden abgelehnt; je Datei höchstens {max}, begrenzter Speicher je Konto; SHA-256-Fingerabdruck und ISBNs (angegeben und in der Datei) werden mit der Liste entfernter Inhalte abgeglichen. Diese automatischen Prüfungen erkennen nur offensichtliche Fälle: Erklärung und Meldungen bleiben die eigentliche Sicherung.",
        "El tipo de archivo se reconoce por su contenido; se rechazan los archivos con DRM (Adobe, Readium LCP, cualquier cifrado salvo la ofuscación de fuentes) y los PDF cifrados; cada archivo tiene un máximo de {max}, con espacio limitado por cuenta; la huella SHA-256 y los ISBN (declarado e inscritos en el archivo) se comparan con la lista de contenidos retirados. Estos controles automáticos solo detectan los casos evidentes: la declaración y las denuncias siguen siendo la verdadera salvaguarda.",
        "Il tipo di file è riconosciuto dal contenuto; i file protetti da DRM (Adobe, Readium LCP, qualsiasi cifratura oltre l’offuscamento dei font) e i PDF cifrati sono rifiutati; ogni file ha un massimo di {max}, con spazio limitato per account; l’impronta SHA-256 e gli ISBN (dichiarato e presenti nel file) sono confrontati con l’elenco dei contenuti rimossi. Questi controlli automatici colgono solo i casi evidenti: la dichiarazione e le segnalazioni restano la vera garanzia.",
        "Het bestandstype wordt aan de inhoud herkend; met DRM beveiligde bestanden (Adobe, Readium LCP, elke versleuteling behalve het verhullen van lettertypen) en versleutelde pdf’s worden geweigerd; elk bestand is hoogstens {max}, met beperkte opslag per account; de SHA-256-vingerafdruk en de ISBN’s (opgegeven en in het bestand) worden vergeleken met de lijst van verwijderde inhoud. Deze automatische controles vangen alleen duidelijke gevallen: de verklaring en de meldingen blijven de echte waarborg.",
        "O tipo de ficheiro é reconhecido pelo conteúdo; ficheiros protegidos por DRM (Adobe, Readium LCP, qualquer cifra além da ofuscação de tipos de letra) e PDF cifrados são recusados; cada ficheiro tem no máximo {max}, com espaço limitado por conta; a impressão SHA-256 e os ISBN (declarado e inscritos no ficheiro) são comparados com a lista de conteúdos removidos. Estes controlos automáticos só apanham os casos evidentes: a declaração e as denúncias continuam a ser a verdadeira salvaguarda.",
    ];
    /// Terms: moderation.
    TermsModerationHead => ["Signalements, décisions et recours", "Notices, decisions and redress", "Meldungen, Entscheidungen und Rechtsbehelfe", "Denuncias, decisiones y recursos", "Segnalazioni, decisioni e ricorsi", "Meldingen, beslissingen en bezwaar", "Denúncias, decisões e recursos"];
    /// Terms: moderation (text).
    TermsModeration => [
        "Chacun peut signaler un contenu illicite avec le formulaire de signalement, sans compte. Nous accusons réception, puis une personne décide avec diligence (objectif\u{a0}: 24 heures), sans décision automatisée. Si le signalement est fondé, le fichier est effacé, les codes de toutes ses cartes cessent de fonctionner, et son empreinte et son ISBN sont bloqués. Le déposant reçoit un exposé des motifs et peut former une réclamation interne dans les 6 mois depuis son compte, s’adresser à un organisme de règlement extrajudiciaire des litiges certifié ou saisir le juge\u{a0}; le notifiant est informé de la décision.",
        "Anyone may report illegal content with the notice form, without an account. We acknowledge receipt, then a person decides diligently (target: 24 hours), with no automated decision. If the notice is upheld, the file is erased, the codes of all its cards stop working, and its fingerprint and ISBN are blocked. The uploader receives a statement of reasons and may file an internal complaint within 6 months from their account, turn to a certified out-of-court dispute settlement body or go to court; the notifier is told the decision.",
        "Jede Person kann rechtswidrige Inhalte ohne Konto über das Meldeformular melden. Wir bestätigen den Eingang; dann entscheidet ein Mensch sorgfältig (Ziel: 24 Stunden), ohne automatisierte Entscheidung. Ist die Meldung begründet, wird die Datei gelöscht, die Codes aller Karten funktionieren nicht mehr, und Fingerabdruck und ISBN werden gesperrt. Der Uploader erhält eine Begründung und kann binnen 6 Monaten über sein Konto Beschwerde einlegen, sich an eine zertifizierte außergerichtliche Streitbeilegungsstelle wenden oder ein Gericht anrufen; die meldende Person wird über die Entscheidung informiert.",
        "Cualquiera puede denunciar un contenido ilícito con el formulario, sin cuenta. Acusamos recibo y una persona decide con diligencia (objetivo: 24 horas), sin decisiones automatizadas. Si la denuncia es fundada, el archivo se borra, los códigos de todas sus tarjetas dejan de funcionar y su huella e ISBN se bloquean. Quien lo subió recibe una exposición de motivos y puede presentar una reclamación interna en 6 meses desde su cuenta, acudir a un órgano certificado de resolución extrajudicial o a los tribunales; el denunciante es informado de la decisión.",
        "Chiunque può segnalare un contenuto illecito con il modulo, senza account. Confermiamo la ricezione, poi una persona decide con diligenza (obiettivo: 24 ore), senza decisioni automatizzate. Se la segnalazione è fondata, il file è cancellato, i codici di tutti i biglietti smettono di funzionare e la sua impronta e il suo ISBN sono bloccati. Chi ha caricato riceve una motivazione e può presentare un reclamo interno entro 6 mesi dal proprio account, rivolgersi a un organismo certificato di risoluzione extragiudiziale o al giudice; il segnalante è informato della decisione.",
        "Iedereen kan illegale inhoud zonder account melden met het meldformulier. We bevestigen de ontvangst, daarna beslist een persoon zorgvuldig (doel: 24 uur), zonder geautomatiseerde beslissing. Is de melding gegrond, dan wordt het bestand gewist, werken de codes van alle kaarten niet meer en worden de vingerafdruk en het ISBN geblokkeerd. De uploader krijgt een motivering en kan binnen 6 maanden via het account een interne klacht indienen, een erkende instantie voor buitengerechtelijke geschilbeslechting inschakelen of naar de rechter stappen; de melder hoort de beslissing.",
        "Qualquer pessoa pode denunciar um conteúdo ilícito com o formulário, sem conta. Acusamos a receção e uma pessoa decide com diligência (objetivo: 24 horas), sem decisões automatizadas. Se a denúncia for procedente, o ficheiro é apagado, os códigos de todos os cartões deixam de funcionar e a impressão e o ISBN são bloqueados. Quem carregou recebe uma exposição de motivos e pode apresentar uma reclamação interna no prazo de 6 meses a partir da conta, recorrer a um organismo certificado de resolução extrajudicial ou ao tribunal; o denunciante é informado da decisão.",
    ];
    /// Terms: repeat infringers.
    TermsRepeatHead => ["Contrevenants répétés", "Repeat infringers", "Wiederholungstäter", "Infractores reincidentes", "Trasgressori recidivi", "Herhaalde inbreuk", "Infratores reincidentes"];
    /// Terms: repeat infringers (text).
    TermsRepeat => [
        "Un compte est suspendu après {upheld} signalement(s) fondé(s) ou {false} fausse(s) déclaration(s) de droits\u{a0}: il ne peut plus déposer ni créer de cadeau. Une décision annulée sur réclamation ne compte plus. En cas de récidive après suspension, le compte est fermé.",
        "An account is suspended after {upheld} upheld notice(s) or {false} false rights declaration(s): it can no longer upload or create gifts. A decision reversed on complaint no longer counts. On recurrence after suspension, the account is closed.",
        "Ein Konto wird nach {upheld} begründeten Meldung(en) oder {false} falschen Rechteerklärung(en) gesperrt: Es kann nichts mehr hochladen und keine Geschenke anlegen. Eine auf Beschwerde aufgehobene Entscheidung zählt nicht mehr. Bei Wiederholung nach der Sperre wird das Konto geschlossen.",
        "Una cuenta se suspende tras {upheld} denuncia(s) fundada(s) o {false} declaración(es) de derechos falsa(s): ya no puede subir ni crear regalos. Una decisión anulada tras reclamación deja de contar. En caso de reincidencia tras la suspensión, la cuenta se cierra.",
        "Un account è sospeso dopo {upheld} segnalazione/i fondata/e o {false} dichiarazione/i dei diritti falsa/e: non può più caricare né creare regali. Una decisione annullata su reclamo non conta più. In caso di recidiva dopo la sospensione, l’account è chiuso.",
        "Een account wordt geschorst na {upheld} gegronde melding(en) of {false} valse rechtenverklaring(en): het kan dan niets meer uploaden of cadeaus maken. Een na klacht herroepen beslissing telt niet meer. Bij herhaling na de schorsing wordt het account gesloten.",
        "Uma conta é suspensa após {upheld} denúncia(s) procedente(s) ou {false} declaração(ões) de direitos falsa(s): deixa de poder carregar ou criar ofertas. Uma decisão anulada por reclamação deixa de contar. Em caso de reincidência após a suspensão, a conta é encerrada.",
    ];
    /// Terms: personal data.
    TermsDataHead => ["Données personnelles", "Personal data", "Personenbezogene Daten", "Datos personales", "Dati personali", "Persoonsgegevens", "Dados pessoais"];
    /// Terms: personal data (text).
    TermsData => [
        "Comptes\u{a0}: adresse e-mail vérifiée et données d’identification, conservées tant que le compte existe puis {identity} jours\u{a0}; adresse IP des dépôts et créations de cadeaux, {connection} jours\u{a0}; déclarations de droits, {declaration} jours après l’effacement du fichier\u{a0}; signalements et décisions, {notices} jours. Remises\u{a0}: aucun nom ni adresse IP\u{a0}; un pseudonyme du réseau client est gardé {security} jours. Fichiers\u{a0}: effacés à la suppression du livre, au retrait, à la fermeture du compte, ou {idle} jours après l’annulation de son dernier cadeau. Aucun traceur, aucun cookie autre que celui de la session. Durées à confirmer avec un conseil.",
        "Accounts: verified e-mail address and identification data, kept while the account exists, then {identity} days; IP address of uploads and gift creations, {connection} days; rights declarations, {declaration} days after the file is erased; notices and decisions, {notices} days. Deliveries: no name and no IP address; a pseudonym of the client network is kept {security} days. Files: erased when the book is deleted, taken down or its account closed, or {idle} days after its last gift was cancelled. No tracker, no cookie except the session one. Periods to be confirmed with counsel.",
        "Konten: geprüfte E-Mail-Adresse und Identifikationsdaten, aufbewahrt solange das Konto besteht, danach {identity} Tage; IP-Adresse von Uploads und Geschenkanlagen, {connection} Tage; Rechteerklärungen, {declaration} Tage nach Löschung der Datei; Meldungen und Entscheidungen, {notices} Tage. Auslieferungen: kein Name, keine IP-Adresse; ein Pseudonym des Client-Netzes wird {security} Tage aufbewahrt. Dateien: gelöscht beim Löschen oder Entfernen des Buches, beim Schließen des Kontos oder {idle} Tage nach Stornierung des letzten Geschenks. Keine Tracker, kein Cookie außer dem Sitzungscookie. Fristen mit Rechtsbeistand zu bestätigen.",
        "Cuentas: correo verificado y datos de identificación, conservados mientras exista la cuenta y después {identity} días; dirección IP de las subidas y creaciones de regalos, {connection} días; declaraciones de derechos, {declaration} días tras borrar el archivo; denuncias y decisiones, {notices} días. Entregas: ningún nombre ni dirección IP; se guarda {security} días un seudónimo de la red del cliente. Archivos: se borran al eliminar o retirar el libro, al cerrar la cuenta o {idle} días después de anular su último regalo. Ningún rastreador, ninguna cookie salvo la de sesión. Plazos por confirmar con un abogado.",
        "Account: e-mail verificata e dati di identificazione, conservati finché esiste l’account, poi {identity} giorni; indirizzo IP dei caricamenti e delle creazioni di regali, {connection} giorni; dichiarazioni dei diritti, {declaration} giorni dopo la cancellazione del file; segnalazioni e decisioni, {notices} giorni. Consegne: nessun nome né indirizzo IP; uno pseudonimo della rete del client è conservato {security} giorni. File: cancellati alla cancellazione o rimozione del libro, alla chiusura dell’account o {idle} giorni dopo l’annullamento dell’ultimo regalo. Nessun tracciatore, nessun cookie oltre a quello di sessione. Durate da confermare con un legale.",
        "Accounts: geverifieerd e-mailadres en identificatiegegevens, bewaard zolang het account bestaat en daarna {identity} dagen; IP-adres van uploads en cadeaus, {connection} dagen; rechtenverklaringen, {declaration} dagen na het wissen van het bestand; meldingen en beslissingen, {notices} dagen. Bezorging: geen naam en geen IP-adres; een pseudoniem van het clientnetwerk wordt {security} dagen bewaard. Bestanden: gewist bij verwijdering van het boek, na een melding, bij sluiting van het account, of {idle} dagen na annulering van het laatste cadeau. Geen trackers, geen cookie behalve de sessiecookie. Termijnen te bevestigen met een jurist.",
        "Contas: e-mail verificado e dados de identificação, conservados enquanto a conta existir e depois {identity} dias; endereço IP dos carregamentos e criações de ofertas, {connection} dias; declarações de direitos, {declaration} dias após o ficheiro ser apagado; denúncias e decisões, {notices} dias. Entregas: nenhum nome nem endereço IP; um pseudónimo da rede do cliente é guardado {security} dias. Ficheiros: apagados quando o livro é eliminado ou removido, quando a conta é encerrada, ou {idle} dias após a anulação da última oferta. Nenhum rastreador, nenhum cookie além do de sessão. Prazos a confirmar com um advogado.",
    ];
    /// Legal notice title.
    LegalTitle => ["Mentions légales", "Legal notice", "Impressum", "Aviso legal", "Note legali", "Colofon", "Informação legal"];
    /// Operator heading.
    LegalOperator => ["Éditeur du service", "Service operator", "Betreiber", "Responsable del servicio", "Gestore del servizio", "Exploitant van de dienst", "Responsável pelo serviço"];
    /// Director of publication.
    LegalDirector => ["Directeur ou directrice de la publication", "Director of publication", "Verantwortlich für den Inhalt", "Director de la publicación", "Direttore della pubblicazione", "Verantwoordelijke uitgever", "Diretor da publicação"];
    /// Host heading.
    LegalHost => ["Hébergeur", "Host", "Hosting-Anbieter", "Proveedor de alojamiento", "Fornitore di hosting", "Hostingprovider", "Fornecedor de alojamento"];
    /// To complete.
    ToComplete => ["[à compléter]", "[to be completed]", "[zu ergänzen]", "[por completar]", "[da completare]", "[aan te vullen]", "[a completar]"];
    /// Contact title.
    ContactTitle => ["Point de contact", "Point of contact", "Kontaktstelle", "Punto de contacto", "Punto di contatto", "Contactpunt", "Ponto de contacto"];
    /// Contact text.
    ContactText => [
        "Point de contact unique pour les autorités des États membres, la Commission et le comité européen des services numériques (DSA, art. 11) et pour les destinataires du service (art. 12)\u{a0}: {email}. Langues\u{a0}: français, anglais.",
        "Single point of contact for Member States’ authorities, the Commission and the European Board for Digital Services (DSA Art. 11) and for recipients of the service (Art. 12): {email}. Languages: French, English.",
        "Zentrale Kontaktstelle für Behörden der Mitgliedstaaten, die Kommission und das Europäische Gremium für digitale Dienste (DSA Art. 11) sowie für Nutzer des Dienstes (Art. 12): {email}. Sprachen: Französisch, Englisch.",
        "Punto de contacto único para las autoridades de los Estados miembros, la Comisión y la Junta Europea de Servicios Digitales (DSA, art. 11) y para los destinatarios del servicio (art. 12): {email}. Idiomas: francés, inglés.",
        "Punto di contatto unico per le autorità degli Stati membri, la Commissione e il comitato europeo per i servizi digitali (DSA, art. 11) e per i destinatari del servizio (art. 12): {email}. Lingue: francese, inglese.",
        "Centraal contactpunt voor de autoriteiten van de lidstaten, de Commissie en de Europese raad voor digitale diensten (DSA art. 11) en voor afnemers van de dienst (art. 12): {email}. Talen: Frans, Engels.",
        "Ponto de contacto único para as autoridades dos Estados-Membros, a Comissão e o Comité Europeu dos Serviços Digitais (DSA, art. 11) e para os destinatários do serviço (art. 12): {email}. Línguas: francês, inglês.",
    ];
    /// Contact missing.
    ContactMissing => ["Le point de contact n’est pas encore configuré.", "The point of contact is not configured yet.", "Die Kontaktstelle ist noch nicht eingerichtet.", "El punto de contacto aún no está configurado.", "Il punto di contatto non è ancora configurato.", "Het contactpunt is nog niet ingesteld.", "O ponto de contacto ainda não está configurado."];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_string_exists_in_every_language_with_the_same_placeholders() {
        let placeholders = |text: &str| {
            let mut found: Vec<String> = text
                .split('{')
                .skip(1)
                .filter_map(|rest| rest.split_once('}').map(|(name, _)| name.to_owned()))
                .collect();
            found.sort();
            found
        };
        for msg in S::ALL {
            let texts = msg.texts();
            let reference = placeholders(texts[0]);
            for (index, text) in texts.iter().enumerate() {
                assert!(!text.trim().is_empty(), "{msg:?} is empty in locale {index}");
                assert_eq!(placeholders(text), reference, "{msg:?} placeholders in locale {index}");
            }
        }
        assert_eq!(
            sf(Locale::En, S::TooMany, &[("minutes", "5")]),
            "Too many requests. Try again in 5 minutes."
        );
    }

    #[test]
    fn french_puts_a_no_break_space_before_double_punctuation() {
        for msg in S::ALL {
            let french = msg.texts()[0];
            for mark in [" ?", " !", " :", " ;"] {
                assert!(!french.contains(mark), "{msg:?}: use U+00A0 before {mark:?}");
            }
        }
    }

    #[test]
    fn no_wording_presents_the_shelf_as_a_way_to_pass_on_bought_books() {
        // Research 09, implication 1: never "share", "lend" or "resell" an eBook.
        for msg in S::ALL {
            for text in msg.texts() {
                let lower = text.to_lowercase();
                for word in ["partager", "prêter", "revendre", "share your", "lend", "resell"] {
                    assert!(!lower.contains(word), "{msg:?}: {text}");
                }
            }
        }
    }
}
