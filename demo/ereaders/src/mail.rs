//! `MailDevice`: "send to my device by e-mail" (Send to Kindle, Send-to-PocketBook).
//!
//! A tiny in-process SMTP server stands for the vendor's mail gateway; behind it sit the
//! service rules and the device's [`Library`]:
//!
//! * the message must be addressed to the device's address;
//! * mail from a sender that is **not on the approved list is accepted at SMTP level and
//!   parked** — the sending server sees `250 OK` and learns nothing, exactly the trap the
//!   research describes (`ereader-ux.devices.md` §3.1 \[V1\]; §7 lesson 9);
//! * mail from an approved sender has its attachments extracted and, if the service takes the
//!   format and the file is sound, added to the library — for Send to Kindle an EPUB is
//!   labelled as converted (§3.1 \[V2\]); **no real conversion happens**;
//! * the service's size limit is enforced during `DATA`.
//!
//! *Assumptions*: which address the real services check is not documented; the simulation
//! checks the `From:` header and falls back to the SMTP envelope sender. Approving a sender
//! does not release mail parked earlier — call [`MailDevice::release`] — because the note does
//! not say what the real services do.
//!
//! The SMTP server speaks `EHLO/HELO`, `MAIL FROM`, `RCPT TO`, `DATA` (with dot-unstuffing),
//! `RSET`, `NOOP`, `VRFY`, `HELP` and `QUIT`; there is **no TLS and no authentication**. It
//! binds `127.0.0.1` on an ephemeral port and is meant for tests only. Every read has an idle
//! timeout, every session a total deadline, and line and message sizes are capped.

use std::net::SocketAddr;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use mail_parser::{MessageParser, MimeHeaders, PartType};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;
use tokio::task::{JoinHandle, JoinSet};

use crate::error::{ImportError, MailError};
use crate::format::FileFormat;
use crate::library::{self, ImportSource, IncomingFile, Library, LibraryEntry};
use crate::profile::{DeviceProfile, MailServiceSpec};

/// Longest SMTP command line accepted.
const MAX_COMMAND_LINE: usize = 2048;
/// Longest line accepted inside `DATA` (RFC 5322 allows 998; be generous but bounded).
const MAX_DATA_LINE: usize = 64 * 1024;
/// Most attachments processed per message (Send to Kindle documents 25 — \[M\]).
const MAX_ATTACHMENTS: usize = 25;
/// Most held mails and delivery records kept.
const MAX_RECORDS: usize = 1000;

/// Limits of the SMTP listener.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailDeviceConfig {
    /// Longest silence tolerated from a client.
    pub idle_timeout: Duration,
    /// Longest whole session.
    pub session_timeout: Duration,
    /// Concurrent sessions; further connections get `421`.
    pub max_sessions: usize,
    /// Protocol errors tolerated before the session is closed.
    pub max_errors: u32,
    /// Host name used in SMTP replies.
    pub hostname: String,
}

impl Default for MailDeviceConfig {
    fn default() -> Self {
        Self {
            idle_timeout: Duration::from_secs(10),
            session_timeout: Duration::from_secs(60),
            max_sessions: 8,
            max_errors: 10,
            hostname: "mail-device.demo.invalid".to_owned(),
        }
    }
}

/// Why a message is parked instead of delivered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoldReason {
    /// The sender is not on the device's approved list.
    SenderNotApproved,
    /// The service wants the account owner to click a confirmation e-mail first.
    AwaitingConfirmation,
}

/// What happened to one attachment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachmentOutcome {
    /// Attachment file name.
    pub file_name: String,
    /// The library entry, or why the service / device refused the file.
    pub result: Result<LibraryEntry, ImportError>,
}

/// What the service did with a message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MailOutcome {
    /// Parked; nothing reached the device.
    Held(HoldReason),
    /// Attachments were processed (each may have succeeded or failed).
    Processed(Vec<AttachmentOutcome>),
    /// The message carried no attachment.
    NoAttachment,
    /// The message could not be parsed as e-mail.
    Unparsable,
}

/// One message received by the gateway.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailRecord {
    /// Sequence number, starting at 1.
    pub id: u64,
    /// Address the approval check used (`From:` header, else the envelope sender), lower-case.
    pub from: String,
    /// SMTP envelope sender (`MAIL FROM`), lower-case; empty for the null sender.
    pub envelope_from: String,
    /// `Subject:` header.
    pub subject: String,
    /// Size of the message as received.
    pub size_bytes: usize,
    /// What happened.
    pub outcome: MailOutcome,
}

impl MailRecord {
    /// Books that reached the library from this message.
    pub fn delivered(&self) -> Vec<&LibraryEntry> {
        match &self.outcome {
            MailOutcome::Processed(items) => {
                items.iter().filter_map(|a| a.result.as_ref().ok()).collect()
            }
            _ => Vec::new(),
        }
    }
}

struct Held {
    id: u64,
    raw: Vec<u8>,
}

#[derive(Default)]
struct State {
    approved: Vec<String>,
    library: Library,
    held: Vec<Held>,
    records: Vec<MailRecord>,
    next_id: u64,
    mailboxes: Vec<String>,
    inbox: Vec<MailboxMessage>,
}

/// A message received for a plain mailbox (see [`MailDevice::add_mailbox`]): not a
/// device delivery, just a mail someone reads — e.g. a sign-in link sent by a shelf.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailboxMessage {
    /// Sequence number, shared with [`MailRecord::id`].
    pub id: u64,
    /// The mailbox address it was delivered to (lower-case).
    pub to: String,
    /// `From:` header (else the envelope sender), lower-case.
    pub from: String,
    /// `Subject:` header.
    pub subject: String,
    /// The first text part of the body.
    pub text: String,
}

