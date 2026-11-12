# eBook Cadeau — HTTP contract (server, partner API, redemption pages)

_Status: v0.3 — the contract the server, the demo stores, the simulated e-readers and the
end-to-end tests are all built against. The redemption UX is justified in
`docs/research/05-ereader-distribution-ux.md`._

_Changes in v0.3 (owner decision, the **two-service model**): the core service — this
contract — **never holds a book file**. `hosted_file`, the upload routes and the creator
API are removed. A file is only ever served by a registered **partner**: a store, or a
separate "shelf" service that hosts files for their rights holders and is registered here
exactly like a store (same signed API, same `EBC_PARTNERS` registry). New delivery
`partner_file` (§4.5): on each claim the core obtains a short-lived signed download link
from the partner and redirects the device to it. Print orders move to the partner API
(§4.6). Names and message are deleted 12 months after the first claim (§2, "Claim
semantics"). A contact point (`EBC_CONTACT_EMAIL`) is shown on the pages. Why: a service
storing users' book files is a hosting provider with DSA duties (notice and action,
statements of reasons, repeat-infringer policy) and a fragile liability exemption when it
knows which work each file is (`docs/research/09-legal-gifting-and-commerce.md` §2.6, §3);
those duties now sit with the shelf, a separate service, and the core only keeps gift
records._

_Changes in v0.2 (from the UX research): letters-only 12-symbol code; upper-case QR payload and
case-insensitive `/k/`; 6-letter pair code with a global failure budget; explicit device chooser
(user-agent sniffing is brittle); claim count shown on the gift page; reissue; optional reveal date._

Conventions: JSON bodies are UTF-8; timestamps are Unix seconds; errors are
`{"error": {"code": "snake_case", "message": "…"}}` with a matching HTTP status. HTML pages
work **without JavaScript** and carry stable element `id`s (listed below) used by tests.

## 1. Actors and vocabulary

