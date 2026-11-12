//! What the service does with gifts, independent of HTTP: creating, cancelling and
//! reissuing gifts, looking codes up, opening download sessions, pairing, redirecting to
//! a store, obtaining download links from partners. Handlers in [`crate::routes`]
//! translate the outcomes into pages and JSON.
//!
//! The service never stores a book: a `partner_file` gift's file is served by its
//! partner, which hands out a short-lived download link on each claim (`docs/api.md`
//! §4.5).

use axum::body::Bytes;
use axum::http::StatusCode;
use ebc_model::{
    BookMeta, Delivery, FileFormat, Gift, GiftCode, GiftId, GiftStatus, LicenceFormat, Locale,
    PairCode, PartnerFileLink,
};
use serde::Deserialize;

use crate::codes;
use crate::config::Partner;
use crate::device::DeviceClass;
use crate::error::ApiError;
use crate::names::{ascii_file_name, display_file_name};
use crate::state::AppState;
use crate::store::{BlobRef, GiftRecord, PairRecord, SessionRecord, StoreError};

/// Largest licence document accepted from a store.
const MAX_LICENCE_BYTES: usize = 1 << 20;
/// Largest JSON answer accepted from a partner's file endpoint.
const MAX_LINK_ANSWER_BYTES: usize = 64 * 1024;
/// A partner's download link is reused for a session while it has at least this many
/// seconds left.
const LINK_REUSE_MARGIN_SECS: i64 = 10;
/// Lifetime assumed for a download link whose partner gave no `expires_at`.
const LINK_DEFAULT_TTL_SECS: i64 = 60;
/// Sendings of one gift to device addresses per 24 hours.
pub(crate) const MAX_SENDS_PER_DAY: usize = 3;

impl From<StoreError> for ApiError {
    fn from(error: StoreError) -> Self {
        Self::internal(error)
    }
}

impl From<crate::clock::EntropyError> for ApiError {
    fn from(error: crate::clock::EntropyError) -> Self {
        Self::internal(error)
    }
}

/// The `delivery` member of a gift creation request (`docs/api.md` §4.2).
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum DeliveryRequest {
    /// The partner serves the file through short-lived download links.
    PartnerFile {
        /// `epub` or `pdf`.
        format: String,
        /// The partner's signed file endpoint.
        url: String,
    },
    /// The store adds the book to the recipient's account.
    ResellerRedirect {
        /// The store's activation URL.
        url: String,
    },
    /// The store issues a licence file.
    ResellerLicence {
        /// `lcpl`, `acsm`…
        format: String,
        /// Where the licence is fetched.
        url: String,
    },
}

/// The body of `POST /api/partner/v1/gifts`.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct GiftRequest {
    #[serde(default)]
    pub(crate) order_ref: Option<String>,
    pub(crate) book: BookMeta,
    #[serde(default)]
    pub(crate) giver_name: Option<String>,
    #[serde(default)]
    pub(crate) recipient_name: Option<String>,
    #[serde(default)]
    pub(crate) message: Option<String>,
    #[serde(default)]
    pub(crate) locale: Option<String>,
    pub(crate) delivery: DeliveryRequest,
    /// The rights basis declared for the cover the partner will upload
    /// ([`crate::card::CoverRights`]); absent means `none`.
    #[serde(default)]
    pub(crate) cover_rights: Option<String>,
    #[serde(default)]
    pub(crate) reveal_at: Option<i64>,
    #[serde(default)]
    pub(crate) expires_at: Option<i64>,
}

impl GiftRequest {
    /// Parses a creation body: `400 invalid_json` for text that is not JSON, `422
    /// invalid_request` for JSON that does not fit (missing member, invalid ISBN…).
    ///
    /// Empty strings in `book` (`"isbn13": ""`, `"publisher": ""`) mean "not known", as
    /// stores serialise absent values that way; they are dropped rather than refused.
    pub(crate) fn parse(body: &[u8]) -> Result<Self, ApiError> {
        let mut value: serde_json::Value = serde_json::from_slice(body)
            .map_err(|error| ApiError::bad_request("invalid_json", error.to_string()))?;
        if let Some(book) = value.get_mut("book").and_then(serde_json::Value::as_object_mut) {
            book.retain(|key, member| {
                key == "title"
                    || !(member.is_null()
                        || member.as_str().is_some_and(|text| text.trim().is_empty()))
            });
        }
        serde_json::from_value(value)
            .map_err(|error| ApiError::unprocessable("invalid_request", error.to_string()))
    }
}

/// Parses a declared cover rights basis (`cover_rights` in a gift body or on the cover
/// upload): `Ok(None)` when nothing was declared, `422 invalid_cover_rights` for a value
/// that is not one of [`crate::card::CoverRights`].
pub(crate) fn parse_cover_rights(
    declared: Option<&str>,
) -> Result<Option<crate::card::CoverRights>, ApiError> {
    let Some(value) = declared.map(str::trim).filter(|v| !v.is_empty()) else {
        return Ok(None);
    };
    crate::card::CoverRights::parse(value).map(Some).ok_or_else(|| {
        ApiError::unprocessable(
            "invalid_cover_rights",
            "cover_rights must be one of none, print_licence, promo_all_media, \
             rights_holder_declared, public_domain",
        )
    })
}