/// Whether `address` is one of `mailboxes` (a full address, or `@domain`).
fn is_mailbox(mailboxes: &[String], address: &str) -> bool {
    mailboxes.iter().any(|entry| {
        entry == address || (entry.starts_with('@') && address.ends_with(entry.as_str()))
    })
}

struct Shared {
    profile: DeviceProfile,
    service: MailServiceSpec,
    address: String,
    config: MailDeviceConfig,
    state: Mutex<State>,
    sessions: Arc<Semaphore>,
}

impl Shared {
    fn state(&self) -> MutexGuard<'_, State> {
        // A poisoned lock only means another session panicked; the data is still usable.
        self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// A device reachable through a mail-to-device service, with its SMTP gateway.
pub struct MailDevice {
    addr: SocketAddr,
    shared: Arc<Shared>,
    accept_task: JoinHandle<()>,
}

impl std::fmt::Debug for MailDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MailDevice")
            .field("smtp", &self.addr)
            .field("address", &self.shared.address)
            .field("service", &self.shared.service.id)
            .finish_non_exhaustive()
    }
}

impl Drop for MailDevice {
    fn drop(&mut self) {
        // Aborting the accept task drops its JoinSet, which aborts every open session.
        self.accept_task.abort();
    }
}

impl MailDevice {
    /// Starts the gateway of `profile`'s own mail service for the device address `address`
    /// (e.g. `sacha_42@kindle.com`). Must be called inside a tokio runtime.
    pub async fn start(profile: DeviceProfile, address: &str) -> Result<Self, MailError> {
        let service =
            profile.mail_service.ok_or(MailError::NoMailService { device: profile.id })?;
        Self::start_with(profile, service, address, MailDeviceConfig::default()).await
    }

    /// Starts a gateway with an explicit service definition and limits.
    pub async fn start_with(
        profile: DeviceProfile,
        service: MailServiceSpec,
        address: &str,
        config: MailDeviceConfig,
    ) -> Result<Self, MailError> {
        let address = address.trim().to_ascii_lowercase();
        let domain_ok = address
            .rsplit_once('@')
            .is_some_and(|(local, domain)| !local.is_empty() && service.domains.contains(&domain));
        if !domain_ok || address.chars().any(|c| c.is_whitespace() || c == '<' || c == '>') {
            return Err(MailError::BadDeviceAddress { address, service: service.name });
        }
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.map_err(MailError::Bind)?;
        let addr = listener.local_addr().map_err(MailError::Bind)?;
        let shared = Arc::new(Shared {
            profile,
            service,
            address,
            sessions: Arc::new(Semaphore::new(config.max_sessions.max(1))),
            config,
            state: Mutex::new(State::default()),
        });
        let accept_shared = Arc::clone(&shared);
        let accept_task = tokio::spawn(async move {
            let mut sessions = JoinSet::new();
            loop {
                tokio::select! {
                    accepted = listener.accept() => {
                        let Ok((stream, _peer)) = accepted else {
                            tokio::time::sleep(Duration::from_millis(20)).await;
                            continue;
                        };
                        let shared = Arc::clone(&accept_shared);
                        match Arc::clone(&shared.sessions).try_acquire_owned() {
                            Ok(permit) => {
                                sessions.spawn(async move {
                                    let deadline = shared.config.session_timeout;
                                    let _ = tokio::time::timeout(deadline, session(stream, &shared)).await;
                                    drop(permit);
                                });
                            }
                            Err(_) => {
                                sessions.spawn(async move {
                                    let mut stream = stream;
                                    let _ = tokio::time::timeout(
                                        Duration::from_secs(2),
                                        stream.write_all(b"421 4.3.2 too many connections, try later\r\n"),
                                    )
                                    .await;
                                });
                            }
                        }
                    }
                    // Reap finished sessions so the set does not grow without bound.
                    Some(_) = sessions.join_next(), if !sessions.is_empty() => {}
                }
            }
        });
        Ok(Self { addr, shared, accept_task })
    }

    /// Socket address of the SMTP gateway (`127.0.0.1:port`).
    pub fn smtp_addr(&self) -> SocketAddr {
        self.addr
    }

    /// The gateway as an `smtp://host:port` URL (the shape of `EBC_SMTP_URL`, docs/api.md §6).
    pub fn smtp_url(&self) -> String {
        format!("smtp://{}", self.addr)
    }

    /// The device's e-mail address.
    pub fn address(&self) -> &str {
        &self.shared.address
    }

    /// The service definition in force.
    pub fn service(&self) -> &MailServiceSpec {
        &self.shared.service
    }

    /// Adds a sender to the approved list: a full address, or a domain (`example.org` or
    /// `@example.org`) to approve every sender of that domain. Mail parked earlier stays
    /// parked (see [`MailDevice::release`]).
    pub fn approve_sender(&self, sender: &str) {
        let sender = sender.trim().to_ascii_lowercase();
        let mut state = self.shared.state();
        if !sender.is_empty() && !state.approved.contains(&sender) && state.approved.len() < 1000 {
            state.approved.push(sender);
        }
    }

    /// The approved senders.
    pub fn approved_senders(&self) -> Vec<String> {
        self.shared.state().approved.clone()
    }

    /// A snapshot of the device's library.
    pub fn library(&self) -> Library {
        self.shared.state().library.clone()
    }

