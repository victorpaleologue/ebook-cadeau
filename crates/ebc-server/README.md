# ebc-server

The eBook Cadeau web service: redemption pages that work in e-reader browsers, file
delivery, the creator API and the partner (reseller) API. The HTTP contract is
[`docs/api.md`](../../docs/api.md) v0.2; this crate implements its §2, §3, §4 and §6.

It is a library plus a thin binary. Tests and demos mount it in-process:

```rust
let state = ebc_server::AppState::for_tests();   // in memory, public dev secret
let router: axum::Router = ebc_server::app(state);
// Or serve it as the binary does, with time-outs and a connection cap:
// ebc_server::serve(listener, state, shutdown_future).await?;
```

## Run it

```sh
# Development: loopback URL, a code secret is generated under ./var on first start.
EBC_PARTNERS=crates/ebc-server/partners.dev.toml cargo run -p ebc-server

# Then: http://127.0.0.1:8080/  (code form)   http://127.0.0.1:8080/aide   /healthz
```

Create a standalone gift by hand:

```sh
curl -s -X POST http://127.0.0.1:8080/api/v1/uploads \
     -H 'X-Rights-Basis: public_domain' -H 'X-File-Name: le-horla.epub' \
     --data-binary @le-horla.epub            # → {"upload_id":"u_…", …}
curl -s -X POST http://127.0.0.1:8080/api/v1/gifts -H 'Content-Type: application/json' -d '{
  "book": {"title": "Le Horla", "contributors": [{"name": "Guy de Maupassant"}]},
  "giver_name": "Camille", "message": "Bonne lecture !", "locale": "fr",
  "delivery": {"type": "hosted_file", "upload_id": "u_…"}}'
# → {"id":"g_…","code":"…","code_display":"XXXX-XXXX-XXXX","claim_url":"…/k/…","manage_token":"…"}
```

## Configuration

| Variable | Default | Meaning |
|---|---|---|
| `EBC_BIND` | `127.0.0.1:8080` | Listen address |
| `EBC_PUBLIC_URL` | `http://127.0.0.1:8080` | Base of claim URLs: `http(s)://host[:port]`, no path. It is typed by hand and upper-cased in the QR code, so keep the host short, letters and dots only (a warning is logged otherwise) |
| `EBC_DATA_DIR` | `./var` | `db.json` (snapshot of all records) and `files/<sha256>` (uploads, covers, card PDFs) |
| `EBC_CODE_SECRET` | — | HMAC key for code hashing, at least 16 characters. **Required** unless the public URL is a loopback address (then one is generated in `EBC_DATA_DIR/code-secret`). Changing it voids every issued code |
| `EBC_PARTNERS` | — | Path to the partners file (below) |
| `EBC_SMTP_URL` | — | `smtp://host:port` (plain, for a local relay), `smtp://…?tls=required`, or `smtps://user:pass@host`. Unset disables send-to-device |
| `EBC_SEND_FROM` | `livraison@localhost` | Sender address recipients must approve (Send to Kindle) |
| `EBC_SEND_DOMAINS` | `kindle.com,pbsync.com` | Device e-mail domains accepted by the send form |
| `EBC_PRINTER_URL` | — | Print provider base URL (`docs/api.md` §5); unset: print orders answer `503` |
| `EBC_RATE_UNKNOWN_PER_10MIN` | `10` | Unknown gift or pair codes per client before `429` (0 = off) |
| `EBC_RATE_UNKNOWN_GLOBAL_PER_10MIN` | `500` | Unknown codes from all clients together before every further wrong guess gets `429` (0 = off) |
| `EBC_RATE_CREATE_PER_10MIN` | `120` | *Addition to the contract.* Uploads + gift creations per client on the open creator API (0 = off) |
| `EBC_RATE_CREATE_GLOBAL_PER_10MIN` | `2000` | The same from all clients together; beyond, every creation gets `429` (0 = off) |
| `EBC_RATE_PARTNER_FAIL_PER_10MIN` | `30` | Failed partner authentications (bad or stale signature, unknown partner) per client network; beyond, `/api/partner/*` answers `429` before reading anything (0 = off) |
| `EBC_ANON_STORAGE_QUOTA_MB` | `10240` | Bytes all anonymous (creator API) uploads may occupy together, attached or not; beyond, `507 storage_quota_exceeded` (0 = no quota) |
| `EBC_ANON_UPLOAD_TTL_MIN` | `60` | An anonymous upload not attached to a gift is deleted after this many minutes (partner uploads: 24 h) |
| `EBC_RETENTION_DAYS` | `30` | Retention of cancelled and expired gifts, see below |
| `EBC_MAX_INFLIGHT_BODY_MB` | `256` | Request-body bytes held in memory by all requests together (at least one largest body, 64 MiB); beyond, `503 server_busy` + `Retry-After` |
| `EBC_HEADER_TIMEOUT_SECS` | `30` | Time a client has to send the request line and headers; also closes idle keep-alive connections |
| `EBC_REQUEST_TIMEOUT_SECS` | `300` | Time a request may take until its response starts, body upload included (`408`) |
| `EBC_BODY_IDLE_TIMEOUT_SECS` | `30` | Longest pause between two pieces of a request body (`408`) |
| `EBC_MAX_CONNECTIONS` | `1024` | Connections served at once; further ones wait in the listen backlog |
| `EBC_WEB_DIR` | — | Directory of the static web app, served at `/app/` |
| `EBC_TRUST_PROXY` | `0` | `1`: identify clients by the **last** `X-Forwarded-For` entry (the one your reverse proxy appends). Only behind a proxy you control |