| Term | Meaning |
|---|---|
| **Gift** | One gifted eBook: metadata, a redemption **code**, and a **delivery** method |
| **Code** | `GiftCode`: 12 symbols from the 20-consonant alphabet `BCDFGHJKLMNPQRSTVWXZ` (11 random + 1 check, 47.5 bits), shown as `XXXX-XXXX-XXXX`. Letters only: no keyboard layer switch on e-ink or phone keyboards, no 0/O or 1/I confusion, no vowels so no accidental words. Input is case-insensitive and ignores spaces, hyphens, dots and underscores |
| **Claim URL** | `{PUBLIC_URL}/k/{CODE}` — printed in clear on the card (lower-case host). The QR code encodes the same URL fully upper-cased (`HTTPS://HOST/K/CODE`) to stay in QR alphanumeric mode, so the server treats `/K/` and `/k/` alike and `PUBLIC_URL` must have a letters-only path-less host |
| **Giver / recipient** | The buyer and the person who receives the card |
| **Partner** | A registered service that creates gifts through the signed partner API (§4): an eBook reseller integrating the "eBook cadeau" option, or a **shelf** — a separate service hosting files for their rights holders (authors, publishers, public-domain and openly licensed works). Every gift belongs to a partner |
| **Delivery** | `partner_file` (the partner serves the file; the core redirects the device to a short-lived link the partner issues on each claim, §4.5), `reseller_redirect` (the reseller adds the book to the recipient's account), `reseller_licence` (the core hands over a licence file such as `.lcpl`/`.acsm` issued by the reseller). The core never stores a book |
| **Download session** | A random 128-bit URL token minted after a successful claim, short-lived (15 minutes). For `partner_file` its URL redirects to the partner's link; for `reseller_licence` it serves the licence |
| **Pair code** | 6 letters from the same alphabet, shown as `XXX-XXX`: links a phone that scanned the QR to an e-reader's browser. Single use, 15 minutes |

## 2. Redemption pages (recipient side, no JavaScript required)

All gift pages send `Cache-Control: no-store`, `Referrer-Policy: no-referrer`,
`X-Robots-Tag: noindex, nofollow`, `X-Content-Type-Options: nosniff` and a
`Content-Security-Policy` without inline script. No cookies are needed.
Pages are served in the gift's locale (fallback: `Accept-Language`, then French).
HTML must render in decade-old WebKit (no flexbox/grid dependence, no web fonts, page
weight < 30 kB excluding the optional cover thumbnail).

| Route | Purpose | Key element ids |
|---|---|---|
| `GET /` | Home with the code form (for people typing the short URL on an e-reader) | `code-form`, `code-input` (name `code`), `code-submit` |
| `GET /k?code=…` | Normalises the typed code — or a pasted claim URL, with or without its scheme (`ebk.example/k/…`, `/k/…`; `GiftCode::parse_typed`, the rule the web app's code field shares) — and 303-redirects to `/k/{CODE}` | — |
| `GET /aide` | Help page: the three ways to get the book, per device; the contact point when `EBC_CONTACT_EMAIL` is set | `help`, `contact-point` |
| `GET /k/{code}` (`/K/` too; optional `?device=`) | The gift page: title, author, giver, message, the primary action for this device, the device chooser, and how many times the gift was already claimed | `gift-title`, `gift-author`, `gift-from`, `gift-message`, `primary-action`, `other-ways`, `device-chooser` (links `device-kobo`, `device-kindle`, `device-pocketbook`, `device-tolino`, `device-phone`, `device-computer`), `claim-count` |
| `POST /k/{code}/download` | `partner_file` only. Asks the partner for a download link (§4.5), mints a download session and 303-redirects to `/d/{session}/{file-name}`. When the partner gives no usable link: `502`, the gift page with an error, **no claim counted** | form id `download-form`, button id `download` |
| `GET\|HEAD /d/{session}/{file-name}` | `partner_file`: `303 See Other` to the partner's link (the one obtained at the claim while it has ≥ 10 s left, else a fresh one; the core never relays the bytes); `502` page when the partner gives none. `reseller_licence`: the licence file — `Content-Type` per format, `Content-Disposition: attachment` with ASCII fallback + `filename*`, `Content-Length`, `Accept-Ranges`, `Range` | — |
| `POST /k/{code}/pair` | From a phone: creates a pair code and shows it with the e-reader instructions and its expiry time | `pair-code`, `pair-url`, `pair-expires` |
| `GET /e` | On the e-reader: form asking for the pair code | `pair-form`, `pair-input` (name `p`), `pair-submit` |
| `POST /e` | Validates the pair code, mints a session (as `download`) and 303-redirects to `/d/{session}/{file-name}`. Wrong codes count against a per-client **and a global** failure budget (`429` + `Retry-After`); while the global budget is spent no pair code is looked up (`429` for everybody) | — |
| `POST /k/{code}/send` | `partner_file`: sends the file to a device address (`device_email`), e.g. Send-to-Kindle; allowed domains are configured. The core obtains a link (§4.5), downloads the file **into memory only** (at most `EBC_SEND_MAX_MB`, default 50 MB — Send to Kindle's own limit; the announced size and SHA-256 are checked) and attaches it; nothing is written to disk. Too large: `502` "download it instead"; partner failure: `502`; neither counts a claim | form id `send-form`, input id `device-email`, result id `send-result` |
| `POST /k/{code}/redeem` | `reseller_redirect`: 303 to the partner URL with signed parameters (§4.3). `reseller_licence`: 303 to a download session serving the licence file | button id `redeem` |
| `GET /o/{session}/catalog.xml` | OPDS 1.2 acquisition feed containing the one publication. The acquisition link is the core's session URL `/d/{session}/{file-name}` (which redirects to the partner's link); `length` only when the partner announced the size | — |
| `GET /o/{session}/catalog.json` | OPDS 2.0 feed, same acquisition link | — |
| `GET /healthz` | `200 ok` | — |

Status codes on `/k/{code}`: `200`; `400` malformed code (bad check symbol — the page says it
looks like a typo and **does not** count as an attempt); `404` well-formed but unknown (counts
towards the rate limit); `410` cancelled or expired; `429` too many unknown codes from this
client (`Retry-After`). Error pages say how many attempts remain and how long to wait.
Before an optional reveal date (`reveal_at`) the gift page shows the date and no action.

### Device classes

The server classifies the request into `kobo`, `kindle`, `pocketbook`, `tolino`,
`eink_other`, `mobile`, `desktop`. **User-agent sniffing is a hint, never the only way**: it
broke twice between 2024 and 2026 (newer Kobo and Kobo-based tolino firmware report `eReader`;
some recent Kindle firmware no longer says `Kindle`). So every gift page carries a device
chooser, and `?device=kobo|kindle|pocketbook|tolino|phone|computer` overrides the detection
(the choice is carried in links and forms; no cookie).

| Class | Primary action for `partner_file` | Then |
|---|---|---|
| `kobo`, `pocketbook`, `tolino`, `eink_other` | Direct **download** button | the built-in browser saves EPUB to the library |
| `kindle` | If the file is in a format the Kindle browser saves (AZW3/MOBI/TXT): download. For EPUB: instructions to use **Send to Kindle** — from a phone via the Kindle app share sheet, or the e-mail form (explain the approved-sender step) | — |
| `mobile` | **Read on this phone or send to an app** (download; the OS share sheet reaches the Kindle, Kobo or Books apps) | "On your e-reader": open `{host}` in its browser and type the code printed on the card (the default e-reader path, no phone needed) → pair code (shorter to type) → Send to Kindle by e-mail |
| `desktop` | Download | e-reader instructions, pairing, e-mail, USB copy hint |

For `reseller_redirect` gifts the primary action on every class is "add to my {store} library"
(nothing is typed on the e-reader: the phone signs in, the e-reader syncs).

### Claim semantics

- States: `created` → `redeemed` (first claim) → `exhausted` (file: session limit
  reached) or `fulfilled` (reseller confirmed); plus `cancelled`, `expired`, `reissued`.
- A `partner_file` gift is **re-downloadable by whoever holds the card** until it expires
  (default: never) or a limit of download sessions is reached (default 10). Every session
  is a claim, whatever the route (typed code, pair code, Send to Kindle, OPDS); the
  partner is asked for a link on each claim and may refuse (e.g. its own limits). The
  gift page shows "claimed n times, last on …" so misuse is visible to the holder.
- **Reissue**: the giver (manage token) or the partner can void the code and get a new one
  for the same gift (lost card): the old code answers `410`.
- No account, e-mail address or cookie is needed for `partner_file` gifts: holding the
  card is the credential.
- **Retention of personal data** (`docs/research/09` §9.1): the giver's and recipient's
  names and the message are deleted **12 months after the first claim**
  (`EBC_RETENTION_AFTER_CLAIM_DAYS`, default 365). The gift keeps working; its page then
  shows no names and no message ("a gift for you"). A cancelled gift loses them at once;
  a cancelled or expired gift is anonymised `EBC_RETENTION_DAYS` (30) days later (§6).
- `reseller_redirect` gifts are single-recipient: once the partner confirms fulfilment
  (§4.4) the gift page only says the book is already in an account and links to the store.
- Codes are stored hashed (HMAC-SHA-256 with a server secret); the clear code exists only
  in the creation response and on the card.

## 3. Standalone mode: a shelf service, registered as a partner

_The creator API of v0.2 (`POST /api/v1/uploads`, `POST /api/v1/gifts` with a manage
token, `GET|DELETE /api/v1/gifts/{id}`, `…/reissue`, `…/card.pdf`, `…/print-orders`) and
`POST /api/partner/v1/uploads` are **removed**: they answer `404 not_found`. There is no
unsigned way to create a gift._

Standalone gifts (an author's own work, a public-domain or openly licensed book) are
created by a **shelf**: a separate service that stores the files of their rights holders,
carries the hosting duties (uploader identity and declaration, DRM refusal, notice and
action under DSA art. 16, statements of reasons, repeat-infringer policy —
`docs/research/09-legal-gifting-and-commerce.md` §3.3), and is registered with the core
in `EBC_PARTNERS` exactly like a store. It creates `partner_file` gifts on behalf of its
users (§4.2), serves the files through its own signed download endpoint (§4.5), and may
order printed cards (§4.6). The reusable checks for uploaded files (EPUB/PDF sniffing,
DRM detection, rights declaration, safe file names) are the library `crates/ebc-epub`.

What a shelf must implement to be a partner: the signed requests of §4.1 (as client), a
`partner_file` endpoint answering §4.5 (as server), and, optionally, webhooks (§4.4).

## 4. Partner API (stores and shelves)

### 4.1 Authentication

Every request carries:

```
X-EBC-Partner: <partner id>
X-EBC-Timestamp: <unix seconds>
X-EBC-Signature: v1=<hex HMAC-SHA-256(secret, "{timestamp}.{METHOD}.{path-with-query}.{hex sha256(body)}")>
Idempotency-Key: <opaque, optional but recommended on POST>
```

Requests older or newer than 300 s are rejected (`401 stale_timestamp`); a wrong signature
is `401 bad_signature`. The signature must be `v1=` followed by exactly 64 hexadecimal
digits: anything else, an unknown partner or a stale timestamp is refused **before the
body is read**. Failed authentications are counted per client network: beyond
`EBC_RATE_PARTNER_FAIL_PER_10MIN` (default 30 per 10 minutes) the API answers `429` with
`Retry-After` without looking at the request. Request bodies are hashed as they arrive and
count against a server-wide in-flight budget (`503 server_busy` + `Retry-After` when it is
spent).

**Each signature is single-use (nonce).** The server keeps every verified signature of a
`POST` or `PUT` — as the decoded MAC, so the case of the hex digits does not matter — until
its timestamp has left the 300 s window. The same signature again (an exact retry, or a
replay by whoever captured the request) is answered with the response the first request
got, success or error, with the header `Idempotent-Replayed: true`, and never acts twice;
headers outside the signature (`Idempotency-Key`, `Content-Type`) change
nothing. Consequences for partners:

- to retry a request **after a failure**, or after correcting an unsigned header, sign it
  again with a new timestamp (the next second);
- two *deliberately* identical requests within one second (e.g. two reissues of the same
  gift) carry the same signature and count as one: wait a second, or add a signed query
  parameter `nonce=<anything>` (the server ignores it; it only makes the signatures differ);
- `GET` requests only read and may be repeated.

`Idempotency-Key` is an additional key for retries signed anew: the same key with the same
body returns the original *successful* response (failed requests are not remembered under
the key); with a different body, `409 idempotency_conflict`. Keys are remembered 24 h.

### 4.2 Gifts

`POST /api/partner/v1/gifts`

```json
{
  "order_ref": "ORD-2026-000123",
  "book": { "title": "…", "isbn13": "…", "contributors": [{"name": "…"}], "publisher": "…", "language": "fr" },
  "giver_name": "…", "recipient_name": "…", "message": "…", "locale": "fr",
  "delivery": { "type": "reseller_redirect", "url": "https://store.example/gift/redeem" }
}
```

`delivery` is one of:

| `delivery` | Meaning |
|---|---|
| `{"type": "partner_file", "format": "epub" \| "pdf", "url": "https://…"}` | The partner serves the file; `url` is its signed file endpoint (§4.5) |
| `{"type": "reseller_redirect", "url": "https://…"}` | The store adds the book to the recipient's account (§4.3) |
| `{"type": "reseller_licence", "format": "lcpl", "url": "https://…"}` | The store issues a licence file, fetched like §4.5 and served through a session |

Every `url` must be an `http(s)` URL on one of the partner's `redirect_hosts`
(`422 redirect_host_not_allowed`); an unknown `format` is `422 invalid_delivery`. Optional
members: `cover_rights`, `reveal_at`, `expires_at` (Unix seconds).

→ `201 {"id", "code", "code_display", "claim_url", "status": "created"}`

`PUT /api/partner/v1/gifts/{id}/cover` — body: the cover image the partner is entitled to
print (JPEG/PNG/WebP, ≤ 20 MB). The server never fetches third-party URLs itself.

`GET /api/partner/v1/gifts/{id}/card.pdf?format=poche|a6|digest|a5&profile=pro|home-a4|home-letter`
→ the print-ready card rendered server-side with the same library the browser uses. `pro`
is the upload profile: PDF/X-4, FOGRA39, every colour built under **240 % total ink** (dark
colours rebuilt with more black and less colour ink, not lightened).

When the composer has something to say about the card, the response carries
`X-EBC-Card-Warnings: <code>, <code>, …` — stable `snake_case` codes, in the order they were
raised, each once (absent when there is nothing to say). The card is printable either way;
the codes are the `CardWarning::code` values of `ebc-card`, e.g.:

| Code | Meaning, what to do |
|---|---|
| `dark_across_creases` | Dark or heavy ink (above 180 % or darker than L\* 35) lies within 2 mm of a crease: order **matt or soft-touch lamination** and the **paper grain parallel to the spine**, or the folds may crack |
| `cover_credit_missing` | No credit line for the cover and no publisher: a generic credit is printed |
| `low_resolution` | The cover prints below 250 ppi (a cover below 150 ppi is refused with `422 card_unprocessable`) |
| `cover_downsampled`, `heavy_crop`, `text_clamped`, `missing_glyphs`, `steps_reduced`, `code_reduced`, `qr_version_high`, … | See `crates/ebc-card/src/error.rs` |

`GET /api/partner/v1/gifts/{id}` →
`{"id", "status": "created|redeemed|exhausted|fulfilled|cancelled|expired", "created_at", "redeemed_at", "downloads", "order_ref"}`
(`downloads`: claims so far).
`POST /api/partner/v1/gifts/{id}/cancel` → voids the code (refund path).
`POST /api/partner/v1/gifts/{id}/reissue` → `200 {"code", "code_display", "claim_url"}`;
the previous code is voided and answers `410` (lost card). Only a gift that can still be
claimed is reissued (`409 not_reissuable`).

### 4.3 Redemption redirect (`reseller_redirect`)

When the recipient presses `redeem`, the browser is sent to

```
{delivery.url}?ebc_gift={id}&ebc_order={order_ref}&ebc_ts={unix}&ebc_sig={hex HMAC-SHA-256(secret, "{id}.{order_ref}.{ts}")}
```

The store verifies the signature and timestamp (≤ 900 s), lets the recipient sign in or
create an account, adds the book to that account, then confirms:

### 4.4 Fulfilment confirmation and webhooks

`POST /api/partner/v1/gifts/{id}/fulfilled` `{"account_ref": "opaque"}` → `200`. A second
confirmation with a different `account_ref` is `409 already_fulfilled`.

If the partner registered a webhook URL, the server POSTs signed events
(`X-EBC-Signature` computed as in §4.1 with the partner secret):
`{"event": "gift.redeemed" | "gift.downloaded" | "gift.cancelled", "gift_id", "order_ref", "at"}`.
Deliveries are retried with backoff; receivers must be idempotent.

### 4.5 File delivery (`partner_file`)

The partner serves the file; the core never stores or relays it. On **each claim** (the
download button after a typed code or a scanned QR code, a pair code at `/e`, the OPDS
button, Send to Kindle), and again when a session URL is used after the link obtained for
it has (almost) expired, the core asks the partner for a short-lived download link.

**Request** (core → partner): `GET {delivery.url}` with the parameters of a redemption
redirect (§4.3) appended to the stored URL, and `Accept: application/json`:

```
GET {delivery.url}?ebc_gift={id}&ebc_order={order_ref}&ebc_ts={unix}&ebc_sig={hex HMAC-SHA-256(secret, "{id}.{order_ref}.{ts}")}
```

The partner verifies the signature and the timestamp (a window of at most 900 s, as §4.3;
the core sends its current time), that the gift is its own and still claimable, then
answers within `outbound_timeout` (15 s) with **either**

- `200` and a JSON **download link** (at most 64 KiB):

  ```json
  { "download_url": "https://files.partner.example/dl/7f3a…/candide.epub",
    "file_name": "candide.epub", "media_type": "application/epub+zip",
    "size_bytes": 345678, "sha256": "9f86…0a08", "expires_at": 1794441900 }
  ```

  | Member | Rule |
  |---|---|
  | `download_url` | required; absolute `http(s)`; its host must be one of the partner's `redirect_hosts`; short-lived and unguessable (it is a bearer link handed to the device) |
  | `file_name` | optional; a plain name (no path, no control character, ≤ 255 bytes); offered to phones and computers |
  | `media_type` | optional; must be the one of `format` (`application/epub+zip`, `application/pdf`) |
  | `size_bytes` | optional; ≥ 1; shown as the OPDS `length`, checked by Send to Kindle |
  | `sha256` | optional; 64 hex digits; checked by Send to Kindle |
  | `expires_at` | optional; Unix seconds, in the future. Without it the core assumes 60 s |

- or `303 See Other` with the download link in `Location` (no metadata).

Anything else — another status, a malformed answer, a `media_type` of another format, an
expired link, a `download_url` outside `redirect_hosts` — is "no link": the recipient sees
a `502` page ("the store could not deliver the book right now"), **no claim is counted**
and no device is sent anywhere. The core **never follows redirects** of the partner, and
logs neither the signed request URL nor the download link (only the gift id and the kind
of failure).

**`form-action`**: browsers check every hop of a form's redirect chain against the
page's `Content-Security-Policy: form-action`. The gift page of a `partner_file` gift
therefore allows `'self'` plus `http://` and `https://` on each of the partner's
`redirect_hosts` (`host:*` when the entry names no port); the pair form at `/e`, which
does not know the gift yet, allows those of every partner. Other pages keep
`form-action 'self'`.

**Redirect** (core → device): the claim mints a download session and answers `303` to
`/d/{session}/{file-name}`, which answers `303` to `download_url` (`Cache-Control:
no-store`, `Referrer-Policy: no-referrer`). The link obtained at the claim is reused for
the session while it has at least 10 s left (kept in memory only, never in the database);
then a fresh one is requested. `HEAD` gets the same `303`. The partner should serve the
file with `Content-Type`, `Content-Length` and `Content-Disposition: attachment` and accept
`GET` and `HEAD`; plain `http` is fine on a loopback demo, `https` elsewhere.

**Send to Kindle**: the core requests a link, then `GET`s it itself (no redirects
followed, `200` required), reads at most `EBC_SEND_MAX_MB` (default 50 MB) into memory,
checks `size_bytes` and `sha256` when given, attaches the bytes to the mail and drops
them. Nothing is written to disk.

**Watermarks**: a DRM-free store typically stamps the copy at this request (claim time)
with the order reference, as the demo store "Septante" does. Each claim gets its own copy
and link.

**What the partner learns, and the Referer caveat**: the gift id, its order reference and
the time of each claim — never the recipient's identity. E-reader browsers that ignore
`Referrer-Policy` (several old Kobo and PocketBook firmwares) may send the gift page's URL,
which contains the code, as `Referer` when they follow the redirect to `download_url`.
That host is the partner's own (`redirect_hosts`), which issued the code; partners must
not log or forward `Referer` on their download hosts.

### 4.6 Print orders

`PUT /api/partner/v1/gifts/{id}/card.pdf` — body: a card PDF the partner composed itself
(e.g. in the card designer), at most 64 MB, starting with `%PDF-` → `200 {"sha256",
"size_bytes"}`. (`GET` on the same path renders the card server-side, §4.2.)

`POST /api/partner/v1/gifts/{id}/print-orders` — body
`{"product": "card-poche", "pdf_sha256": "…", "address": {…}, "quantity": 1}`
(`quantity` optional, 1–1000, default 1). The server forwards the uploaded PDF to the
configured print provider (`EBC_PRINTER_URL`, §5) and returns
`201 {"order_id", "status", "price_cents", "currency"}`. Both requests are signed (§4.1)
and single-use like every `POST`/`PUT`.

The **postal address** (`address`, the same object the print provider takes in §5):

```json
{ "name": "Sacha Martin", "line1": "12 rue des Lilas", "line2": "Bâtiment B",
  "postal_code": "69003", "city": "Lyon", "country": "FR" }
```

| Member | Rule |
|---|---|
| `name` | required, the addressee, 1–80 characters |
| `line1` | required, street and number, 1–120 characters |
| `line2` | optional, at most 120 characters; empty or `null` is omitted |
| `postal_code` | required, 1–12 characters |
| `city` | required, 1–80 characters |
| `country` | optional, ISO 3166-1 alpha-2 (case-insensitive, sent upper-cased); default `FR` |

Every member is a string; text is trimmed, characters are counted (not bytes), control
characters and any other member are refused. The server forwards the normalised object.
Whether the printer posts to the country is the printer's answer (`unsupported_country`
below), not the server's.

Errors of these routes:

| Status / code | When |
|---|---|
| `400 invalid_address` | the address is not an object, misses a required member, has a member of the wrong type, too long, with a control character, unknown (e.g. `postcode`), or `country` is not two letters; `error.field` names it (`"address"`, `"address.postal_code"`, …) |
| `422 invalid_request` | `product` is not a product id, or `quantity` is out of range |
| `409 card_pdf_missing`, `409 pdf_mismatch` | no card uploaded, or `pdf_sha256` is not the uploaded card's |
| `409 gift_gone` | the gift is cancelled or expired |
| `415 unsupported_media_type` | the uploaded card is not a PDF |
| `503 printing_not_configured` | no `EBC_PRINTER_URL` |
| `422 preflight_failed` | the printer's preflight refused the PDF (§5); `error.findings` are the printer's |
| `422 printer_rejected` | the printer refused the order with any other `4xx` (`unsupported_country`, `unknown_product`, `invalid_quantity`, `invalid_address`, `duplicate_reference`, …) |
| `502 printer_unavailable` | the printer did not answer, answered `5xx`, or answered `2xx` without an `order_id` |

Both `422` refusals carry the printer's answer: `error.printer_status` (its HTTP status),
`error.printer_code` (its `error.code`, or `http_<status>` when it gave none usable),
`error.printer_message` (its message, at most 300 characters, when given) and
`error.findings` (its findings, or `[]`). Nothing is recorded on the gift unless the
printer accepted the order.

## 5. Print provider interface (what the fictional printer implements)

| Route | Purpose |
|---|---|
| `GET /v1/products` | Catalogue: `[{ "id": "card-poche", "trim_mm": [230, 178], "bleed_mm": 3, "spine_mm": 10, "paper": "350 g/m² …", "price_cents": {"1": 390, "10": 1900} }]` |
| `POST /v1/orders` | `multipart/form-data`: part `order` (JSON: `product`, `quantity`, `address`, `reference`, optional `callback_url`) and part `file` (the PDF). `address` is the object of §4.6 — `{name, line1, line2?, postal_code, city, country}`, `country` an ISO 3166-1 alpha-2 code. The printer **preflights** the PDF against the product: `201 {"order_id", "status": "accepted", "price_cents", "currency"}` or `422 {"error": {"code": "preflight_failed", "findings": [...]}}`. Other refusals are `{"error": {"code", "message"}}` with a `4xx`: `400 invalid_order` / `invalid_multipart` / `missing_part` (unreadable request, e.g. an address member missing), `422 unknown_product`, `invalid_quantity`, `invalid_reference`, `invalid_address`, `unsupported_country`, `invalid_callback_url`, `409 duplicate_reference`, `413` too large. The eBook Cadeau server passes them on as `422 printer_rejected` (§4.6) |
| `GET /v1/orders/{id}` | `{"status": "accepted|printing|shipped|cancelled", "tracking": …}` |
| Webhook | Optional `callback_url` in the order: status changes are POSTed there |

## 6. Configuration (server)

| Variable | Default | Meaning |
|---|---|---|
| `EBC_BIND` | `127.0.0.1:8080` | listen address |
| `EBC_PUBLIC_URL` | `http://127.0.0.1:8080` | base of claim URLs (keep it short: it is typed by hand) |
| `EBC_DATA_DIR` | `./var` | database, partners' covers, card PDFs kept for print orders, licence files of live sessions — never a book |
| `EBC_CODE_SECRET` | — (required outside dev) | HMAC key for code hashing |
| `EBC_PARTNERS` | — | path to a TOML file of partners — stores and shelves alike (`id`, `name`, `secret`, `webhook_url`, `redirect_hosts`: the hosts every delivery URL and every `partner_file` download link must be on) |
| `EBC_SMTP_URL` | — | `smtp://host:port` used for send-to-device; unset disables the feature |
| `EBC_SEND_FROM` | `livraison@localhost` | sender address recipients must approve (Send to Kindle) |
| `EBC_SEND_DOMAINS` | `kindle.com,pbsync.com` | allowed device e-mail domains |
| `EBC_SEND_MAX_MB` | `50` | largest partner file fetched (into memory only) and attached to a send-to-device mail |
| `EBC_PRINTER_URL` | — | print provider base URL |
| `EBC_RATE_UNKNOWN_PER_10MIN` | `10` | unknown-code attempts per client before `429` |
| `EBC_RATE_UNKNOWN_GLOBAL_PER_10MIN` | `500` | unknown gift or pair codes from all clients together before every further wrong guess gets `429` (codes are bearer secrets not tied to an account, so a guess is a guess against all live codes). While it is spent, **pair codes are not looked up at all** (`429` for everybody); gift codes stay budgeted per client network |
| `EBC_RATE_PARTNER_FAIL_PER_10MIN` | `30` | failed partner authentications per client network before `/api/partner/*` answers `429` (0 = off) |
| `EBC_RETENTION_DAYS` | `30` | a cancelled gift loses names and message at once; a cancelled or expired gift is anonymised (names, message, cover and card files removed; id, status, dates and the code hash kept so the code still answers `410`) this many days after cancellation or expiry |
| `EBC_RETENTION_AFTER_CLAIM_DAYS` | `365` | names and message of a gift are deleted this many days after its first claim; the gift keeps working (≥ 1) |
| `EBC_CONTACT_EMAIL` | — | the core's single point of contact (DSA arts 11–12; notices about a gift record), shown in the footer of every page and on `/aide`. The full notice-and-action machinery (art. 16) belongs to the shelf, which holds the files |
| `EBC_MAX_INFLIGHT_BODY_MB` | `256` | request-body bytes held by all requests together (never less than one largest body, 64 MiB); beyond, `503 server_busy` |
| `EBC_HEADER_TIMEOUT_SECS` | `30` | time to send the request line and headers (also closes idle keep-alive connections) |
| `EBC_REQUEST_TIMEOUT_SECS` | `300` | time until the response starts, body upload included (`408`) |
| `EBC_BODY_IDLE_TIMEOUT_SECS` | `30` | longest pause inside a request body (`408`) |
| `EBC_MAX_CONNECTIONS` | `1024` | connections served at once; more wait in the listen backlog |
| `EBC_WEB_DIR` | — | directory of the static web app, served at `/app/` (`.wasm` as `application/wasm`) |
| `EBC_TRUST_PROXY` | `0` | honour `X-Forwarded-For` for client identification (only behind a trusted reverse proxy) |