    /// Every message received so far, oldest first.
    pub fn records(&self) -> Vec<MailRecord> {
        self.shared.state().records.clone()
    }

    /// Messages currently parked.
    pub fn held(&self) -> Vec<MailRecord> {
        let state = self.shared.state();
        state.records.iter().filter(|r| state.held.iter().any(|h| h.id == r.id)).cloned().collect()
    }

    /// Also accepts mail for `address` (a full address, or `@domain` for every address of
    /// a domain) as a **plain mailbox**: such messages are kept as text
    /// ([`MailDevice::mailbox`]) and never reach the device. For the demo shelf's
    /// sign-in links and notices; the device rules are untouched.
    pub fn add_mailbox(&self, address: &str) {
        let address = address.trim().to_ascii_lowercase();
        let mut state = self.shared.state();
        if !address.is_empty() && !state.mailboxes.contains(&address) && state.mailboxes.len() < 100
        {
            state.mailboxes.push(address);
        }
    }

    /// Messages received for the mailbox `address`, oldest first (every mailbox when
    /// `address` is empty).
    pub fn mailbox(&self, address: &str) -> Vec<MailboxMessage> {
        let address = address.trim().to_ascii_lowercase();
        self.shared
            .state()
            .inbox
            .iter()
            .filter(|m| address.is_empty() || m.to == address)
            .cloned()
            .collect()
    }

    /// Delivers a parked message as if the account owner had approved it (the "verify this
    /// sender" / "confirm this delivery" click). Returns the updated record.
    pub fn release(&self, id: u64) -> Result<MailRecord, MailError> {
        let mut state = self.shared.state();
        let position =
            state.held.iter().position(|h| h.id == id).ok_or(MailError::UnknownMail(id))?;
        let held = state.held.remove(position);
        let outcome = process(&self.shared, &mut state, &held.raw);
        let record =
            state.records.iter_mut().find(|r| r.id == id).ok_or(MailError::UnknownMail(id))?;
        record.outcome = outcome;
        Ok(record.clone())
    }
}

/// Whether `sender` is covered by an approved-list entry.
fn is_approved(approved: &[String], sender: &str) -> bool {
    let sender = sender.to_ascii_lowercase();
    let domain = sender.rsplit_once('@').map(|(_, d)| d);
    approved.iter().any(|entry| {
        if entry.contains('@') && !entry.starts_with('@') {
            *entry == sender
        } else {
            domain == Some(entry.trim_start_matches('@'))
        }
    })
}

/// Handles one complete message. Called with the state lock held (no I/O happens here).
fn receive(shared: &Shared, envelope_from: &str, raw: Vec<u8>) {
    let mut state = shared.state();
    state.next_id += 1;
    let id = state.next_id;
    let (header_from, subject) = {
        let parsed = MessageParser::default().parse(&raw);
        let header_from = parsed
            .as_ref()
            .and_then(|m| m.from())
            .and_then(|a| a.first())
            .and_then(|a| a.address())
            .map(|a| a.trim().to_ascii_lowercase())
            .filter(|a| !a.is_empty());
        let subject: String =
            parsed.as_ref().and_then(|m| m.subject()).unwrap_or("").chars().take(300).collect();
        (header_from, subject)
    };
    let from = header_from.unwrap_or_else(|| envelope_from.to_owned());
    let size_bytes = raw.len();
    let hold = if !is_approved(&state.approved, &from) {
        Some(HoldReason::SenderNotApproved)
    } else if shared.service.confirmation_click {
        Some(HoldReason::AwaitingConfirmation)
    } else {
        None
    };
    let outcome = match hold {
        Some(reason) => {
            if state.held.len() < MAX_RECORDS {
                state.held.push(Held { id, raw });
            }
            MailOutcome::Held(reason)
        }
        None => process(shared, &mut state, &raw),
    };
    if state.records.len() < MAX_RECORDS {
        state.records.push(MailRecord {
            id,
            from,
            envelope_from: envelope_from.to_owned(),
            subject,
            size_bytes,
            outcome,
        });
    }
}

/// Keeps a message for plain mailboxes as text.
fn to_mailboxes(shared: &Shared, envelope_from: &str, raw: &[u8], mailboxes: &[String]) {
    let parsed = MessageParser::default().parse(raw);
    let from = parsed
        .as_ref()
        .and_then(|m| m.from())
        .and_then(|a| a.first())
        .and_then(|a| a.address())
        .map(|a| a.trim().to_ascii_lowercase())
        .unwrap_or_else(|| envelope_from.to_owned());
    let subject: String =
        parsed.as_ref().and_then(|m| m.subject()).unwrap_or("").chars().take(300).collect();
    let text: String = parsed
        .as_ref()
        .and_then(|m| m.body_text(0))
        .map(|t| t.chars().take(64 * 1024).collect())
        .unwrap_or_default();
    let mut state = shared.state();
    for to in mailboxes {
        state.next_id += 1;
        let id = state.next_id;
        if state.inbox.len() < MAX_RECORDS {
            state.inbox.push(MailboxMessage {
                id,
                to: to.clone(),
                from: from.clone(),
                subject: subject.clone(),
                text: text.clone(),
            });
        }
    }
}