/// A gift that was just created, with its code.
pub(crate) struct CreatedGift {
    pub(crate) record: GiftRecord,
    pub(crate) code: GiftCode,
}

/// What a code typed by a recipient turned out to be.
pub(crate) enum CodeLookup {
    /// A gift (whatever its status).
    Gift(Box<GiftRecord>),
    /// A code voided by a reissue.
    Voided,
    /// Well-formed, never issued.
    Unknown,
}

/// Why a claim cannot go ahead.
#[derive(Debug)]
pub(crate) enum ClaimError {
    /// Cancelled or expired (`410`).
    Gone(GiftStatus),
    /// Before the reveal date (`403`).
    NotRevealed,
    /// The session limit is reached (`410`).
    Exhausted,
    /// The action does not exist for this kind of gift, or the gift is already in an
    /// account (`409`).
    NotAvailable,
    /// The store did not deliver (`502`).
    StoreUnavailable,
    /// The partner's file is larger than a device e-mail may carry (`EBC_SEND_MAX_MB`,
    /// `502`).
    TooLargeToSend,
    /// Too many sendings today (`429`).
    SendLimit,
    /// Something broke on our side (`500`).
    Internal,
}

impl ClaimError {
    fn internal(detail: impl std::fmt::Display) -> Self {
        tracing::error!(%detail, "claim failed");
        Self::Internal
    }

