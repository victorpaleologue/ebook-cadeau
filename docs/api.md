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
| `GET /k?code=…` | Normalises the typed code and 303-redirects to `/k/{CODE}` | — |
| `GET /aide` | Help page: the three ways to get the book, per device | `help` |
| `GET /k/{code}` (`/K/` too; optional `?device=`) | The gift page: title, author, giver, message, the primary action for this device, the device chooser, and how many times the gift was already claimed | `gift-title`, `gift-author`, `gift-from`, `gift-message`, `primary-action`, `other-ways`, `device-chooser` (links `device-kobo`, `device-kindle`, `device-pocketbook`, `device-tolino`, `device-phone`, `device-computer`), `claim-count` |
| `POST /k/{code}/download` | `hosted_file` only. Mints a download session and 303-redirects to `/d/{session}/{file-name}` | form id `download-form`, button id `download` |
| `GET /d/{session}/{file-name}` | The file. `Content-Type` per format (`application/epub+zip`), `Content-Disposition: attachment` with ASCII fallback + `filename*`, `Content-Length`, `Accept-Ranges`. `HEAD` supported | — |
| `POST /k/{code}/pair` | From a phone: creates a pair code and shows it with the e-reader instructions and its expiry time | `pair-code`, `pair-url`, `pair-expires` |
| `GET /e` | On the e-reader: form asking for the pair code | `pair-form`, `pair-input` (name `p`), `pair-submit` |
| `POST /e` | Validates the pair code and 303-redirects to `/d/{session}/{file-name}`. Wrong codes count against a per-client **and a global** failure budget (`429` + `Retry-After`) | — |
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

`POST /api/v1/gifts/{id}/print-orders` (Bearer) — body `{"product": "card-poche", "pdf_sha256": "…", "address": {…}}`
plus the PDF uploaded beforehand via `PUT /api/v1/gifts/{id}/card.pdf`. The server forwards
the PDF to the configured print provider (`EBC_PRINTER_URL`, see §5) and returns
`201 {"order_id", "status", "price_cents", "currency"}`.

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
is `401 bad_signature`. Replaying an `Idempotency-Key` with the same body returns the
original response; with a different body, `409 idempotency_conflict`.

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
| `POST /v1/orders` | `multipart/form-data`: part `order` (JSON: `product`, `quantity`, `address`, `reference`) and part `file` (the PDF). The printer **preflights** the PDF against the product: `201 {"order_id", "status": "accepted", "price_cents"}` or `422 {"error": {"code": "preflight_failed", "findings": [...]}}` |
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
| `EBC_RATE_UNKNOWN_GLOBAL_PER_10MIN` | `500` | unknown gift or pair codes from all clients together before every further wrong guess gets `429` (codes are bearer secrets not tied to an account, so a guess is a guess against all live codes) |
| `EBC_WEB_DIR` | — | directory of the static web app, served at `/app/` (`.wasm` as `application/wasm`) |
| `EBC_TRUST_PROXY` | `0` | honour `X-Forwarded-For` for client identification (only behind a trusted reverse proxy) |
