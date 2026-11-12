# eBook Cadeau — HTTP contract (server, partner API, redemption pages)

_Status: v0.2 — the contract the server, the demo stores, the simulated e-readers and the
end-to-end tests are all built against. The redemption UX is justified in
`docs/research/05-ereader-distribution-ux.md`._

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
| **Partner** | An eBook reseller integrating the "eBook cadeau" option |
| **Delivery** | `hosted_file` (we serve the file), `reseller_redirect` (the reseller adds the book to the recipient's account), `reseller_licence` (we hand over a licence file such as `.lcpl`/`.acsm` issued by the reseller) |
| **Download session** | A random 128-bit URL token minted after a successful claim, short-lived |
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
| `GET /aide` | Help page: the three ways to get the book, per device | `help` |
| `GET /k/{code}` (`/K/` too; optional `?device=`) | The gift page: title, author, giver, message, the primary action for this device, the device chooser, and how many times the gift was already claimed | `gift-title`, `gift-author`, `gift-from`, `gift-message`, `primary-action`, `other-ways`, `device-chooser` (links `device-kobo`, `device-kindle`, `device-pocketbook`, `device-tolino`, `device-phone`, `device-computer`), `claim-count` |
| `POST /k/{code}/download` | `hosted_file` only. Mints a download session and 303-redirects to `/d/{session}/{file-name}` | form id `download-form`, button id `download` |
| `GET /d/{session}/{file-name}` | The file. `Content-Type` per format (`application/epub+zip`), `Content-Disposition: attachment` with ASCII fallback + `filename*`, `Content-Length`, `Accept-Ranges`. `HEAD` supported | — |
| `POST /k/{code}/pair` | From a phone: creates a pair code and shows it with the e-reader instructions and its expiry time | `pair-code`, `pair-url`, `pair-expires` |
| `GET /e` | On the e-reader: form asking for the pair code | `pair-form`, `pair-input` (name `p`), `pair-submit` |
| `POST /e` | Validates the pair code and 303-redirects to `/d/{session}/{file-name}`. Wrong codes count against a per-client **and a global** failure budget (`429` + `Retry-After`); while the global budget is spent no pair code is looked up (`429` for everybody) | — |
| `POST /k/{code}/send` | Sends the file to a device address (`device_email`), e.g. Send-to-Kindle; allowed domains are configured | form id `send-form`, input id `device-email`, result id `send-result` |
| `POST /k/{code}/redeem` | `reseller_redirect`: 303 to the partner URL with signed parameters (§4.3). `reseller_licence`: 303 to a download session serving the licence file | button id `redeem` |
| `GET /o/{session}/catalog.xml` | OPDS 1.2 acquisition feed containing the one publication | — |
| `GET /o/{session}/catalog.json` | OPDS 2.0 feed | — |
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

| Class | Primary action for `hosted_file` | Then |
|---|---|---|
| `kobo`, `pocketbook`, `tolino`, `eink_other` | Direct **download** button | the built-in browser saves EPUB to the library |
| `kindle` | If the file is in a format the Kindle browser saves (AZW3/MOBI/TXT): download. For EPUB: instructions to use **Send to Kindle** — from a phone via the Kindle app share sheet, or the e-mail form (explain the approved-sender step) | — |
| `mobile` | **Read on this phone or send to an app** (download; the OS share sheet reaches the Kindle, Kobo or Books apps) | "On your e-reader": open `{host}` in its browser and type the code printed on the card (the default e-reader path, no phone needed) → pair code (shorter to type) → Send to Kindle by e-mail |
| `desktop` | Download | e-reader instructions, pairing, e-mail, USB copy hint |

For `reseller_redirect` gifts the primary action on every class is "add to my {store} library"
(nothing is typed on the e-reader: the phone signs in, the e-reader syncs).

### Claim semantics

- States: `created` → `redeemed` (first claim) → `exhausted` (hosted file: session limit
  reached) or `fulfilled` (reseller confirmed); plus `cancelled`, `expired`, `reissued`.
- A hosted-file gift is **re-downloadable by whoever holds the card** until it expires
  (default: never) or a limit of download sessions is reached (default 10). The gift page
  shows "claimed n times, last on …" so misuse is visible to the holder.
- **Reissue**: the giver (manage token) or the partner can void the code and get a new one
  for the same gift (lost card): the old code answers `410`.
- No account, e-mail address or cookie is needed for hosted files: holding the card is the
  credential.
- `reseller_redirect` gifts are single-recipient: once the partner confirms fulfilment
  (§4.4) the gift page only says the book is already in an account and links to the store.
- Codes are stored hashed (HMAC-SHA-256 with a server secret); the clear code exists only
  in the creation response and on the card.

## 3. Creator API (standalone mode)

`POST /api/v1/uploads` — body: the file bytes. Headers: `Content-Type`,
`X-File-Name` (percent-encoded UTF-8), `X-Rights-Basis: own_work | public_domain | open_licence | licensed_distributor`
(mandatory declaration). Limits: 50 MB; only EPUB/PDF (sniffed, not trusted from the header);
EPUBs carrying DRM (`META-INF/encryption.xml` with a non-font-obfuscation algorithm, or
`META-INF/rights.xml`) are refused with `drm_protected`.
→ `201 {"upload_id": "...", "sha256": "...", "size_bytes": 123, "media_type": "application/epub+zip"}`

`POST /api/v1/gifts`

```json
{
  "book": { "title": "…", "isbn13": "9782070360024", "contributors": [{"name": "…"}] },
  "giver_name": "Camille", "recipient_name": "Sacha", "message": "Joyeux anniversaire !",
  "locale": "fr",
  "delivery": { "type": "hosted_file", "upload_id": "…" }
}
```

→ `201 {"id": "g_…", "code": "BCDFGHJKLMNP", "code_display": "BCDF-GHJK-LMNP", "claim_url": "https://…/k/BCDFGHJKLMNP", "manage_token": "…"}`

`GET /api/v1/gifts/{id}` (header `Authorization: Bearer {manage_token}`) →
`{"id", "status": "created|redeemed|fulfilled|cancelled|expired", "created_at", "redeemed_at", "downloads"}`.
`DELETE /api/v1/gifts/{id}` cancels the gift (the code stops working, hosted file deleted).
`POST /api/v1/gifts/{id}/reissue` → `200 {"code", "code_display", "claim_url"}` (the previous
code is voided). The creation body accepts an optional `"reveal_at": <unix>`.

`POST /api/v1/gifts/{id}/print-orders` (Bearer) — body
`{"product": "card-poche", "pdf_sha256": "…", "address": {…}, "quantity": 1}`
(`quantity` optional, 1–1000, default 1) plus the PDF uploaded beforehand via
`PUT /api/v1/gifts/{id}/card.pdf`. The server forwards the PDF to the configured print
provider (`EBC_PRINTER_URL`, see §5) and returns
`201 {"order_id", "status", "price_cents", "currency"}`.

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

Errors of this route:

| Status / code | When |
|---|---|
| `400 invalid_address` | the address is not an object, misses a required member, has a member of the wrong type, too long, with a control character, unknown (e.g. `postcode`), or `country` is not two letters; `error.field` names it (`"address"`, `"address.postal_code"`, …) |
| `422 invalid_request` | `product` is not a product id, or `quantity` is out of range |
| `409 card_pdf_missing`, `409 pdf_mismatch` | no card uploaded, or `pdf_sha256` is not the uploaded card's |
| `503 printing_not_configured` | no `EBC_PRINTER_URL` |
| `422 preflight_failed` | the printer's preflight refused the PDF (§5); `error.findings` are the printer's |
| `422 printer_rejected` | the printer refused the order with any other `4xx` (`unsupported_country`, `unknown_product`, `invalid_quantity`, `invalid_address`, `duplicate_reference`, …) |
| `502 printer_unavailable` | the printer did not answer, answered `5xx`, or answered `2xx` without an `order_id` |

Both `422` refusals carry the printer's answer: `error.printer_status` (its HTTP status),
`error.printer_code` (its `error.code`, or `http_<status>` when it gave none usable),
`error.printer_message` (its message, at most 300 characters, when given) and
`error.findings` (its findings, or `[]`). Nothing is recorded on the gift unless the
printer accepted the order.

## 4. Partner API (reseller-integrated mode)

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
headers outside the signature (`Idempotency-Key`, `X-Rights-Basis`, `X-File-Name`) change
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

`delivery` may also be `{"type": "reseller_licence", "format": "lcpl", "url": "https://…"}` or
`{"type": "hosted_file", "upload_id": "…"}` (upload via `POST /api/partner/v1/uploads`, same
rules as §3 with `X-Rights-Basis: licensed_distributor`).

→ `201 {"id", "code", "code_display", "claim_url", "status": "created"}`

`PUT /api/partner/v1/gifts/{id}/cover` — body: the cover image the partner is entitled to
print (JPEG/PNG/WebP, ≤ 20 MB). The server never fetches third-party URLs itself.

`GET /api/partner/v1/gifts/{id}/card.pdf?format=poche|a6|digest|a5&profile=pro|home-a4|home-letter`
→ the print-ready card rendered server-side with the same library the browser uses.

`GET /api/partner/v1/gifts/{id}` → status as in §3 plus `order_ref`.
`POST /api/partner/v1/gifts/{id}/cancel` → voids the code (refund path).
`POST /api/partner/v1/gifts/{id}/reissue` → new code, as in §3.

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

## 5. Print provider interface (what the fictional printer implements)

| Route | Purpose |
|---|---|
| `GET /v1/products` | Catalogue: `[{ "id": "card-poche", "trim_mm": [230, 178], "bleed_mm": 3, "spine_mm": 10, "paper": "350 g/m² …", "price_cents": {"1": 390, "10": 1900} }]` |
| `POST /v1/orders` | `multipart/form-data`: part `order` (JSON: `product`, `quantity`, `address`, `reference`, optional `callback_url`) and part `file` (the PDF). `address` is the object of §3 — `{name, line1, line2?, postal_code, city, country}`, `country` an ISO 3166-1 alpha-2 code. The printer **preflights** the PDF against the product: `201 {"order_id", "status": "accepted", "price_cents", "currency"}` or `422 {"error": {"code": "preflight_failed", "findings": [...]}}`. Other refusals are `{"error": {"code", "message"}}` with a `4xx`: `400 invalid_order` / `invalid_multipart` / `missing_part` (unreadable request, e.g. an address member missing), `422 unknown_product`, `invalid_quantity`, `invalid_reference`, `invalid_address`, `unsupported_country`, `invalid_callback_url`, `409 duplicate_reference`, `413` too large. The eBook Cadeau server passes them on as `422 printer_rejected` (§3) |
| `GET /v1/orders/{id}` | `{"status": "accepted|printing|shipped|cancelled", "tracking": …}` |
| Webhook | Optional `callback_url` in the order: status changes are POSTed there |

## 6. Configuration (server)

| Variable | Default | Meaning |
|---|---|---|
| `EBC_BIND` | `127.0.0.1:8080` | listen address |
| `EBC_PUBLIC_URL` | `http://127.0.0.1:8080` | base of claim URLs (keep it short: it is typed by hand) |
| `EBC_DATA_DIR` | `./var` | database and uploaded files |
| `EBC_CODE_SECRET` | — (required outside dev) | HMAC key for code hashing |
| `EBC_PARTNERS` | — | path to a TOML file of partners (`id`, `name`, `secret`, `webhook_url`, `redirect_hosts`) |
| `EBC_SMTP_URL` | — | `smtp://host:port` used for send-to-device; unset disables the feature |
| `EBC_SEND_FROM` | `livraison@localhost` | sender address recipients must approve (Send to Kindle) |
| `EBC_SEND_DOMAINS` | `kindle.com,pbsync.com` | allowed device e-mail domains |
| `EBC_PRINTER_URL` | — | print provider base URL |
| `EBC_RATE_UNKNOWN_PER_10MIN` | `10` | unknown-code attempts per client before `429` |
| `EBC_RATE_UNKNOWN_GLOBAL_PER_10MIN` | `500` | unknown gift or pair codes from all clients together before every further wrong guess gets `429` (codes are bearer secrets not tied to an account, so a guess is a guess against all live codes). While it is spent, **pair codes are not looked up at all** (`429` for everybody); gift codes stay budgeted per client network |
| `EBC_RATE_CREATE_PER_10MIN` / `EBC_RATE_CREATE_GLOBAL_PER_10MIN` | `120` / `2000` | uploads + gift creations on the open creator API, per client network / from all clients together (0 = off) |
| `EBC_RATE_PARTNER_FAIL_PER_10MIN` | `30` | failed partner authentications per client network before `/api/partner/*` answers `429` (0 = off) |
| `EBC_ANON_STORAGE_QUOTA_MB` | `10240` | bytes all creator-API uploads may occupy together; beyond, `507 storage_quota_exceeded` (0 = no quota) |
| `EBC_ANON_UPLOAD_TTL_MIN` | `60` | a creator-API upload not attached to a gift is deleted after this (partner uploads: 24 h) |
| `EBC_RETENTION_DAYS` | `30` | a cancelled gift loses names and message at once; a cancelled or expired gift is anonymised (names, message, files, upload record removed; id, status, dates and the code hash kept so the code still answers `410`) this many days after cancellation or expiry |
| `EBC_MAX_INFLIGHT_BODY_MB` | `256` | request-body bytes held by all requests together (never less than one largest body, 64 MiB); beyond, `503 server_busy` |
| `EBC_HEADER_TIMEOUT_SECS` | `30` | time to send the request line and headers (also closes idle keep-alive connections) |
| `EBC_REQUEST_TIMEOUT_SECS` | `300` | time until the response starts, body upload included (`408`) |
| `EBC_BODY_IDLE_TIMEOUT_SECS` | `30` | longest pause inside a request body (`408`) |
| `EBC_MAX_CONNECTIONS` | `1024` | connections served at once; more wait in the listen backlog |
| `EBC_WEB_DIR` | — | directory of the static web app, served at `/app/` (`.wasm` as `application/wasm`) |
| `EBC_TRUST_PROXY` | `0` | honour `X-Forwarded-For` for client identification (only behind a trusted reverse proxy) |