Outbound HTTP (webhooks, licences, print provider) honours the usual `HTTPS_PROXY` /
`NO_PROXY` variables and never follows redirects.

### Retention of personal data

A gift holds the giver's and recipient's names and the message. They are kept while the
gift can be claimed (by default a gift never expires: the card stays usable). Then:

- **cancelled** (giver `DELETE`, partner `cancel`): names and message are blanked at once,
  the hosted file, cover and card PDF deleted;
- **cancelled or expired, `EBC_RETENTION_DAYS` (30) days later**: the record is anonymised —
  names, message, the times of sendings, files and the upload record are removed; the id,
  status, dates, order reference and the code hash stay, so that the printed code keeps
  answering `410` (not `404`, which would count as a guess) and the partner can still read
  the status.

The purge runs with the next write (an upload, a claim). Device e-mail addresses are never
stored; partners' `account_ref` only as a hash.

### Partners file (development example)

`partners.dev.toml` — never commit real secrets:

```toml
[[partners]]
id = "septante"                       # value of X-EBC-Partner
name = "Septante (démo)"              # shown to recipients: "Ajouter à ma bibliothèque …"
secret = "dev-secret-septante-0123456789"   # ≥ 16 characters; signs requests, redirects, webhooks
webhook_url = "http://127.0.0.1:8101/webhooks/ebc"
redirect_hosts = ["127.0.0.1:8101"]   # hosts delivery URLs may point to (host or host:port)

[[partners]]
id = "grande-enseigne"
name = "Grande Enseigne (démo)"
secret = "dev-secret-grande-enseigne-0123456789"
redirect_hosts = ["127.0.0.1:8102"]
```

A partner request is signed as `docs/api.md` §4.1 says:

```sh
ts=$(date +%s); path=/api/partner/v1/gifts; body='{"order_ref":"ORD-1","book":{"title":"Candide"},"delivery":{"type":"reseller_redirect","url":"http://127.0.0.1:8101/cadeau/activer"}}'
hash=$(printf %s "$body" | sha256sum | cut -d' ' -f1)
sig=$(printf %s "$ts.POST.$path.$hash" | openssl dgst -sha256 -hmac "dev-secret-septante-0123456789" | sed 's/^.* //')
curl -s -X POST "http://127.0.0.1:8080$path" -H "X-EBC-Partner: septante" -H "X-EBC-Timestamp: $ts" \
     -H "X-EBC-Signature: v1=$sig" -H "Idempotency-Key: ORD-1" -d "$body"
```

