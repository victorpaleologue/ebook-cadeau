//! Send-to-device: e-mailing the hosted file to a device address such as
//! `…@kindle.com` (`POST /k/{code}/send`, `docs/api.md` §2).
//!
//! The address is the only user input that reaches SMTP. It is parsed with a strict
//! grammar (no display name, no quoting, no whitespace or control character, ASCII only)
//! and its domain must be one of the configured device domains, so the form can be used
//! neither to inject headers nor to mail arbitrary people.

use std::time::{Duration, UNIX_EPOCH};

use lettre::message::header::ContentType;
use lettre::message::{Attachment, Mailbox, MultiPart, SinglePart};
use lettre::{Address, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

/// Why a file could not be sent.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub(crate) enum MailError {
    /// `EBC_SEND_FROM` or `EBC_SMTP_URL` is unusable.
    #[error("mail configuration: {0}")]
    Config(String),
    /// The message could not be built.
    #[error("message: {0}")]
    Message(String),
    /// The SMTP server refused or is unreachable.
    #[error("smtp: {0}")]
    Transport(String),
}

impl MailError {
    /// A word for logs that cannot contain the recipient's address (SMTP error texts
    /// sometimes quote it, and the address is deliberately not kept anywhere).
    pub(crate) fn kind(&self) -> &'static str {
        match self {
            Self::Config(_) => "configuration",
            Self::Message(_) => "message",
            Self::Transport(_) => "transport",
        }
    }
}

/// Parses a device address typed by a recipient. `None` unless it is a plain
/// `local@domain` whose domain is one of `allowed_domains` (lower case).
pub(crate) fn parse_device_address(input: &str, allowed_domains: &[String]) -> Option<Address> {
    let input = input.trim();
    if input.len() > 254 || !input.is_ascii() {
        return None;
    }
    let (local, domain) = input.rsplit_once('@')?;
    let local_ok = (1..=64).contains(&local.len())
        && local
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-' | b'+'))
        && !local.starts_with('.')
        && !local.ends_with('.')
        && !local.contains("..");
    let domain = domain.to_ascii_lowercase();
    if !local_ok || !allowed_domains.contains(&domain) {
        return None;
    }
    Address::new(local, domain).ok()
}

/// One file to send.
pub(crate) struct Outgoing<'a> {
    /// `EBC_SMTP_URL`.
    pub(crate) smtp_url: &'a str,
    /// `EBC_SEND_FROM`.
    pub(crate) from: &'a str,
    /// The validated device address.
    pub(crate) to: Address,
    /// Subject line (ASCII).
    pub(crate) subject: String,
    /// Plain-text body.
    pub(crate) text: String,
    /// Attachment name (ASCII, no comma: Send-to-PocketBook refuses other names).
    pub(crate) attachment_name: String,
    /// Attachment media type.
    pub(crate) media_type: &'a str,
    /// The file.
    pub(crate) bytes: Vec<u8>,
    /// Value of the `Date` header (injected, not read from the wall clock).
    pub(crate) now_unix: i64,
    /// Random token for the `Message-ID` (injected).
    pub(crate) message_token: String,
}

/// Builds the message: `From`, `To`, `Subject`, `Date`, `Message-ID`, a text part and the
/// attachment.
pub(crate) fn build_message(outgoing: Outgoing<'_>) -> Result<Message, MailError> {
    let from: Mailbox =
        outgoing.from.parse().map_err(|e| MailError::Config(format!("EBC_SEND_FROM: {e}")))?;
    let content_type = ContentType::parse(outgoing.media_type)
        .or_else(|_| ContentType::parse("application/octet-stream"))
        .map_err(|e| MailError::Message(e.to_string()))?;
    let date = UNIX_EPOCH + Duration::from_secs(u64::try_from(outgoing.now_unix).unwrap_or(0));
    let message_id = format!("<{}@{}>", outgoing.message_token, from.email.domain());
    Message::builder()
        .from(from)
        .to(Mailbox::new(None, outgoing.to))
        .subject(outgoing.subject)
        .date(date)
        .message_id(Some(message_id))
        .multipart(MultiPart::mixed().singlepart(SinglePart::plain(outgoing.text)).singlepart(
            Attachment::new(outgoing.attachment_name).body(outgoing.bytes, content_type),
        ))
        .map_err(|e| MailError::Message(e.to_string()))
}