    /// The HTTP status of the page that reports this error.
    pub(crate) fn status(&self) -> StatusCode {
        match self {
            Self::Gone(_) | Self::Exhausted => StatusCode::GONE,
            Self::NotRevealed => StatusCode::FORBIDDEN,
            Self::NotAvailable => StatusCode::CONFLICT,
            Self::StoreUnavailable | Self::TooLargeToSend => StatusCode::BAD_GATEWAY,
            Self::SendLimit => StatusCode::TOO_MANY_REQUESTS,
            Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// A sending to a device address counted before the mail is sent
/// ([`AppState::reserve_send`]).
#[derive(Debug)]
pub(crate) struct SendReservation {
    /// The gift as updated by the reservation.
    record: GiftRecord,
    /// When the sending was counted.
    at: i64,
    /// Whether it was the first claim of the gift.
    first: bool,
}

/// A download link a partner handed out, kept in memory for its session
/// ([`crate::state::Inner::partner_links`]).
#[derive(Debug, Clone)]
pub(crate) struct CachedLink {
    /// Where the device is redirected.
    pub(crate) url: reqwest::Url,
    /// When the link stops working (the partner's `expires_at`, or a default).
    pub(crate) expires_at: i64,
}

/// A partner's answer, checked: the download URL parsed, the rest as given.
#[derive(Debug, Clone)]
pub(crate) struct ResolvedLink {
    url: reqwest::Url,
    link: PartnerFileLink,
}

/// A freshly minted download session.
pub(crate) struct Ticket {
    /// The URL token (128 random bits, hexadecimal).
    pub(crate) token: String,
    /// The file name that completes the URL.
    pub(crate) file_name: String,
}

impl Ticket {
    /// `/d/{session}/{file-name}`.
    pub(crate) fn download_path(&self) -> String {
        format!("/d/{}/{}", self.token, self.file_name)
    }
}

/// The file a session will serve.
struct Payload {
    sha256: String,
    size_bytes: u64,
    media_type: String,
    file_name: String,
    display_name: String,
    licence: bool,
    /// The bytes of a licence, to store again once the session references them.
    bytes: Option<Bytes>,
    /// The download link of a partner file, cached once the session exists.
    link: Option<ResolvedLink>,
}

fn unprocessable(code: &'static str, message: impl Into<String>) -> ApiError {
    ApiError::unprocessable(code, message)
}

/// Bounds the free-form parts of the book metadata (the whole body is already capped).
fn check_book(book: &BookMeta) -> Result<(), ApiError> {
    let bad_text =
        |text: &str, max: usize| text.chars().count() > max || text.chars().any(char::is_control);
    if book.title.trim().is_empty() || bad_text(&book.title, 300) {
        return Err(unprocessable(
            "invalid_book",
            "book.title must be 1 to 300 characters without control characters",
        ));
    }
    if book.contributors.len() > 20
        || book.contributors.iter().any(|c| c.name.trim().is_empty() || bad_text(&c.name, 200))
    {
        return Err(unprocessable(
            "invalid_book",
            "book.contributors: at most 20 names of 1 to 200 characters",
        ));
    }
    if book.publisher.as_deref().is_some_and(|p| bad_text(p, 200)) {
        return Err(unprocessable("invalid_book", "book.publisher is too long"));
    }
    Ok(())
}

impl AppState {
    /// The partner record for an id stored on a gift.
    pub(crate) fn partner_of(&self, gift: &GiftRecord) -> Option<&Partner> {
        gift.partner.as_deref().and_then(|id| self.config.partner(id))
    }

    /// Writes a file to the store without blocking the async runtime.
    pub(crate) async fn put_blob(&self, sha256: &str, bytes: Bytes) -> Result<(), ApiError> {
        let store = std::sync::Arc::clone(self.db.store());
        let name = sha256.to_owned();
        tokio::task::spawn_blocking(move || store.put_blob(&name, &bytes))
            .await
            .map_err(ApiError::internal)?
            .map_err(ApiError::internal)
    }

    /// Reads a whole stored file.
    pub(crate) async fn read_blob(&self, sha256: &str) -> Result<Option<Bytes>, StoreError> {
        let Some(blob) = self.db.store().open_blob(sha256)? else { return Ok(None) };
        tokio::task::spawn_blocking(move || blob.read_all())
            .await
            .map_err(|e| StoreError::Io(e.to_string()))?
            .map(Some)
    }

    /// Creates a gift (`POST /api/partner/v1/gifts`).
    pub(crate) fn create_gift(
        &self,
        request: GiftRequest,
        partner: &Partner,
    ) -> Result<CreatedGift, ApiError> {
        let now = self.now();
        check_book(&request.book)?;
        let locale = match request.locale.as_deref().map(str::trim).filter(|l| !l.is_empty()) {
            Some(tag) => Locale::from_tag(tag).ok_or_else(|| {
                unprocessable("invalid_locale", "locale must be one of fr, en, de, es, it, nl, pt")
            })?,
            None => Locale::default(),
        };
        let order_ref = request
            .order_ref
            .as_deref()
            .map(str::trim)
            .filter(|r| !r.is_empty())
            .map(str::to_owned);
        if order_ref.as_deref().is_some_and(|r| {
            r.chars().count() > 128 || r.chars().any(|c| c.is_control() || c.is_whitespace())
        }) {
            return Err(unprocessable(
                "invalid_order_ref",
                "order_ref: at most 128 characters without spaces",
            ));
        }
        let text = |value: &Option<String>| value.as_deref().and_then(Gift::normalise_text);
        let cover_rights = parse_cover_rights(request.cover_rights.as_deref())?.unwrap_or_default();

        // Every delivery URL points to the partner, on one of its redirect_hosts.
        let url = match &request.delivery {
            DeliveryRequest::PartnerFile { url, .. }
            | DeliveryRequest::ResellerRedirect { url }
            | DeliveryRequest::ResellerLicence { url, .. } => url,
        };
        if !partner.allows_url(url) {
            return Err(unprocessable(
                "redirect_host_not_allowed",
                "delivery.url must be an http(s) URL on one of the partner's redirect_hosts",
            ));
        }
        let delivery = match &request.delivery {
            DeliveryRequest::PartnerFile { format, url } => Delivery::PartnerFile {
                format: FileFormat::parse(format).ok_or_else(|| {
                    unprocessable("invalid_delivery", "delivery.format must be epub or pdf")
                })?,
                url: url.clone(),
            },
            DeliveryRequest::ResellerLicence { format, url } => Delivery::ResellerLicence {
                format: LicenceFormat::from(format.as_str()),
                url: url.clone(),
            },
            DeliveryRequest::ResellerRedirect { url } => {
                Delivery::ResellerRedirect { url: url.clone() }
            }
        };

        let mut id_bytes = [0u8; 16];
        self.entropy.fill(&mut id_bytes)?;
        let id = GiftId::from_random_bytes(&id_bytes).map_err(ApiError::internal)?;

        for _ in 0..8 {
            // Codes are derived from a stored nonce so that the card can be rendered
            // later (`card.pdf`).
            let nonce = codes::random_hex(self.entropy.as_ref(), 16)?;
            let (code, counter) = codes::derive_gift_code(&self.config.code_secret, &nonce, 0)
                .ok_or_else(|| ApiError::internal("no gift code could be derived"))?;
            let model = Gift {
                id: id.clone(),
                code,
                book: request.book.clone(),
                giver_name: text(&request.giver_name),
                recipient_name: text(&request.recipient_name),
                message: text(&request.message),
                locale,
                created_unix: now,
                expires_unix: request.expires_at,
                reveal_unix: request.reveal_at,
                delivery: delivery.clone(),
            };
            model.validate().map_err(|e| unprocessable("invalid_gift", e.to_string()))?;
            let record = GiftRecord {
                id: id.as_str().to_owned(),
                code_hash: codes::gift_code_hash(&self.config.code_secret, &code),
                code_nonce: Some(nonce),
                code_counter: counter,
                partner: Some(partner.id.clone()),
                order_ref: order_ref.clone(),
                book: model.book,
                giver_name: model.giver_name,
                recipient_name: model.recipient_name,
                message: model.message,
                locale,
                delivery: model.delivery,
                cover_rights,
                status: GiftStatus::Created,
                created_at: now,
                expires_at: request.expires_at,
                reveal_at: request.reveal_at,
                redeemed_at: None,
                last_claim_at: None,
                claims: 0,
                account_ref_hash: None,
                cover: None,
                card_pdf: None,
                print_orders: Vec::new(),
                sends: Vec::new(),
                ended_at: None,
                anonymised_at: None,
                personal_data_deleted_at: None,
            };
            let (inserted, stale) = self.db.update(|db| {
                // Every write runs the purge (retention, expired sessions).
                let stale = db.purge(now, &self.config.purge_policy());
                if db.codes.contains_key(&record.code_hash)
                    || db.voided.contains_key(&record.code_hash)
                {
                    return (None, stale);
                }
                db.codes.insert(record.code_hash.clone(), record.id.clone());
                db.gifts.insert(record.id.clone(), record.clone());
                (Some(record), stale)
            })?;
            self.db.delete_unused_blobs(&stale);
            if let Some(record) = inserted {
                return Ok(CreatedGift { record, code });
            }
        }
        Err(ApiError::internal("could not draw an unused gift code"))
    }

    /// The gift with this id, if any.
    pub(crate) fn gift(&self, id: &str) -> Option<GiftRecord> {
        self.db.view(|db| db.gifts.get(id).cloned())
    }

    /// The clear code of a partner gift, re-derived from its nonce.
    pub(crate) fn derived_code(&self, gift: &GiftRecord) -> Option<GiftCode> {
        let nonce = gift.code_nonce.as_deref()?;
        let (code, _) =
            codes::derive_gift_code(&self.config.code_secret, nonce, gift.code_counter)?;
        // Guard against a secret that changed since the gift was created.
        (codes::gift_code_hash(&self.config.code_secret, &code) == gift.code_hash).then_some(code)
    }

    /// Cancels a gift: the code stops working, sessions and pair codes are dropped, its
    /// cover and card PDF are deleted and the names and message are forgotten at once
    /// (the rest of the record is anonymised after `EBC_RETENTION_DAYS`). Idempotent.
    pub(crate) fn cancel_gift(&self, id: &str) -> Result<GiftRecord, ApiError> {
        let now = self.now();
        let (record, newly, files) = self
            .db
            .update(|db| {
                let gift = db.gifts.get_mut(id)?;
                let newly = gift.status != GiftStatus::Cancelled;
                gift.status = GiftStatus::Cancelled;
                gift.ended_at.get_or_insert(now);
                gift.forget_personal_data();
                let record = gift.clone();
                let mut files: Vec<String> = record
                    .card_pdf
                    .iter()
                    .chain(record.cover.iter())
                    .map(|b| b.sha256.clone())
                    .collect();
                db.sessions.retain(|_, session| {
                    let keep = session.gift_id != id;
                    if !keep && session.licence {
                        files.push(session.sha256.clone());
                    }
                    keep
                });
                db.pairs.retain(|_, pair| pair.gift_id != id);
                if let Some(gift) = db.gifts.get_mut(id) {
                    gift.card_pdf = None;
                    gift.cover = None;
                }
                Some((record, newly, files))
            })?
            .ok_or_else(|| ApiError::not_found("no such gift"))?;
        self.db.delete_unused_blobs(&files);
        if newly {
            self.emit(&record, "gift.cancelled");
        }
        Ok(record)
    }

    /// Voids the code of a gift and issues a new one (lost card).
    pub(crate) fn reissue_gift(&self, id: &str) -> Result<(GiftRecord, GiftCode), ApiError> {
        let now = self.now();
        let current = self.gift(id).ok_or_else(|| ApiError::not_found("no such gift"))?;
        if !current.status_at(now).allows_claim() {
            return Err(ApiError::conflict(
                "not_reissuable",
                "only a gift that can still be claimed gets a new code",
            ));
        }
        for _ in 0..8 {
            let nonce = codes::random_hex(self.entropy.as_ref(), 16)?;
            let (code, counter) = codes::derive_gift_code(&self.config.code_secret, &nonce, 0)
                .ok_or_else(|| ApiError::internal("no gift code could be derived"))?;
            let hash = codes::gift_code_hash(&self.config.code_secret, &code);
            let updated = self.db.update(|db| -> Result<Option<GiftRecord>, ApiError> {
                if db.codes.contains_key(&hash) || db.voided.contains_key(&hash) {
                    return Ok(None);
                }
                let gift =
                    db.gifts.get_mut(id).ok_or_else(|| ApiError::not_found("no such gift"))?;
                if !gift.status_at(now).allows_claim() {
                    return Err(ApiError::conflict(
                        "not_reissuable",
                        "the gift can no longer be claimed",
                    ));
                }
                let old = std::mem::replace(&mut gift.code_hash, hash.clone());
                gift.code_nonce = Some(nonce);
                gift.code_counter = counter;
                let record = gift.clone();
                db.codes.remove(&old);
                db.voided.insert(old, now);
                db.codes.insert(hash, record.id.clone());
                // A pair code obtained with the old card must not outlive it.
                db.pairs.retain(|_, pair| pair.gift_id != id);
                Ok(Some(record))
            })??;
            if let Some(record) = updated {
                return Ok((record, code));
            }
        }
        Err(ApiError::internal("could not draw an unused gift code"))
    }

    /// Looks a well-formed code up.
    pub(crate) fn lookup_code(&self, code: &GiftCode) -> CodeLookup {
        let hash = codes::gift_code_hash(&self.config.code_secret, code);
        self.db.view(|db| match db.codes.get(&hash).and_then(|id| db.gifts.get(id)) {
            Some(gift) => CodeLookup::Gift(Box::new(gift.clone())),
            None if db.voided.contains_key(&hash) => CodeLookup::Voided,
            None => CodeLookup::Unknown,
        })
    }

    /// Whether `gift` can be claimed now; the error says why not.
    pub(crate) fn check_claimable(&self, gift: &GiftRecord) -> Result<(), ClaimError> {
        let now = self.now();
        match gift.status_at(now) {
            status if status.is_gone() => Err(ClaimError::Gone(status)),
            GiftStatus::Exhausted => Err(ClaimError::Exhausted),
            GiftStatus::Fulfilled => Err(ClaimError::NotAvailable),
            _ if !gift.is_revealed_at(now) => Err(ClaimError::NotRevealed),
            _ => Ok(()),
        }
    }

    /// Counts one claim on `gift_id` and applies `change` in the same transaction.
    /// With `send`, the claim is also a sending to a device address: refused with
    /// [`ClaimError::SendLimit`] when [`MAX_SENDS_PER_DAY`] are already recorded, and
    /// recorded in the same transaction (so that concurrent requests cannot both pass).
    /// Returns the updated gift and whether this was the first claim.
    fn record_claim(
        &self,
        gift_id: &str,
        capped: bool,
        send: bool,
        change: impl FnOnce(&mut crate::store::Db, &GiftRecord),
    ) -> Result<(GiftRecord, bool, Vec<String>), ClaimError> {
        let now = self.now();
        let cap = self.config.max_download_sessions;
        self.db
            .update(|db| -> Result<(GiftRecord, bool, Vec<String>), ClaimError> {
                let stale = db.purge(now, &self.config.purge_policy());
                let gift =
                    db.gifts.get_mut(gift_id).ok_or(ClaimError::Gone(GiftStatus::Cancelled))?;
                match gift.status_at(now) {
                    status if status.is_gone() => return Err(ClaimError::Gone(status)),
                    GiftStatus::Exhausted => return Err(ClaimError::Exhausted),
                    GiftStatus::Fulfilled => return Err(ClaimError::NotAvailable),
                    _ if !gift.is_revealed_at(now) => return Err(ClaimError::NotRevealed),
                    _ => {}
                }
                if capped && cap > 0 && gift.claims >= cap {
                    gift.status = GiftStatus::Exhausted;
                    return Err(ClaimError::Exhausted);
                }
                if send {
                    gift.sends.retain(|at| now.saturating_sub(*at) < 86_400);
                    if gift.sends.len() >= MAX_SENDS_PER_DAY {
                        return Err(ClaimError::SendLimit);
                    }
                    gift.sends.push(now);
                }
                let first = gift.redeemed_at.is_none();
                gift.claims = gift.claims.saturating_add(1);
                gift.last_claim_at = Some(now);
                if first {
                    gift.redeemed_at = Some(now);
                    gift.status = GiftStatus::Redeemed;
                }
                if capped && cap > 0 && gift.claims >= cap {
                    gift.status = GiftStatus::Exhausted;
                }
                let record = gift.clone();
                change(db, &record);
                Ok((record, first, stale))
            })
            .map_err(ClaimError::internal)?
    }

    /// The file a session for `gift` serves: a download link obtained from the partner
    /// right now, or a licence fetched from the store right now.
    async fn payload(&self, gift: &GiftRecord) -> Result<Payload, ClaimError> {
        match &gift.delivery {
            Delivery::PartnerFile { format, url } => {
                let resolved = self.request_partner_link(gift, url, *format).await?;
                let extension = format.as_str();
                Ok(Payload {
                    sha256: String::new(),
                    size_bytes: resolved.link.size_bytes.unwrap_or(0),
                    media_type: format.media_type().to_owned(),
                    file_name: ascii_file_name(&gift.book.title, extension),
                    display_name: resolved
                        .link
                        .file_name
                        .clone()
                        .unwrap_or_else(|| display_file_name(&gift.book.title, extension)),
                    licence: false,
                    bytes: None,
                    link: Some(resolved),
                })
            }
            Delivery::ResellerLicence { format, url } => {
                let bytes = self.fetch_licence(gift, url).await?;
                let sha256 = codes::sha256_hex(&bytes);
                let size_bytes = bytes.len() as u64;
                self.put_blob(&sha256, bytes.clone()).await.map_err(|_| ClaimError::Internal)?;
                let extension = format.as_str();
                Ok(Payload {
                    sha256,
                    size_bytes,
                    media_type: format
                        .media_type()
                        .unwrap_or("application/octet-stream")
                        .to_owned(),
                    file_name: ascii_file_name(&gift.book.title, extension),
                    display_name: display_file_name(&gift.book.title, extension),
                    licence: true,
                    bytes: Some(bytes),
                    link: None,
                })
            }
            Delivery::ResellerRedirect { .. } => Err(ClaimError::NotAvailable),
        }
    }

    /// The store URL of a reseller gift with the signed parameters of `docs/api.md` §4.3.
    /// `None` when the partner is gone or no longer allows the stored URL: the target
    /// always comes from the gift and is re-checked against the allow-list.
    pub(crate) fn signed_store_url(&self, gift: &GiftRecord, url: &str) -> Option<reqwest::Url> {
        let partner = self.partner_of(gift)?;
        if !partner.allows_url(url) {
            return None;
        }
        let mut target = reqwest::Url::parse(url).ok()?;
        let ts = self.now();
        let order_ref = gift.order_ref.as_deref().unwrap_or("");
        let signature = codes::redirect_signature(&partner.secret, &gift.id, order_ref, ts);
        target
            .query_pairs_mut()
            .append_pair("ebc_gift", &gift.id)
            .append_pair("ebc_order", order_ref)
            .append_pair("ebc_ts", &ts.to_string())
            .append_pair("ebc_sig", &signature);
        Some(target)
    }

    /// Fetches the licence of a `reseller_licence` gift from the store (signed like a
    /// redemption redirect; no redirects followed; at most 1 MiB).
    async fn fetch_licence(&self, gift: &GiftRecord, url: &str) -> Result<Bytes, ClaimError> {
        let target = self.signed_store_url(gift, url).ok_or(ClaimError::StoreUnavailable)?;
        let mut response = self.http.get(target).send().await.map_err(|error| {
            // The URL carries `ebc_sig`, a valid store signature: never log it.
            let error = error.without_url();
            tracing::warn!(gift = %gift.id, %error, "licence request failed");
            ClaimError::StoreUnavailable
        })?;
        if !response.status().is_success() {
            tracing::warn!(gift = %gift.id, status = %response.status(), "the store refused the licence");
            return Err(ClaimError::StoreUnavailable);
        }
        let mut body = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(|_| ClaimError::StoreUnavailable)? {
            if body.len() + chunk.len() > MAX_LICENCE_BYTES {
                return Err(ClaimError::StoreUnavailable);
            }
            body.extend_from_slice(&chunk);
        }
        if body.is_empty() {
            return Err(ClaimError::StoreUnavailable);
        }
        Ok(Bytes::from(body))
    }

    /// Asks the partner of a `partner_file` gift for a download link (`docs/api.md`
    /// §4.5): a signed `GET` of `url` with the parameters of a redemption redirect. The
    /// partner answers `200` with a [`PartnerFileLink`] as JSON, or `303` with the link
    /// in `Location`. The link must be valid and on one of the partner's
    /// `redirect_hosts`. No redirect is followed; neither URL is ever logged (both carry
    /// signatures).
    async fn request_partner_link(
        &self,
        gift: &GiftRecord,
        url: &str,
        format: FileFormat,
    ) -> Result<ResolvedLink, ClaimError> {
        let partner = self.partner_of(gift).ok_or(ClaimError::StoreUnavailable)?;
        let target = self.signed_store_url(gift, url).ok_or(ClaimError::StoreUnavailable)?;
        let unavailable = |reason: &str| {
            tracing::warn!(gift = %gift.id, partner = %partner.id, reason, "no download link from the partner");
            ClaimError::StoreUnavailable
        };
        let mut response = self
            .http
            .get(target)
            .header(reqwest::header::ACCEPT, "application/json")
            .send()
            .await
            .map_err(|error| {
                let error = error.without_url();
                tracing::warn!(gift = %gift.id, %error, "partner file request failed");
                ClaimError::StoreUnavailable
            })?;
        let link = match response.status() {
            reqwest::StatusCode::SEE_OTHER => {
                let location = response
                    .headers()
                    .get(reqwest::header::LOCATION)
                    .and_then(|value| value.to_str().ok())
                    .ok_or_else(|| unavailable("303 without Location"))?;
                PartnerFileLink {
                    download_url: location.trim().to_owned(),
                    file_name: None,
                    media_type: None,
                    size_bytes: None,
                    sha256: None,
                    expires_at: None,
                }
            }
            status if status.is_success() => {
                let mut body = Vec::new();
                while let Some(chunk) =
                    response.chunk().await.map_err(|_| unavailable("unreadable answer"))?
                {
                    if body.len() + chunk.len() > MAX_LINK_ANSWER_BYTES {
                        return Err(unavailable("answer too large"));
                    }
                    body.extend_from_slice(&chunk);
                }
                serde_json::from_slice::<PartnerFileLink>(&body)
                    .map_err(|_| unavailable("answer is not a download link"))?
            }
            status => {
                tracing::warn!(gift = %gift.id, %status, "the partner refused the file");
                return Err(ClaimError::StoreUnavailable);
            }
        };
        if let Err(error) = link.validate(format, self.now()) {
            tracing::warn!(gift = %gift.id, %error, "unusable download link from the partner");
            return Err(ClaimError::StoreUnavailable);
        }
        if !partner.allows_url(&link.download_url) {
            return Err(unavailable("download link outside the partner's redirect_hosts"));
        }
        let url = reqwest::Url::parse(&link.download_url)
            .map_err(|_| unavailable("download link is not a URL"))?;
        Ok(ResolvedLink { url, link })
    }

    /// Keeps a partner's download link for the session `key`, and forgets expired ones.
    fn cache_link(&self, key: &str, resolved: &ResolvedLink) {
        let now = self.now();
        let expires_at =
            resolved.link.expires_at.unwrap_or_else(|| now.saturating_add(LINK_DEFAULT_TTL_SECS));
        let mut links =
            self.partner_links.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        links.retain(|_, link| link.expires_at > now);
        links.insert(key.to_owned(), CachedLink { url: resolved.url.clone(), expires_at });
    }

    /// Where `GET /d/{session}/…` of a partner-file session sends the device: the link
    /// obtained for the session while it has [`LINK_REUSE_MARGIN_SECS`] left, else a
    /// fresh one from the partner.
    pub(crate) async fn partner_download(
        &self,
        key: &str,
        session: &SessionRecord,
    ) -> Result<reqwest::Url, ClaimError> {
        let now = self.now();
        let cached = self
            .partner_links
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(key)
            .filter(|link| link.expires_at.saturating_sub(now) >= LINK_REUSE_MARGIN_SECS)
            .map(|link| link.url.clone());
        if let Some(url) = cached {
            return Ok(url);
        }
        let gift = self.gift(&session.gift_id).ok_or(ClaimError::Gone(GiftStatus::Cancelled))?;
        if gift.status_at(now).is_gone() {
            return Err(ClaimError::Gone(gift.status_at(now)));
        }
        let Delivery::PartnerFile { format, url } = &gift.delivery else {
            return Err(ClaimError::NotAvailable);
        };
        let resolved = self.request_partner_link(&gift, url, *format).await?;
        self.cache_link(key, &resolved);
        Ok(resolved.url)
    }

    /// The bytes of a `partner_file` gift, for a send-to-device e-mail: a fresh download
    /// link, then the file itself, in memory only, at most `EBC_SEND_MAX_MB`, checked
    /// against the size and SHA-256 the partner announced. Never written to disk.
    pub(crate) async fn partner_file_bytes(
        &self,
        gift: &GiftRecord,
    ) -> Result<(Bytes, FileFormat), ClaimError> {
        let Delivery::PartnerFile { format, url } = &gift.delivery else {
            return Err(ClaimError::NotAvailable);
        };
        let resolved = self.request_partner_link(gift, url, *format).await?;
        let limit = self.config.max_send_bytes;
        if resolved.link.size_bytes.is_some_and(|size| size > limit) {
            return Err(ClaimError::TooLargeToSend);
        }
        let failed = |reason: &str| {
            tracing::warn!(gift = %gift.id, reason, "the partner's file could not be fetched");
            ClaimError::StoreUnavailable
        };
        let mut response = self.http.get(resolved.url).send().await.map_err(|error| {
            let error = error.without_url();
            tracing::warn!(gift = %gift.id, %error, "partner file download failed");
            ClaimError::StoreUnavailable
        })?;
        if response.status() != reqwest::StatusCode::OK {
            return Err(failed("the download link did not answer 200"));
        }
        if response.content_length().is_some_and(|length| length > limit) {
            return Err(ClaimError::TooLargeToSend);
        }
        let mut body = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(|_| failed("download cut off"))? {
            if (body.len() + chunk.len()) as u64 > limit {
                return Err(ClaimError::TooLargeToSend);
            }
            body.extend_from_slice(&chunk);
        }
        if body.is_empty() {
            return Err(failed("empty file"));
        }
        if resolved.link.size_bytes.is_some_and(|size| size != body.len() as u64) {
            return Err(failed("size differs from the announced one"));
        }
        if resolved
            .link
            .sha256
            .as_deref()
            .is_some_and(|sha| !sha.eq_ignore_ascii_case(&codes::sha256_hex(&body)))
        {
            return Err(failed("SHA-256 differs from the announced one"));
        }
        Ok((Bytes::from(body), *format))
    }

    /// Mints a download session for a `partner_file` or `reseller_licence` gift.
    pub(crate) async fn open_session(
        &self,
        gift_id: &str,
        device: DeviceClass,
    ) -> Result<Ticket, ClaimError> {
        let gift = self.gift(gift_id).ok_or(ClaimError::Gone(GiftStatus::Cancelled))?;
        self.check_claimable(&gift)?;
        let payload = self.payload(&gift).await?;
        let token = codes::random_hex(self.entropy.as_ref(), 16).map_err(ClaimError::internal)?;
        let session = SessionRecord {
            gift_id: gift.id.clone(),
            file_name: payload.file_name.clone(),
            display_name: payload.display_name,
            media_type: payload.media_type,
            sha256: payload.sha256.clone(),
            size_bytes: payload.size_bytes,
            expires_at: self.now().saturating_add(self.config.session_ttl_secs),
            device: device.as_str().to_owned(),
            licence: payload.licence,
            partner_file: payload.link.is_some(),
        };
        let key = codes::token_hash(&token);
        let session_key = key.clone();
        let outcome = self.record_claim(gift_id, true, false, |db, _| {
            db.sessions.insert(session_key, session);
        });
        let (record, first, stale) = match outcome {
            Ok(done) => done,
            Err(error) => {
                if payload.licence {
                    self.db.delete_unused_blobs(&[payload.sha256]);
                }
                return Err(error);
            }
        };
        if let Some(bytes) = payload.bytes {
            // Again, now that the session protects the licence (see `delete_unused_blobs`).
            self.put_blob(&payload.sha256, bytes).await.map_err(|_| ClaimError::Internal)?;
        }
        if let Some(resolved) = &payload.link {
            self.cache_link(&key, resolved);
        }
        self.db.delete_unused_blobs(&stale);
        if first {
            self.emit(&record, "gift.redeemed");
        }
        self.emit(&record, "gift.downloaded");
        Ok(Ticket { token, file_name: payload.file_name })
    }

    /// Counts the press of `redeem` on a `reseller_redirect` gift and returns where to
    /// send the browser.
    pub(crate) fn redeem_redirect(&self, gift: &GiftRecord) -> Result<reqwest::Url, ClaimError> {
        let Delivery::ResellerRedirect { url } = &gift.delivery else {
            return Err(ClaimError::NotAvailable);
        };
        self.check_claimable(gift)?;
        let target = self.signed_store_url(gift, url).ok_or(ClaimError::StoreUnavailable)?;
        let (record, first, stale) = self.record_claim(&gift.id, false, false, |_, _| {})?;
        self.db.delete_unused_blobs(&stale);
        if first {
            self.emit(&record, "gift.redeemed");
        }
        Ok(target)
    }

    /// Creates the pair code of a gift, replacing any earlier one.
    pub(crate) fn create_pair(&self, gift: &GiftRecord) -> Result<(PairCode, i64), ClaimError> {
        if matches!(gift.delivery, Delivery::ResellerRedirect { .. }) {
            return Err(ClaimError::NotAvailable);
        }
        self.check_claimable(gift)?;
        let expires_at = self.now().saturating_add(self.config.pair_ttl_secs);
        for _ in 0..8 {
            let code =
                codes::random_pair_code(self.entropy.as_ref()).map_err(ClaimError::internal)?;
            let hash = codes::pair_code_hash(&self.config.code_secret, &code);
            let now = self.now();
            let inserted = self
                .db
                .update(|db| {
                    db.pairs.retain(|_, pair| pair.expires_at > now && pair.gift_id != gift.id);
                    if db.pairs.contains_key(&hash) {
                        return false;
                    }
                    db.pairs.insert(hash, PairRecord { gift_id: gift.id.clone(), expires_at });
                    true
                })
                .map_err(ClaimError::internal)?;
            if inserted {
                return Ok((code, expires_at));
            }
        }
        Err(ClaimError::internal("could not draw an unused pair code"))
    }

    /// Consumes a pair code: the gift it stood for, once, if it is still valid.
    pub(crate) fn take_pair(&self, code: &PairCode) -> Result<Option<String>, StoreError> {
        let hash = codes::pair_code_hash(&self.config.code_secret, code);
        let now = self.now();
        if !self.db.view(|db| db.pairs.contains_key(&hash)) {
            return Ok(None);
        }
        self.db.update(|db| {
            db.pairs.remove(&hash).filter(|pair| pair.expires_at > now).map(|pair| pair.gift_id)
        })
    }

    /// Reserves one sending of the partner file of `gift_id` to a device address: the gift
    /// must be claimable, below its claim limit and sent fewer than [`MAX_SENDS_PER_DAY`]
    /// times in the last 24 hours. The claim and the sending are counted **now**, in one
    /// transaction, before any mail is sent; a failed delivery hands them back with
    /// [`release_send`](Self::release_send), a successful one is announced with
    /// [`confirm_send`](Self::confirm_send). The address itself is never kept.
    pub(crate) fn reserve_send(&self, gift_id: &str) -> Result<SendReservation, ClaimError> {
        let before = self.gift(gift_id).ok_or(ClaimError::Gone(GiftStatus::Cancelled))?;
        if !matches!(before.delivery, Delivery::PartnerFile { .. }) {
            return Err(ClaimError::NotAvailable);
        }
        let (record, first, stale) = self.record_claim(gift_id, true, true, |_, _| {})?;
        self.db.delete_unused_blobs(&stale);
        Ok(SendReservation {
            at: record.last_claim_at.unwrap_or_else(|| self.now()),
            first,
            record,
        })
    }

    /// Hands back a reservation whose mail could not be sent: the claim and the sending
    /// no longer count.
    pub(crate) fn release_send(&self, reservation: SendReservation) {
        let cap = self.config.max_download_sessions;
        let outcome = self.db.update(|db| {
            let Some(gift) = db.gifts.get_mut(&reservation.record.id) else { return };
            if let Some(index) = gift.sends.iter().rposition(|at| *at == reservation.at) {
                gift.sends.remove(index);
            }
            gift.claims = gift.claims.saturating_sub(1);
            match gift.status {
                GiftStatus::Redeemed | GiftStatus::Exhausted
                    if reservation.first && gift.claims == 0 =>
                {
                    gift.status = GiftStatus::Created;
                    gift.redeemed_at = None;
                    gift.last_claim_at = None;
                }
                GiftStatus::Exhausted if cap == 0 || gift.claims < cap => {
                    gift.status = GiftStatus::Redeemed;
                }
                _ => {}
            }
        });
        if let Err(error) = outcome {
            tracing::warn!(gift = %reservation.record.id, %error, "a failed sending could not be handed back");
        }
    }

    /// Announces a sending that was delivered to the mail server (webhooks).
    pub(crate) fn confirm_send(&self, reservation: &SendReservation) {
        if reservation.first {
            self.emit(&reservation.record, "gift.redeemed");
        }
        self.emit(&reservation.record, "gift.downloaded");
    }

    /// Attaches a stored file (cover, card PDF) to a gift; the previous one is deleted.
    /// `cover_rights`, when given with a cover, replaces the rights basis declared for it.
    pub(crate) async fn attach_blob(
        &self,
        gift_id: &str,
        blob: BlobRef,
        bytes: Bytes,
        cover: bool,
        cover_rights: Option<crate::card::CoverRights>,
    ) -> Result<(), ApiError> {
        let sha256 = blob.sha256.clone();
        self.put_blob(&sha256, bytes.clone()).await?;
        let previous = self.db.update(|db| {
            let gift = db.gifts.get_mut(gift_id)?;
            if let (true, Some(rights)) = (cover, cover_rights) {
                gift.cover_rights = rights;
            }
            let slot = if cover { &mut gift.cover } else { &mut gift.card_pdf };
            Some(slot.replace(blob))
        })?;
        let Some(previous) = previous else {
            self.db.delete_unused_blobs(&[sha256]);
            return Err(ApiError::not_found("no such gift"));
        };
        // Again, now that the gift protects the file (see `delete_unused_blobs`).
        self.put_blob(&sha256, bytes).await?;
        self.db.delete_unused_blobs(&previous.into_iter().map(|b| b.sha256).collect::<Vec<_>>());
        Ok(())
    }
}