## Routes

Recipient pages (no JavaScript, no cookie): `GET /`, `GET /k?code=`, `GET /aide`,
`GET /k/{code}` (`/K/` too, `?device=kobo|kindle|pocketbook|tolino|phone|computer`),
`POST /k/{code}/download|pair|send|redeem`, `GET|POST /e`, `GET|HEAD /d/{session}/{file-name}`,
`GET /o/{session}/catalog.xml|catalog.json`, `GET /healthz`.
Creator API: `POST /api/v1/uploads`, `POST /api/v1/gifts`, `GET|DELETE /api/v1/gifts/{id}`,
`POST …/reissue`, `PUT …/card.pdf`, `POST …/print-orders`.
A print order's `address` is `{name, line1, line2?, postal_code, city, country}` (`country`
ISO 3166-1 alpha-2, default `FR`); a malformed one is `400 invalid_address` with
`error.field`. The printer's `4xx` refusals come back as `422 preflight_failed` /
`422 printer_rejected` carrying `printer_code`, `printer_message` and `findings`; only a
network error, a `5xx` or an unusable answer is `502 printer_unavailable` (`docs/api.md` §3).
Partner API: `POST /api/partner/v1/uploads`, `POST /api/partner/v1/gifts`, `GET …/gifts/{id}`,
`PUT …/cover[?cover_rights=…]`, `GET …/card.pdf`, `POST …/cancel|reissue|fulfilled`; signed webhooks.