/// Sends the file through the configured SMTP server (`smtp://host:port` is plain SMTP
/// for local relays and sinks; `smtps://…` and `smtp://…?tls=required` use TLS).
pub(crate) async fn send(outgoing: Outgoing<'_>, timeout: Duration) -> Result<(), MailError> {
    let transport = AsyncSmtpTransport::<Tokio1Executor>::from_url(outgoing.smtp_url)
        .map_err(|e| MailError::Config(format!("EBC_SMTP_URL: {e}")))?
        .timeout(Some(timeout))
        .build::<Tokio1Executor>();
    let message = build_message(outgoing)?;
    transport.send(message).await.map(|_| ()).map_err(|e| MailError::Transport(e.to_string()))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn domains() -> Vec<String> {
        vec!["kindle.com".to_owned(), "pbsync.com".to_owned()]
    }

    #[test]
    fn device_addresses_on_allowed_domains_are_accepted() {
        for ok in ["sacha_42@kindle.com", " Sacha.Lit+ebook@KINDLE.COM ", "a@pbsync.com"] {
            let address = parse_device_address(ok, &domains()).unwrap_or_else(|| panic!("{ok}"));
            assert!(domains().iter().any(|d| d == address.domain()), "{ok}");
        }
    }

    #[test]
    fn everything_else_is_refused() {
        let long_local = format!("{}@kindle.com", "a".repeat(65));
        let hostile = [
            "",
            "kindle.com",
            "@kindle.com",
            "sacha@gmail.com",
            "sacha@kindle.com.evil.example",
            "sacha@evilkindle.com",
            "sacha@kindle.com\r\nBcc: victim@example.org",
            "sacha@kindle.com\nSubject: x",
            "sacha\r\n@kindle.com",
            "sacha@kindle.com, victim@example.org",
            "victim@example.org, sacha@kindle.com",
            "Sacha <sacha@kindle.com>",
            "\"sacha@example.org\"@kindle.com",
            "sacha@example.org@kindle.com",
            "sa cha@kindle.com",
            "sacha@kindle.com ; rm -rf",
            ".sacha@kindle.com",
            "sa..cha@kindle.com",
            "sachä@kindle.com",
            "sacha@kindle.com\u{0}",
            "sacha%example.org@kindle.com",
            "sacha!victim@kindle.com",
            long_local.as_str(),
        ];
        for bad in hostile {
            assert!(parse_device_address(bad, &domains()).is_none(), "{bad:?}");
        }
        assert!(parse_device_address("a@kindle.com", &[]).is_none());
    }

    #[test]
    fn the_message_is_deterministic_apart_from_the_mime_boundary() {
        let outgoing = || Outgoing {
            smtp_url: "smtp://127.0.0.1:2525",
            from: "livraison@localhost",
            to: parse_device_address("sacha@kindle.com", &domains()).unwrap(),
            subject: "Le-Horla".to_owned(),
            text: "Voici votre livre.".to_owned(),
            attachment_name: "Le-Horla.epub".to_owned(),
            media_type: "application/epub+zip",
            bytes: b"PK\x03\x04 pretend".to_vec(),
            now_unix: 1_794_441_600,
            message_token: "abc123".to_owned(),
        };
        let text = String::from_utf8(build_message(outgoing()).unwrap().formatted()).unwrap();
        assert!(text.contains("From: livraison@localhost\r\n"), "{text}");
        assert!(text.contains("To: sacha@kindle.com\r\n"), "{text}");
        assert!(text.contains("Subject: Le-Horla\r\n"), "{text}");
        assert!(text.contains("Message-ID: <abc123@localhost>\r\n"), "{text}");
        assert!(text.contains("Date: Thu, 12 Nov 2026 00:00:00 +0000\r\n"), "{text}");
        assert!(text.contains("Content-Type: application/epub+zip"), "{text}");
        assert!(text.contains("filename=\"Le-Horla.epub\""), "{text}");

        let mut bad_sender = outgoing();
        bad_sender.from = "not an address";
        assert!(matches!(build_message(bad_sender), Err(MailError::Config(_))));
    }
}