/// Extracts the attachments of an approved message and imports what the service accepts.
fn process(shared: &Shared, state: &mut State, raw: &[u8]) -> MailOutcome {
    let Some(message) = MessageParser::default().parse(raw) else {
        return MailOutcome::Unparsable;
    };
    let from = message
        .from()
        .and_then(|a| a.first())
        .and_then(|a| a.address())
        .unwrap_or("")
        .to_ascii_lowercase();
    let mut outcomes = Vec::new();
    for part in message.attachments().take(MAX_ATTACHMENTS) {
        let bytes: &[u8] = match &part.body {
            PartType::Binary(bytes) | PartType::InlineBinary(bytes) => bytes.as_ref(),
            PartType::Text(text) | PartType::Html(text) => text.as_bytes(),
            // Forwarded messages and nested multiparts are not documents.
            PartType::Message(_) | PartType::Multipart(_) => continue,
        };
        let file_name = part.attachment_name().unwrap_or("attachment").trim().to_owned();
        let declared = part
            .content_type()
            .and_then(|ct| {
                let subtype = ct.subtype().unwrap_or("");
                FileFormat::from_media_type(&format!("{}/{subtype}", ct.ctype()))
            })
            .or_else(|| FileFormat::from_file_name(&file_name));
        let result = accept(shared, state, &file_name, bytes, declared, &from);
        outcomes.push(AttachmentOutcome { file_name, result });
    }
    if outcomes.is_empty() { MailOutcome::NoAttachment } else { MailOutcome::Processed(outcomes) }
}

fn accept(
    shared: &Shared,
    state: &mut State,
    file_name: &str,
    bytes: &[u8],
    declared: Option<FileFormat>,
    from: &str,
) -> Result<LibraryEntry, ImportError> {
    let service = &shared.service;
    let rules = service.file_names;
    let bad_name =
        |reason: &'static str| ImportError::BadFileName { file_name: file_name.to_owned(), reason };
    if rules.ascii_only && !file_name.is_ascii() {
        return Err(bad_name("non-ASCII characters"));
    }
    if rules.no_commas && file_name.contains(',') {
        return Err(bad_name("commas"));
    }
    if rules.max_len.is_some_and(|max| file_name.len() > max) {
        return Err(bad_name("too long"));
    }
    let file = IncomingFile { bytes, file_name, declared };
    let (format, info) = library::examine(file, service.max_message_bytes)?;
    if !service.accepted.contains(&format) {
        return Err(ImportError::UnsupportedFormat { format, device: service.id });
    }
    if let Some(scheme) = info.drm {
        return Err(ImportError::DrmProtected { scheme });
    }
    let (stored, notes) = match service.converts_epub_to {
        Some(target) if format == FileFormat::Epub => (
            target,
            vec![format!(
                "converted from EPUB by {} (simulated: the bytes were not converted)",
                service.name
            )],
        ),
        _ => (format, Vec::new()),
    };
    if !shared.profile.reads(stored) {
        return Err(ImportError::UnsupportedFormat { format: stored, device: shared.profile.id });
    }
    let source = ImportSource::Mail { from: from.to_owned(), service: service.id };
    let mut entry = library::build_entry(file, format, &info, source, notes);
    entry.format = stored;
    entry.kepub = false;
    Ok(state.library.push_unique(entry))
}

enum Line {
    Data(Vec<u8>),
    TooLong,
    Eof,
}

/// Reads CRLF- (or LF-) terminated lines with a hard length cap and no unbounded buffering.
struct LineReader<R> {
    inner: R,
    buf: Vec<u8>,
    start: usize,
}

impl<R: AsyncRead + Unpin> LineReader<R> {
    fn new(inner: R) -> Self {
        Self { inner, buf: Vec::with_capacity(8192), start: 0 }
    }

    async fn read_line(&mut self, max: usize) -> std::io::Result<Line> {
        let mut discarding = false;
        let mut scanned = self.start;
        loop {
            if let Some(offset) = self.buf[scanned..].iter().position(|b| *b == b'\n') {
                let end = scanned + offset;
                let mut line = &self.buf[self.start..end];
                if line.last() == Some(&b'\r') {
                    line = &line[..line.len() - 1];
                }
                let result = if discarding || line.len() > max {
                    Line::TooLong
                } else {
                    Line::Data(line.to_vec())
                };
                self.start = end + 1;
                return Ok(result);
            }
            if self.buf.len() - self.start > max {
                // The line is already too long: drop what we have and keep looking for its end.
                discarding = true;
                self.buf.clear();
                self.start = 0;
            } else if self.start > 0 {
                self.buf.drain(..self.start);
                self.start = 0;
            }
            scanned = self.buf.len();
            let mut chunk = [0u8; 8192];
            let read = self.inner.read(&mut chunk).await?;
            if read == 0 {
                return Ok(Line::Eof);
            }
            self.buf.extend_from_slice(&chunk[..read]);
        }
    }
}

async fn reply<W: AsyncWrite + Unpin>(writer: &mut W, text: &str) -> std::io::Result<()> {
    writer.write_all(text.as_bytes()).await?;
    writer.write_all(b"\r\n").await?;
    writer.flush().await
}

/// Extracts the address of `MAIL FROM:<a@b> PARAMS` / `RCPT TO:<a@b>` and the parameters.
fn parse_path(argument: &str, keyword: &str) -> Option<(String, String)> {
    let rest = argument.trim_start();
    let head = rest.get(..keyword.len())?;
    if !head.eq_ignore_ascii_case(keyword) {
        return None;
    }
    let rest = rest[keyword.len()..].trim_start();
    let (address, params) = if let Some(inner) = rest.strip_prefix('<') {
        let end = inner.find('>')?;
        (&inner[..end], &inner[end + 1..])
    } else {
        // Tolerate the bracket-less form some clients send.
        match rest.split_once(' ') {
            Some((address, params)) => (address, params),
            None => (rest, ""),
        }
    };
    // Source routes ("@relay:user@host") are obsolete; keep the final mailbox.
    let address = address.rsplit(':').next().unwrap_or("").trim();
    if address.chars().any(|c| c.is_whitespace() || c.is_control()) || address.len() > 320 {
        return None;
    }
    Some((address.to_ascii_lowercase(), params.trim().to_owned()))
}

