# eBook Cadeau — HTTP contract (server, partner API, redemption pages)

_Status: v0.1 — the contract the server, the demo stores, the simulated e-readers and the
end-to-end tests are all built against. The redemption UX is justified in
`docs/research/05-ereader-distribution-ux.md`._

Conventions: JSON bodies are UTF-8; timestamps are Unix seconds; errors are
`{"error": {"code": "snake_case", "message": "…"}}` with a matching HTTP status. HTML pages
work **without JavaScript** and carry stable element `id`s (listed below) used by tests.

## 1. Actors and vocabulary

| Term | Meaning |
|---|---|
| **Gift** | One gifted eBook: metadata, a redemption **code**, and a **delivery** method |
| **Code** | `GiftCode`: 10 Crockford-base32 symbols (9 data + 1 check), shown as `XXXXX-XXXXX` |
| **Claim URL** | `{PUBLIC_URL}/k/{CODE}` — encoded in the QR code and printed in clear on the card |
| **Giver / recipient** | The buyer and the person who receives the card |
| **Partner** | An eBook reseller integrating the "eBook cadeau" option |
| **Delivery** | `hosted_file` (we serve the file), `reseller_redirect` (the reseller adds the book to the recipient's account), `reseller_licence` (we hand over a licence file such as `.lcpl`/`.acsm` issued by the reseller) |
| **Download session** | A random 128-bit URL token minted after a successful claim, short-lived |
| **Pair number** | A 6-digit number that links a phone that scanned the QR to an e-reader's browser |

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
| `GET /k/{code}` | The gift page: title, author, giver, message, and the primary action for this device | `gift-title`, `gift-author`, `gift-from`, `gift-message`, `primary-action`, `other-ways` |
| `POST /k/{code}/download` | `hosted_file` only. Mints a download session and 303-redirects to `/d/{session}/{file-name}` | form id `download-form`, button id `download` |
| `GET /d/{session}/{file-name}` | The file. `Content-Type` per format (`application/epub+zip`), `Content-Disposition: attachment` with ASCII fallback + `filename*`, `Content-Length`, `Accept-Ranges`. `HEAD` supported | — |
| `POST /k/{code}/pair` | From a phone: creates a pair number and shows it with the e-reader instructions | `pair-number`, `pair-url` |
| `GET /e` | On the e-reader: form asking for the pair number | `pair-form`, `pair-input` (name `n`), `pair-submit` |
| `POST /e` | Validates the pair number and 303-redirects to `/d/{session}/{file-name}` | — |
| `POST /k/{code}/send` | Sends the file to a device address (`device_email`), e.g. Send-to-Kindle; allowed domains are configured | form id `send-form`, input id `device-email`, result id `send-result` |
| `POST /k/{code}/redeem` | `reseller_redirect`: 303 to the partner URL with signed parameters (§4.3). `reseller_licence`: 303 to a download session serving the licence file | button id `redeem` |
| `GET /o/{session}/catalog.xml` | OPDS 1.2 acquisition feed containing the one publication | — |
| `GET /o/{session}/catalog.json` | OPDS 2.0 feed | — |
| `GET /healthz` | `200 ok` | — |

Status codes on `/k/{code}`: `200`; `400` malformed code (bad check symbol — the page says it
looks like a typo and **does not** count as an attempt); `404` well-formed but unknown (counts
towards the rate limit); `410` cancelled or expired; `429` too many unknown codes from this
client (`Retry-After`).

### Device classes

The server classifies the `User-Agent` into `kobo`, `kindle`, `pocketbook`, `tolino`,
`eink_other`, `mobile`, `desktop` and picks the primary action:

| Class | Primary action for `hosted_file` | Notes |
|---|---|---|
| `kobo`, `pocketbook`, `tolino`, `eink_other` | Direct **download** button | the built-in browser saves EPUB to the library |
| `kindle` | **Send to Kindle** form (the Kindle browser cannot save EPUB); download offered only for formats Kindle accepts | explain the approved-sender step |
| `mobile` | "Send to my e-reader" (**pair number**) first, then "Read on this phone" (download), then "Send to Kindle" | the phone is usually just the scanner |
| `desktop` | Download, then pairing and email | |

### Claim semantics

- A gift is **re-downloadable by whoever holds the card** until it expires (default: none)
  or a limit of download sessions is reached (default 10); the first successful claim sets
  `redeemed_at`.
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

→ `201 {"id": "g_…", "code": "ABCDE12345", "code_display": "ABCDE-12345", "claim_url": "https://…/k/ABCDE12345", "manage_token": "…"}`

`GET /api/v1/gifts/{id}` (header `Authorization: Bearer {manage_token}`) →
`{"id", "status": "created|redeemed|fulfilled|cancelled|expired", "created_at", "redeemed_at", "downloads"}`.
`DELETE /api/v1/gifts/{id}` cancels the gift (the code stops working, hosted file deleted).

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