Beyond the contract: `GET /{CODE}` redirects to `/k/{CODE}`; `/static/eink.css` (the one
style sheet); `/o/{session}/cover` (the partner's cover, for OPDS clients); the form field
`opds=1` on `POST /k/{code}/download` redirects to the OPDS feed instead of the file.

## The card (`card.pdf`)

`GET /api/partner/v1/gifts/{id}/card.pdf?format=poche|a6|digest|a5&profile=pro|home-a4|home-letter`
returns the print-ready card composed by `ebc-card` (cargo feature `card`, default on;
defaults: `poche`, `pro`). The same gift always gives the same bytes.

| `profile` | Output |
|---|---|
| `pro` | trim + 3 mm bleed, no marks, PDF/X-4, every colour built under 240 % total ink (the composer's `upload_eu` profile: what web-to-print and print-on-demand portals accept) |
| `home-a4`, `home-letter` | the card centred on the sheet with cut and fold guides, plain PDF, to print duplex at 100 % |

**Covers and rights.** The cover a partner uploads with `PUT …/cover` is printed only
under a rights basis the partner *declares*: `"cover_rights"` in the gift creation body,
or `PUT …/cover?cover_rights=…` (the query string is part of the request signature).
Values: `none` (default), `print_licence`, `promo_all_media`, `rights_holder_declared`,
`public_domain`. Without a declaration the cover is stored but never enters a print
file — the card is typographic (title and author as text). `promo_all_media` only allows
the reduced-size layout of home-printed cards.

| Answer | When |
|---|---|
| `200 application/pdf` | the card; when the composer has something to say about it, `X-EBC-Card-Warnings: code, code, …` lists its warning codes (e.g. `dark_across_creases`: order matt or soft-touch lamination, grain parallel to the spine — see `docs/api.md` §4.2) |
| `400 invalid_parameter` | unknown `format` / `profile` |
| `422 invalid_cover_rights` | an unknown `cover_rights` value (creation or cover upload) |
| `422 cover_rights_refused` | the declared basis does not allow this print route (e.g. `promo_all_media` with `profile=pro`) |
| `422 card_unprocessable` | the card cannot be laid out as asked: it does not fit the sheet (`a5` on A4), the declared cover is not a decodable JPEG/PNG/WebP image, … |
| `409 gift_cancelled`, `409 code_unavailable` | the gift is cancelled, or its code cannot be reproduced |
| `501 not_implemented` | the server was built without the `card` feature |

## Security notes

- Codes, pair codes, manage tokens and download tokens are stored only as HMAC/SHA-256.
  Partner gifts keep a random nonce from which the code is re-derived with
  `EBC_CODE_SECRET` (needed to render `card.pdf`); without the secret it reveals nothing.
- Unknown codes are budgeted per client network (IPv4 /24, IPv6 /56) and globally; typos
  (wrong check letter, wrong length, digits) are answered `400` without a look-up and
  never counted. Rate-limit state is in memory: a restart forgets it.
- **When the global budget is spent** (500 unknown codes in 10 minutes from everywhere)
  every further wrong guess gets `429`, and the two kinds of code are treated differently:
  - *pair codes* are not looked up at all, for anybody: they are 6 letters (20⁶ ≈ 2²⁶),
    an attacker who controls many networks (one IPv6 /48 is 256 /56 "clients") could
    otherwise search that space with one free guess per fresh network. Refusing costs
    little: a pair code lives 15 minutes and the e-reader can take the gift code instead;
  - *gift codes* stay budgeted per network: a network that has not failed yet is still
    looked up. They have 47.5 bits, so even a fresh guess from each of 2²⁴ networks
    finds a live code with negligible probability, whereas refusing them all would let
    anybody lock every recipient out of their gift with 500 requests. This is the
    deliberate trade-off between guessing resistance and denial of service.
- Partner requests: the signature format is checked, and the partner and timestamp looked
  up, before the body is read; the body is hashed as it streams in, under a server-wide
  in-flight budget (`EBC_MAX_INFLIGHT_BODY_MB`), and failed authentications are budgeted
  per network. Every verified `POST`/`PUT` signature is a single-use nonce for its 300 s
  window (keyed on the decoded MAC): replays, with whatever unsigned headers, get the
  first answer and never act twice (`docs/api.md` §4.1).
- Send-to-device: the sending is counted (and the daily limit of three checked) in one
  transaction before the mail is sent, and handed back if the mail cannot be delivered,
  so concurrent requests cannot exceed the limit.
- Request logs never contain a code or a token (paths are redacted before logging).
- `reseller_redirect` / `reseller_licence` targets come from the stored gift and are
  checked against the partner's `redirect_hosts` at creation **and** at redemption.
- The send form only accepts plain addresses on `EBC_SEND_DOMAINS`, three sendings per
  gift and day, and the address is neither stored nor logged.
- Uploads are sniffed (never trusted from headers), EPUBs with DRM are refused, at most
  1 MiB of archive metadata is read and nothing is decompressed.
- The binary (`ebc_server::serve`) applies header, request and body time-outs and a
  connection cap (see the configuration). It speaks plain HTTP/1.1: terminate TLS in a
  reverse proxy, which should also bound slow *readers* of responses (hyper has no write
  time-out).
- Errors of outbound requests are logged without their URL (licence URLs carry a valid
  store signature).

## Known limits

- `card.pdf` needs the cargo feature `card` (on by default); a build with
  `--no-default-features` answers `501 not_implemented` unless another `CardRenderer` is
  passed to `AppState::builder(..).card_renderer(..)`.
- Cards are composed with the composer's bundled fonts and FOGRA39 output intent; a
  partner cannot choose the printing condition, the spine style or the ground colour yet.
- Storage is one JSON snapshot rewritten on every change, with a single writer process.
- Uploads (≤ 50 MB) are buffered in memory while they are checked; all request bodies
  together are bounded by `EBC_MAX_INFLIGHT_BODY_MB`. A partner who knows a partner id can
  still make the server hash (not keep) a body up to the size limit with a well-formed
  but wrong signature, a few times per network and 10 minutes: the MAC covers the body,
  so it cannot be checked earlier.
- Page strings specific to the server (`src/text.rs`) are not reviewed by native speakers.
- The gift page shows no cover thumbnail (the server has no image scaler).

## Tests

```sh
cargo test -p ebc-server            # offline; uses loopback sockets only
cargo clippy -p ebc-server --all-targets -- -D warnings
```