enum DataEnd {
    Complete(Vec<u8>),
    TooBig,
    LineTooLong,
    Disconnected,
}

async fn read_data<R: AsyncRead + Unpin>(
    reader: &mut LineReader<R>,
    max_bytes: u64,
    idle: Duration,
) -> DataEnd {
    let max = usize::try_from(max_bytes).unwrap_or(usize::MAX);
    let mut data: Vec<u8> = Vec::new();
    let mut too_big = false;
    let mut line_too_long = false;
    loop {
        let line = match tokio::time::timeout(idle, reader.read_line(MAX_DATA_LINE)).await {
            Ok(Ok(Line::Data(line))) => line,
            Ok(Ok(Line::TooLong)) => {
                line_too_long = true;
                continue;
            }
            Ok(Ok(Line::Eof)) | Ok(Err(_)) | Err(_) => return DataEnd::Disconnected,
        };
        if line == b"." {
            break;
        }
        if too_big || line_too_long {
            continue;
        }
        // Dot-unstuffing (RFC 5321 §4.5.2): a leading dot was added by the client.
        let content = line.strip_prefix(b".").unwrap_or(&line[..]);
        if data.len() + content.len() + 2 > max {
            too_big = true;
            data = Vec::new();
            continue;
        }
        data.extend_from_slice(content);
        data.extend_from_slice(b"\r\n");
    }
    if too_big {
        DataEnd::TooBig
    } else if line_too_long {
        DataEnd::LineTooLong
    } else {
        DataEnd::Complete(data)
    }
}

async fn session(stream: TcpStream, shared: &Shared) -> std::io::Result<()> {
    let (read_half, mut writer) = stream.into_split();
    let mut reader = LineReader::new(read_half);
    let host = &shared.config.hostname;
    let idle = shared.config.idle_timeout;
    reply(&mut writer, &format!("220 {host} ESMTP eBook Cadeau demo mail device (simulation)"))
        .await?;

    let mut greeted = false;
    let mut sender: Option<String> = None;
    let mut recipients: Vec<String> = Vec::new();
    let mut errors = 0u32;

    loop {
        if errors > shared.config.max_errors {
            reply(&mut writer, "421 4.7.0 too many errors, closing").await?;
            return Ok(());
        }
        let line = match tokio::time::timeout(idle, reader.read_line(MAX_COMMAND_LINE)).await {
            Err(_) => {
                reply(&mut writer, "421 4.4.2 idle for too long, closing").await?;
                return Ok(());
            }
            Ok(Ok(Line::Eof)) => return Ok(()),
            Ok(Err(err)) => return Err(err),
            Ok(Ok(Line::TooLong)) => {
                errors += 1;
                reply(&mut writer, "500 5.5.2 line too long").await?;
                continue;
            }
            Ok(Ok(Line::Data(line))) => line,
        };
        let Ok(line) = String::from_utf8(line) else {
            errors += 1;
            reply(&mut writer, "500 5.5.2 command is not valid UTF-8").await?;
            continue;
        };
        let (verb, argument) = match line.trim().split_once(char::is_whitespace) {
            Some((verb, argument)) => (verb.to_ascii_uppercase(), argument.trim().to_owned()),
            None => (line.trim().to_ascii_uppercase(), String::new()),
        };
        match verb.as_str() {
            "EHLO" => {
                greeted = true;
                sender = None;
                recipients.clear();
                reply(
                    &mut writer,
                    &format!(
                        "250-{host} greets you\r\n250-SIZE {}\r\n250 8BITMIME",
                        shared.service.max_message_bytes
                    ),
                )
                .await?;
            }
            "HELO" => {
                greeted = true;
                sender = None;
                recipients.clear();
                reply(&mut writer, &format!("250 {host}")).await?;
            }
            "MAIL" => {
                if !greeted {
                    errors += 1;
                    reply(&mut writer, "503 5.5.1 say EHLO first").await?;
                } else if sender.is_some() {
                    errors += 1;
                    reply(&mut writer, "503 5.5.1 nested MAIL command").await?;
                } else if let Some((address, params)) = parse_path(&argument, "FROM:") {
                    let declared_size = params
                        .split_whitespace()
                        .filter_map(|p| p.split_once('='))
                        .find(|(key, _)| key.eq_ignore_ascii_case("SIZE"))
                        .and_then(|(_, value)| value.parse::<u64>().ok());
                    if declared_size.is_some_and(|size| size > shared.service.max_message_bytes) {
                        reply(&mut writer, "552 5.3.4 message size exceeds the service limit")
                            .await?;
                    } else {
                        sender = Some(address);
                        reply(&mut writer, "250 2.1.0 sender ok").await?;
                    }
                } else {
                    errors += 1;
                    reply(&mut writer, "501 5.5.4 syntax: MAIL FROM:<address>").await?;
                }
            }
            "RCPT" => {
                if sender.is_none() {
                    errors += 1;
                    reply(&mut writer, "503 5.5.1 MAIL first").await?;
                } else if let Some((address, _)) = parse_path(&argument, "TO:") {
                    let mailbox = is_mailbox(&shared.state().mailboxes, &address);
                    if address != shared.address && !mailbox {
                        reply(&mut writer, "550 5.1.1 no such device address").await?;
                    } else if recipients.len() >= 16 {
                        reply(&mut writer, "452 4.5.3 too many recipients").await?;
                    } else {
                        recipients.push(address);
                        reply(&mut writer, "250 2.1.5 recipient ok").await?;
                    }
                } else {
                    errors += 1;
                    reply(&mut writer, "501 5.5.4 syntax: RCPT TO:<address>").await?;
                }
            }
            "DATA" => {
                let Some(envelope_from) = sender.clone().filter(|_| !recipients.is_empty()) else {
                    errors += 1;
                    reply(&mut writer, "503 5.5.1 MAIL and RCPT first").await?;
                    continue;
                };
                reply(&mut writer, "354 end data with <CR><LF>.<CR><LF>").await?;
                let end = read_data(&mut reader, shared.service.max_message_bytes, idle).await;
                sender = None;
                let (device, mailboxes): (Vec<String>, Vec<String>) =
                    recipients.drain(..).partition(|r| *r == shared.address);
                match end {
                    DataEnd::Complete(raw) => {
                        // The service rules run before the reply, so a client that has seen
                        // "250" can rely on the device state being up to date.
                        if !mailboxes.is_empty() {
                            to_mailboxes(shared, &envelope_from, &raw, &mailboxes);
                        }
                        if !device.is_empty() {
                            receive(shared, &envelope_from, raw);
                        }
                        reply(&mut writer, "250 2.0.0 message accepted").await?;
                    }
                    DataEnd::TooBig => {
                        reply(&mut writer, "552 5.3.4 message size exceeds the service limit")
                            .await?;
                    }
                    DataEnd::LineTooLong => {
                        errors += 1;
                        reply(&mut writer, "500 5.5.2 line too long").await?;
                    }
                    DataEnd::Disconnected => return Ok(()),
                }
            }
            "RSET" => {
                sender = None;
                recipients.clear();
                reply(&mut writer, "250 2.0.0 reset").await?;
            }
            "NOOP" => reply(&mut writer, "250 2.0.0 ok").await?,
            "VRFY" => reply(&mut writer, "252 2.5.2 cannot verify, try RCPT").await?,
            "HELP" => reply(&mut writer, "214 2.0.0 EHLO MAIL RCPT DATA RSET NOOP QUIT").await?,
            "QUIT" => {
                reply(&mut writer, &format!("221 2.0.0 {host} closing")).await?;
                return Ok(());
            }
            "STARTTLS" | "AUTH" => {
                errors += 1;
                reply(
                    &mut writer,
                    "502 5.5.1 not implemented (this simulation has no TLS and no AUTH)",
                )
                .await?;
            }
            _ => {
                errors += 1;
                reply(&mut writer, "500 5.5.2 command not recognised").await?;
            }
        }
    }
}

