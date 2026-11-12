# ebc-server

The eBook Cadeau web service: redemption pages that work in e-reader browsers, file
delivery, the creator API and the partner (reseller) API. The HTTP contract is
[`docs/api.md`](../../docs/api.md) v0.2; this crate implements its §2, §3, §4 and §6.

It is a library plus a thin binary. Tests and demos mount it in-process:

```rust
let state = ebc_server::AppState::for_tests();   // in memory, public dev secret
let router: axum::Router = ebc_server::app(state);
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
| `EBC_WEB_DIR` | — | Directory of the static web app, served at `/app/` |
| `EBC_TRUST_PROXY` | `0` | `1`: identify clients by the **last** `X-Forwarded-For` entry (the one your reverse proxy appends). Only behind a proxy you control |

Outbound HTTP (webhooks, licences, print provider) honours the usual `HTTPS_PROXY` /
`NO_PROXY` variables and never follows redirects.

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
Partner API: `POST /api/partner/v1/uploads`, `POST /api/partner/v1/gifts`, `GET …/gifts/{id}`,
`PUT …/cover`, `GET …/card.pdf`, `POST …/cancel|reissue|fulfilled`; signed webhooks.

Beyond the contract: `GET /{CODE}` redirects to `/k/{CODE}`; `/static/eink.css` (the one
style sheet); `/o/{session}/cover` (the partner's cover, for OPDS clients); the form field
`opds=1` on `POST /k/{code}/download` redirects to the OPDS feed instead of the file.

## Security notes

- Codes, pair codes, manage tokens and download tokens are stored only as HMAC/SHA-256.
  Partner gifts keep a random nonce from which the code is re-derived with
  `EBC_CODE_SECRET` (needed to render `card.pdf`); without the secret it reveals nothing.
- Unknown codes are budgeted per client network (IPv4 /24, IPv6 /56) and globally; typos
  (wrong check letter, wrong length, digits) are answered `400` without a look-up and
  never counted. Rate-limit state is in memory: a restart forgets it.
- Request logs never contain a code or a token (paths are redacted before logging).
- `reseller_redirect` / `reseller_licence` targets come from the stored gift and are
  checked against the partner's `redirect_hosts` at creation **and** at redemption.
- The send form only accepts plain addresses on `EBC_SEND_DOMAINS`, three sendings per
  gift and day, and the address is neither stored nor logged.
- Uploads are sniffed (never trusted from headers), EPUBs with DRM are refused, at most
  1 MiB of archive metadata is read and nothing is decompressed.
- Terminate TLS and set request time-outs in a reverse proxy; the binary speaks plain
  HTTP/1.1.

## Known limits

- `card.pdf` answers `501 not_implemented` until a `CardRenderer` backed by `ebc-card` is
  passed to `AppState::builder(..).card_renderer(..)`.
- Storage is one JSON snapshot rewritten on every change, with a single writer process.
- Uploads (≤ 50 MB) are buffered in memory while they are checked.
- Page strings specific to the server (`src/text.rs`) are not reviewed by native speakers.
- The gift page shows no cover thumbnail (the server has no image scaler).

## Tests

```sh
cargo test -p ebc-server            # offline; uses loopback sockets only
cargo clippy -p ebc-server --all-targets -- -D warnings
```
