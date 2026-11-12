//! Outgoing mail: sign-in links, notice acknowledgements, decisions and statements of
//! reasons. Plain text only; addresses are validated before they get here
//! ([`crate::config::normalise_email`]), so nothing can inject a header.

use std::future::Future;
use std::pin::Pin;
use std::sync::Mutex;
use std::time::{Duration, UNIX_EPOCH};

use lettre::message::Mailbox;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

/// A mail to send.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutgoingMail {
    /// Recipient (a validated plain address).
    pub to: String,
    /// Subject line.
    pub subject: String,
    /// Plain-text body.
    pub text: String,
}

/// Why a mail could not be sent.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum MailError {
    /// No SMTP server is configured.
    #[error("no SMTP server is configured (EBC_SMTP_URL)")]
    NotConfigured,
    /// The configuration is unusable.
    #[error("mail configuration: {0}")]
    Config(String),
    /// The message could not be built.
    #[error("mail message: {0}")]
    Message(String),
    /// The SMTP server refused or could not be reached.
    #[error("mail transport: {0}")]
    Transport(String),
}

/// The future a [`Mailer`] returns.
pub type MailFuture<'a> = Pin<Box<dyn Future<Output = Result<(), MailError>> + Send + 'a>>;

/// Sends mail.
pub trait Mailer: Send + Sync + 'static {
    /// Sends one mail. `now_unix` dates it.
    fn send(&self, mail: OutgoingMail, now_unix: i64) -> MailFuture<'_>;
}

/// SMTP through `lettre` (`smtp://host:port` plain for a local relay or sink,
/// `smtps://…` or `smtp://…?tls=required` with TLS).
#[derive(Debug, Clone)]
pub struct SmtpMailer {
    url: String,
    from: String,
    timeout: Duration,
}

impl SmtpMailer {
    /// A mailer for `url`, sending as `from`.
    #[must_use]
    pub fn new(url: &str, from: &str, timeout: Duration) -> Self {
        Self { url: url.to_owned(), from: from.to_owned(), timeout }
    }
}

/// Builds the message (`From`, `To`, `Subject`, `Date`, text body).
///
/// # Errors
/// [`MailError::Config`] for an unusable sender, [`MailError::Message`] otherwise.
pub fn build_message(from: &str, mail: &OutgoingMail, now_unix: i64) -> Result<Message, MailError> {
    let from: Mailbox = from.parse().map_err(|e| MailError::Config(format!("sender: {e}")))?;
    let to: Mailbox = mail.to.parse().map_err(|e| MailError::Message(format!("recipient: {e}")))?;
    let date = UNIX_EPOCH + Duration::from_secs(u64::try_from(now_unix).unwrap_or(0));
    Message::builder()
        .from(from)
        .to(to)
        .subject(mail.subject.replace(['\r', '\n'], " "))
        .date(date)
        .body(mail.text.clone())
        .map_err(|e| MailError::Message(e.to_string()))
}

impl Mailer for SmtpMailer {
    fn send(&self, mail: OutgoingMail, now_unix: i64) -> MailFuture<'_> {
        Box::pin(async move {
            let transport = AsyncSmtpTransport::<Tokio1Executor>::from_url(&self.url)
                .map_err(|e| MailError::Config(format!("EBC_SMTP_URL: {e}")))?
                .timeout(Some(self.timeout))
                .build::<Tokio1Executor>();
            let message = build_message(&self.from, &mail, now_unix)?;
            transport
                .send(message)
                .await
                .map(|_| ())
                .map_err(|e| MailError::Transport(e.to_string()))
        })
    }
}

/// No SMTP server: every sending fails with [`MailError::NotConfigured`].
#[derive(Debug, Clone, Copy, Default)]
pub struct NoMailer;

impl Mailer for NoMailer {
    fn send(&self, _mail: OutgoingMail, _now_unix: i64) -> MailFuture<'_> {
        Box::pin(async { Err(MailError::NotConfigured) })
    }
}

/// Keeps every mail in memory (tests).
#[derive(Debug, Default)]
pub struct MemoryMailer {
    sent: Mutex<Vec<OutgoingMail>>,
}

impl MemoryMailer {
    /// An empty outbox.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Every mail sent so far, oldest first.
    #[must_use]
    pub fn sent(&self) -> Vec<OutgoingMail> {
        self.sent.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone()
    }

    /// Mails sent to `to`, oldest first.
    #[must_use]
    pub fn sent_to(&self, to: &str) -> Vec<OutgoingMail> {
        self.sent().into_iter().filter(|m| m.to == to).collect()
    }
}

impl Mailer for MemoryMailer {
    fn send(&self, mail: OutgoingMail, _now_unix: i64) -> MailFuture<'_> {
        self.sent.lock().unwrap_or_else(std::sync::PoisonError::into_inner).push(mail);
        Box::pin(async { Ok(()) })
    }
}

impl<M: Mailer> Mailer for std::sync::Arc<M> {
    fn send(&self, mail: OutgoingMail, now_unix: i64) -> MailFuture<'_> {
        self.as_ref().send(mail, now_unix)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn messages_carry_no_injected_header() {
        let mail = OutgoingMail {
            to: "autrice@exemple.fr".to_owned(),
            subject: "Lien\r\nBcc: x@y.fr".to_owned(),
            text: "Bonjour".to_owned(),
        };
        let raw = build_message("etagere@exemple.fr", &mail, 1_794_441_600).unwrap().formatted();
        let text = String::from_utf8_lossy(&raw);
        assert!(!text.contains("\r\nBcc:"), "{text}");
        assert!(text.contains("To: autrice@exemple.fr"));
        let bad = OutgoingMail { to: "not an address".to_owned(), ..mail };
        assert!(build_message("etagere@exemple.fr", &bad, 0).is_err());
    }
}