/// Builds a MIME message with one attachment — the e-mail a person (or the gift server) sends
/// to a device address. Deterministic: no date, no random boundary.
pub fn compose_with_attachment(
    from: &str,
    to: &str,
    subject: &str,
    file_name: &str,
    media_type: &str,
    content: &[u8],
) -> Vec<u8> {
    let encoded_word = |text: &str| {
        if text.is_ascii() && !text.contains(['\r', '\n']) {
            text.to_owned()
        } else {
            format!("=?UTF-8?B?{}?=", crate::opds::base64_encode(text.as_bytes()))
        }
    };
    let clean = |text: &str| text.replace(['\r', '\n'], " ");
    let filename_param = if file_name.is_ascii() {
        format!("filename=\"{}\"", clean(file_name).replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        // RFC 2231 / 5987 extended parameter.
        let mut encoded = String::new();
        for byte in file_name.bytes() {
            if byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_') {
                encoded.push(char::from(byte));
            } else {
                encoded.push_str(&format!("%{byte:02X}"));
            }
        }
        format!("filename*=UTF-8''{encoded}")
    };
    let boundary = "=_ebc_demo_boundary_7f3a9c";
    let mut out = String::new();
    out.push_str(&format!("From: {}\r\n", clean(from)));
    out.push_str(&format!("To: {}\r\n", clean(to)));
    out.push_str(&format!("Subject: {}\r\n", encoded_word(subject)));
    out.push_str("MIME-Version: 1.0\r\n");
    out.push_str(&format!("Content-Type: multipart/mixed; boundary=\"{boundary}\"\r\n\r\n"));
    out.push_str(&format!("--{boundary}\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n"));
    out.push_str("Votre livre est en pièce jointe.\r\n");
    out.push_str(&format!("--{boundary}\r\nContent-Type: {}\r\n", clean(media_type)));
    out.push_str(&format!("Content-Disposition: attachment; {filename_param}\r\n"));
    out.push_str("Content-Transfer-Encoding: base64\r\n\r\n");
    let encoded = crate::opds::base64_encode(content);
    for chunk in encoded.as_bytes().chunks(76) {
        out.push_str(&String::from_utf8_lossy(chunk));
        out.push_str("\r\n");
    }
    out.push_str(&format!("--{boundary}--\r\n"));
    out.into_bytes()
}

/// A minimal SMTP client: connects, sends one message, quits. For tests and demos — the
/// counterpart of the gateway above, with the same limitations (no TLS, no AUTH).
///
/// Returns the server's final reply to `DATA` on success; any refusal is a
/// [`MailError::Smtp`] naming the step.
pub async fn send_mail(
    server: SocketAddr,
    envelope_from: &str,
    recipient: &str,
    message: &[u8],
    timeout: Duration,
) -> Result<String, MailError> {
    let run = async {
        let stream = TcpStream::connect(server).await.map_err(MailError::Io)?;
        let (read_half, mut writer) = stream.into_split();
        let mut reader = LineReader::new(read_half);
        expect(&mut reader, "greeting", 220).await?;
        command(&mut writer, &mut reader, "EHLO ebc-demo-client.invalid", "EHLO", 250).await?;
        command(
            &mut writer,
            &mut reader,
            &format!("MAIL FROM:<{envelope_from}>"),
            "MAIL FROM",
            250,
        )
        .await?;
        command(&mut writer, &mut reader, &format!("RCPT TO:<{recipient}>"), "RCPT TO", 250)
            .await?;
        command(&mut writer, &mut reader, "DATA", "DATA", 354).await?;
        // Dot-stuffing and CRLF normalisation.
        let mut wire = Vec::with_capacity(message.len() + 64);
        for line in message.split(|b| *b == b'\n') {
            let line = line.strip_suffix(b"\r").unwrap_or(line);
            if line.first() == Some(&b'.') {
                wire.push(b'.');
            }
            wire.extend_from_slice(line);
            wire.extend_from_slice(b"\r\n");
        }
        wire.extend_from_slice(b".\r\n");
        writer.write_all(&wire).await.map_err(MailError::Io)?;
        writer.flush().await.map_err(MailError::Io)?;
        let accepted = expect(&mut reader, "end of DATA", 250).await?;
        let _ = command(&mut writer, &mut reader, "QUIT", "QUIT", 221).await;
        Ok(accepted)
    };
    match tokio::time::timeout(timeout, run).await {
        Ok(result) => result,
        Err(_) => Err(MailError::Timeout { stage: "session" }),
    }
}

async fn command<W: AsyncWrite + Unpin, R: AsyncRead + Unpin>(
    writer: &mut W,
    reader: &mut LineReader<R>,
    line: &str,
    stage: &'static str,
    want: u16,
) -> Result<String, MailError> {
    reply(writer, line).await.map_err(MailError::Io)?;
    expect(reader, stage, want).await
}

/// Reads one (possibly multi-line) SMTP reply and checks its code.
async fn expect<R: AsyncRead + Unpin>(
    reader: &mut LineReader<R>,
    stage: &'static str,
    want: u16,
) -> Result<String, MailError> {
    let mut text = String::new();
    for _ in 0..64 {
        let line = match reader.read_line(MAX_COMMAND_LINE).await.map_err(MailError::Io)? {
            Line::Data(line) => String::from_utf8_lossy(&line).into_owned(),
            Line::TooLong => {
                return Err(MailError::Smtp {
                    stage,
                    code: 0,
                    message: "reply too long".to_owned(),
                });
            }
            Line::Eof => {
                return Err(MailError::Smtp {
                    stage,
                    code: 0,
                    message: "connection closed".to_owned(),
                });
            }
        };
        let code = line.get(..3).and_then(|c| c.parse::<u16>().ok()).unwrap_or(0);
        let rest = line.get(4..).unwrap_or("").to_owned();
        if !text.is_empty() {
            text.push('\n');
        }
        text.push_str(&rest);
        if line.as_bytes().get(3) == Some(&b'-') {
            continue;
        }
        return if code == want {
            Ok(text)
        } else {
            Err(MailError::Smtp { stage, code, message: text })
        };
    }
    Err(MailError::Smtp { stage, code: 0, message: "endless multi-line reply".to_owned() })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn approval_matches_addresses_and_domains() {
        let approved = vec![
            "livraison@ebk.example".to_owned(),
            "@amie.example".to_owned(),
            "club.example".to_owned(),
        ];
        assert!(is_approved(&approved, "Livraison@EBK.example"));
        assert!(!is_approved(&approved, "autre@ebk.example"));
        assert!(is_approved(&approved, "camille@amie.example"));
        assert!(is_approved(&approved, "x@club.example"));
        assert!(!is_approved(&approved, "x@sub.club.example"));
        assert!(!is_approved(&approved, ""));
        assert!(!is_approved(&[], "livraison@ebk.example"));
    }

    #[test]
    fn smtp_paths_are_parsed_leniently_but_safely() {
        assert_eq!(
            parse_path("FROM:<A@B.example> SIZE=12", "FROM:"),
            Some(("a@b.example".into(), "SIZE=12".into()))
        );
        assert_eq!(
            parse_path("from: <a@b.example>", "FROM:"),
            Some(("a@b.example".into(), String::new()))
        );
        assert_eq!(
            parse_path("TO:a@b.example", "TO:"),
            Some(("a@b.example".into(), String::new()))
        );
        assert_eq!(parse_path("FROM:<>", "FROM:"), Some((String::new(), String::new())));
        assert_eq!(
            parse_path("FROM:<@relay.example:a@b.example>", "FROM:"),
            Some(("a@b.example".into(), String::new()))
        );
        assert_eq!(parse_path("FROM:<a@b.example", "FROM:"), None);
        assert_eq!(parse_path("TO:<a@b.example>", "FROM:"), None);
        assert_eq!(parse_path("", "FROM:"), None);
        assert_eq!(parse_path("FRO", "FROM:"), None);
        assert_eq!(parse_path("FROM:<a b@c>", "FROM:"), None);
        assert_eq!(parse_path("é", "FROM:"), None);
        assert_eq!(parse_path(&format!("FROM:<{}@b>", "a".repeat(400)), "FROM:"), None);
    }

    #[test]
    fn composed_messages_parse_back_with_their_attachment() {
        let content: Vec<u8> = (0..=255u8).cycle().take(1000).collect();
        let raw = compose_with_attachment(
            "Camille <camille@amie.example>",
            "sacha_42@kindle.com",
            "Un livre pour toi — « Le Horla »",
            "le-horla.epub",
            "application/epub+zip",
            &content,
        );
        let message = MessageParser::default().parse(&raw).unwrap();
        assert_eq!(
            message.from().unwrap().first().unwrap().address(),
            Some("camille@amie.example")
        );
        assert_eq!(message.subject(), Some("Un livre pour toi — « Le Horla »"));
        let attachment = message.attachments().next().unwrap();
        assert_eq!(attachment.attachment_name(), Some("le-horla.epub"));
        assert_eq!(attachment.contents(), content.as_slice());
        // Non-ASCII file names survive too.
        let raw = compose_with_attachment(
            "a@b.example",
            "c@d.example",
            "s",
            "déjà vu.epub",
            "application/epub+zip",
            b"x",
        );
        let message = MessageParser::default().parse(&raw).unwrap();
        assert_eq!(message.attachments().next().unwrap().attachment_name(), Some("déjà vu.epub"));
        // Deterministic.
        assert_eq!(
            compose_with_attachment("a@b", "c@d", "s", "f.pdf", "application/pdf", b"x"),
            compose_with_attachment("a@b", "c@d", "s", "f.pdf", "application/pdf", b"x")
        );
        // Header injection through CR/LF is neutralised.
        let raw = compose_with_attachment(
            "a@b\r\nBcc: evil@x",
            "c@d",
            "s\r\nX: y",
            "f\r\n.pdf",
            "application/pdf",
            b"x",
        );
        let text = String::from_utf8_lossy(&raw);
        assert!(!text.contains("\r\nBcc:"));
        assert!(!text.contains("\r\nX: y"));
    }

    #[tokio::test]
    async fn line_reader_splits_caps_and_survives_fragmentation() {
        let (mut client, server) = tokio::io::duplex(64);
        let mut reader = LineReader::new(server);
        let writer = tokio::spawn(async move {
            client.write_all(b"EHLO a\r\nNO").await.unwrap();
            client.write_all(b"OP\n").await.unwrap();
            client.write_all(&vec![b'x'; 5000]).await.unwrap();
            client.write_all(b"\r\nQUIT\r\npartial").await.unwrap();
        });
        assert!(matches!(reader.read_line(100).await.unwrap(), Line::Data(l) if l == b"EHLO a"));
        assert!(matches!(reader.read_line(100).await.unwrap(), Line::Data(l) if l == b"NOOP"));
        assert!(matches!(reader.read_line(100).await.unwrap(), Line::TooLong));
        assert!(matches!(reader.read_line(100).await.unwrap(), Line::Data(l) if l == b"QUIT"));
        assert!(matches!(reader.read_line(100).await.unwrap(), Line::Eof));
        writer.await.unwrap();
    }

    #[tokio::test]
    async fn device_addresses_must_belong_to_the_service() {
        let kindle = DeviceProfile::kindle_paperwhite();
        for bad in ["sacha@gmail.com", "kindle.com", "@kindle.com", "sa cha@kindle.com", ""] {
            let err = MailDevice::start(kindle.clone(), bad).await.unwrap_err();
            assert!(matches!(err, MailError::BadDeviceAddress { .. }), "{bad:?}: {err}");
        }
        let err = MailDevice::start(DeviceProfile::kobo_clara(), "x@kobo.com").await.unwrap_err();
        assert!(matches!(err, MailError::NoMailService { device: "kobo-clara" }));
        let device = MailDevice::start(kindle, " Sacha_42@Kindle.com ").await.unwrap();
        assert_eq!(device.address(), "sacha_42@kindle.com");
        assert!(device.smtp_url().starts_with("smtp://127.0.0.1:"));
    }

    #[tokio::test]
    async fn plain_mailboxes_keep_text_and_leave_the_device_alone() {
        let device = MailDevice::start(DeviceProfile::kindle_paperwhite(), "sacha@kindle.com")
            .await
            .unwrap();
        let text = b"From: etagere@exemple.fr\r\nTo: autrice@exemple.fr\r\nSubject: Lien\r\n\r\nOuvrez http://x/connexion/abc\r\n";
        let refused = send_mail(
            device.smtp_addr(),
            "etagere@exemple.fr",
            "autrice@exemple.fr",
            text,
            Duration::from_secs(5),
        )
        .await;
        assert!(refused.is_err(), "not a mailbox yet");
        device.add_mailbox("@Exemple.fr");
        send_mail(
            device.smtp_addr(),
            "etagere@exemple.fr",
            "autrice@exemple.fr",
            text,
            Duration::from_secs(5),
        )
        .await
        .unwrap();
        let inbox = device.mailbox("autrice@exemple.fr");
        assert_eq!(inbox.len(), 1);
        assert_eq!(
            (inbox[0].from.as_str(), inbox[0].subject.as_str()),
            ("etagere@exemple.fr", "Lien")
        );
        assert!(inbox[0].text.contains("/connexion/abc"));
        assert!(device.records().is_empty() && device.library().entries().is_empty());
        assert!(device.mailbox("autre@ailleurs.fr").is_empty());
    }
}
