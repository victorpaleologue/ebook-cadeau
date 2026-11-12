# ebc-shelf — the shelf

The service that **hosts rights holders' eBook files** and gives them with eBook Cadeau
cards. eBook Cadeau is two services (owner decision, `docs/api.md` v0.3):

- the **core** (`crates/ebc-server`) holds gifts, codes, cards and redemption pages and
  **never stores a book file**;
- the **shelf** (this crate) stores the files of their rights holders and registers with
  the core **as a partner, exactly like a store** (`EBC_PARTNERS`, the signed partner API
  of `docs/api.md` §4). It creates `partner_file` gifts and, on each claim, answers the
  core's signed link request with a short-lived download link on its own host (§4.5).

The shelf is the part that is a **host** in law (DSA art. 6; French LCEN art. 6), so the
hosting duties live here (`docs/research/09-legal-gifting-and-commerce.md` §2.6, §3, §9).
It may be run by an author or publisher for their own books (`EBC_SHELF_MODE=own`), or by
an operator for many rights holders (`multi`).

> **Legal status of this crate**: the duties below are implemented as the research
> describes them. The research is material to brief a lawyer, **not legal advice**, and
> nothing here has been reviewed by counsel. The terms, legal notice, statements of
> reasons and the identification field list are **drafts**. See "What the operator and
> counsel must still do".

## Run it

```sh
# 1. The core knows the shelf as a partner (crates/ebc-server/partners.dev.toml):
#    [[partners]] id = "etagere", secret = "dev-secret-etagere-0123456789",
#    redirect_hosts = ["127.0.0.1:8104"]
EBC_PARTNERS=crates/ebc-server/partners.dev.toml cargo run -p ebc-server

# 2. The shelf, single-author mode, mail to a local SMTP relay or sink:
EBC_SHELF_OPERATOR_EMAIL=autrice@exemple.fr \
EBC_SHELF_PARTNER_SECRET=dev-secret-etagere-0123456789 \
EBC_SMTP_URL=smtp://127.0.0.1:2525 \
cargo run -p ebc-shelf
# → http://127.0.0.1:8104/  (sign in with the operator address; the link arrives by mail)
```

The whole demo constellation starts it too, with a demo webmail for the sign-in links
(`cargo run -p ebc-demo-integration --bin ebc-demo`, `demo/README.md`).

As a library (tests and demos mount it in-process, as the core does):

```rust
let (state, outbox) = ebc_shelf::AppState::for_tests();   // in memory, test secrets
let router: axum::Router = ebc_shelf::app(state.clone());
// Or as the binary does, with time-outs and a connection cap:
// ebc_shelf::serve(listener, state, shutdown_future).await?;
```

### Deploying

- One process, one data directory (`db.json` snapshot replaced atomically on each change,
  files under `files/<sha256>`), plain HTTP/1.1 behind a TLS reverse proxy that also bounds
  slow readers (`EBC_SHELF_TRUST_PROXY=1` then). Use a **short HTTPS host** of its own: it
  is the host of every download link and must be the shelf's `redirect_hosts` entry at the
  core.
- At the core: one `[[partners]]` entry (`id` = `EBC_SHELF_PARTNER_ID`, `secret` =
  `EBC_SHELF_PARTNER_SECRET`, `redirect_hosts = ["<shelf host>"]`). No webhook is needed.
- A real SMTP relay (`EBC_SMTP_URL`, `smtps://…` or `smtp://…?tls=required`): without
  mail nobody can sign in.
- The binary purges retention periods and re-sends unconfirmed cancellations every five
  minutes (`AppState::maintain`).
- Host it in the EU/EEA (research 09 §9.3). Back up the data directory: it holds the
  identification data and declarations the law asks you to keep.

## Configuration

| Variable | Default | Meaning |
|---|---|---|
| `EBC_SHELF_BIND` | `127.0.0.1:8104` | listen address |
| `EBC_SHELF_PUBLIC_URL` | `http://127.0.0.1:8104` | `http(s)://host[:port]`, no path: download links and the §4.5 endpoint (`/api/ebc/link`) are on this host |
| `EBC_SHELF_DATA_DIR` | `./var-shelf` | `db.json` and `files/<sha256>` |
| `EBC_SHELF_NAME` | `Étagère eBook Cadeau` | name shown on every page and in mails |
| `EBC_SHELF_MODE` | `own` | `own`: only the operator's account exists and uploads (an author self-hosting); `multi`: open sign-up for rights holders, the operator moderates |
| `EBC_SHELF_OPERATOR_EMAIL` | — (required) | the operator's account (moderation; in `own` mode the only one) |
| `EBC_SHELF_SECRET` | — | HMAC key for form tokens and pseudonyms (≥ 16 characters). Required unless the public URL is loopback (then generated in the data directory) |
| `EBC_CORE_URL` | `http://127.0.0.1:8080` | the core, for the signed partner API |
| `EBC_CORE_PUBLIC_URL` | `EBC_CORE_URL` | the core as browsers reach it (the card designer link) |
| `EBC_SHELF_PARTNER_ID` | `etagere` | `X-EBC-Partner` at the core |
| `EBC_SHELF_PARTNER_SECRET` | — (required) | the secret shared with the core (≥ 16 characters) |
| `EBC_SMTP_URL` | — | sign-in links, acknowledgements, decisions |
| `EBC_SHELF_MAIL_FROM` | `etagere@localhost` | sender of every mail |
| `EBC_SHELF_CONTACT_EMAIL` | — | the single point of contact (DSA arts 11–12), on `/contact`, the terms and the legal notice. **Set it** |
| `EBC_SHELF_LEGAL_OPERATOR_NAME`, `…_OPERATOR_ADDRESS`, `…_OPERATOR_REGISTRATION`, `…_PUBLICATION_DIRECTOR`, `…_HOST_NAME`, `…_HOST_ADDRESS`, `…_HOST_PHONE` | — | the legal notice (LCEN art. 6 III); unset values show "[à compléter]" |
| `EBC_SHELF_MAX_FILE_MB` | `50` | largest file |
| `EBC_SHELF_QUOTA_ACCOUNT_MB` | `200` | storage per account |
| `EBC_SHELF_QUOTA_TOTAL_MB` | `5120` | storage of the whole shelf |
| `EBC_SHELF_MAX_CONCURRENT_UPLOADS` | `4` | uploads checked at once (each is held in memory) |
| `EBC_SHELF_LOGIN_TTL_MIN` | `15` | lifetime of a sign-in link |
| `EBC_SHELF_SESSION_DAYS` | `7` | lifetime of a session |
| `EBC_SHELF_LINK_TTL_SECS` | `300` | lifetime of a download link handed to the core (≤ 3600) |
| `EBC_SHELF_SUSPEND_AFTER_UPHELD` | `2` | upheld notices before an account is suspended |
| `EBC_SHELF_SUSPEND_AFTER_FALSE_DECLARATIONS` | `1` | false rights declarations before suspension |
| `EBC_SHELF_APPEAL_DAYS` | `182` | how long an uploader can contest a decision (DSA art. 20(1): six months) |
| `EBC_SHELF_MAX_GIFTS_PER_REQUEST` | `50` | cards made at once |
| `EBC_SHELF_RATE_LOGIN_PER_HOUR` | `5` | sign-in requests per client network and per address |
| `EBC_SHELF_RATE_NOTICES_PER_HOUR` | `10` | notices per client network |
| `EBC_SHELF_RATE_UPLOADS_PER_HOUR` | `30` | uploads per account |
| `EBC_SHELF_RATE_FAIL_PER_10MIN` | `30` | refused link requests and unknown download links per client network before `429` |
| `EBC_SHELF_RETAIN_*_DAYS`, `EBC_SHELF_IDLE_FILE_DAYS` | see "Retention" | retention periods |
| `EBC_SHELF_HEADER_TIMEOUT_SECS`, `…_REQUEST_TIMEOUT_SECS`, `…_BODY_IDLE_TIMEOUT_SECS`, `…_MAX_CONNECTIONS` | `30`, `300`, `30`, `1024` | as the core's |
| `EBC_SHELF_TRUST_PROXY` | `0` | identify clients by the last `X-Forwarded-For` entry |

## Routes

No page needs JavaScript; every form is plain HTML. French by default; `?lang=en|de|es|it|nl|pt`
(remembered in a cookie), else `Accept-Language`. Element ids used by the tests in brackets.

| Route | What |
|---|---|
| `GET /` | what the shelf is (no catalogue, no search), the purchased-book warning (`intro`, `not-purchases`) |
| `GET\|POST /connexion` | sign-in by e-mail link (`signin-form`, `email`, `signin-submit`; `link-sent`). `own` mode: any other address gets `403` and no mail |
| `GET\|POST /connexion/{token}` | the link: a confirmation button (a `GET` never uses it up, so mail scanners cannot) (`confirm-signin`); `POST` opens the session |
| `POST /deconnexion` | sign out |
| `GET /compte` | account: identity status, books, decisions, storage, close (`account`, `identity-missing`, `identity-link`, `upload-link`, `suspended`, `book-{id}`, `decision-{notice}`) |
| `GET\|POST /compte/identite` | LCEN identification (`identity-form`) |
| `POST /compte/fermer` | close the account (books erased, gifts cancelled) |
| `GET\|POST /compte/decisions/{notice}` | statement of reasons and the internal complaint (`appeal-form`, `appeal-status`) |
| `GET /livres/nouveau`, `POST /livres` | the upload gate (multipart: `file`, `title`, `author`, `isbn`, `language`, `basis`, `death_year`, `licence`, `licensor`, `declared`, `csrf`). A refusal answers the form with `error` and the header `X-EBC-Refusal: <code>` |
| `GET /livres/{id}` | the book: file, declaration, gifts with code, claim URL, deliveries, card links, designer link (`book-title`, `book-status`, `declaration`, `gift-{id}`, `code-{id}`, `card-home-{id}`, `card-pro-{id}`, `designer-{id}`, `gift-form`, `delete-form`) |
| `POST /livres/{id}/cadeaux` | create 1–50 gifts at the core (giver, recipient, message — all optional —, locale, reveal date) |
| `POST /livres/{id}/supprimer` | delete the book: file erased, every gift cancelled at the core |
| `POST /cadeaux/{id}/annuler`, `…/nouveau-code`, `…/reessayer` | cancel, reissue (lost card), retry a creation the core did not confirm |
| `GET /cadeaux/{id}/carte.pdf?profile=home-a4\|home-letter\|pro&format=poche\|a6\|digest\|a5` | the card, from the core's signed `card.pdf` (warnings passed on in `X-EBC-Card-Warnings`) |
| `GET\|POST /signaler` | the public notice form, no account (`notice-form`; `notice-id`) |
| `GET /moderation`, `GET\|POST /moderation/signalements/{id}`, `POST /moderation/reclamations/{id}` | operator only: open notices, decision form (`decision-form`), complaints, blocklist, suspended accounts |
| `GET /conditions`, `/mentions-legales`, `/contact` | terms (DSA art. 14), legal notice (LCEN), points of contact (arts 11–12) — **drafts** |
| `GET /api/ebc/link?ebc_gift&ebc_order&ebc_ts&ebc_sig` | the `partner_file` endpoint of `docs/api.md` §4.5, called by the core |
| `GET\|HEAD /f/{token}/{name}` | the short-lived download link |
| `GET /healthz`, `GET /static/shelf.css` | — |

### Gifts and cards

A gift is created at the core with `POST /api/partner/v1/gifts` (signed, §4.1),
`Idempotency-Key` = the shelf's gift id (also the `order_ref`), and
`delivery = {"type": "partner_file", "format": "epub"|"pdf", "url": "<shelf>/api/ebc/link"}`.
It is recorded as *pending* first, so a lost answer or an outage is retried with the same
key ("Réessayer"); reissue and cancel go through the core too, and a cancellation the core
did not confirm is retried by maintenance (meanwhile the §4.5 endpoint already refuses the
gift). The card PDF is the core's `GET …/card.pdf`, fetched with a signed request and
handed to the owner. The **designer link** is
`{EBC_CORE_PUBLIC_URL}/app/#code=…&title=…&author=…&lang=…`: the values are in the URL
**fragment**, which browsers never send to a server, so the code stays out of access logs
(the designer also accepts them in the query string; `web/README.md`, "Prefilled from a
link").

### The §4.5 endpoint

`GET /api/ebc/link` verifies `ebc_sig` (hex HMAC-SHA-256 of `"{gift}.{order}.{ts}"` with the
partner secret, constant-time), refuses a timestamp more than 900 s old or 300 s ahead,
checks that the gift is the shelf's own (the core's gift id **and** the order reference
match), live, and its book hosted; then mints a 128-bit token valid
`EBC_SHELF_LINK_TTL_SECS` (kept in memory, by its hash) and answers

```json
{"download_url": "https://etagere.example/f/<token>/<file>", "file_name": "…",
 "media_type": "application/epub+zip", "size_bytes": 345678, "sha256": "…", "expires_at": …}
```

Refusals: `400 malformed`, `401 stale_timestamp`, `403 bad_signature`, `404 unknown_gift`,
`410 gift_gone` (cancelled, taken down, deleted); refused requests count against a
per-network budget (`429`). Each answer is one counted delivery. The link serves the file
**byte-identical** to the upload, with `Content-Type`, `Content-Length`,
`Content-Disposition: attachment` (ASCII name + `filename*`), `Accept-Ranges`, single
`Range`, `HEAD`, `Cache-Control: private, no-store`. Each download starting at byte 0 is
logged with a **pseudonym** of the client network (HMAC under the shelf secret), never the
address. **Neither the signed query, nor the token, nor any request header (`Referer` may
carry a gift code) is logged**: the request log holds the method and the redacted path
(`/f/…/…`, `/connexion/…`).

## Legal duties: what the code does, what remains

| Duty (source) | Implemented | Remains for the operator / counsel |
|---|---|---|
| Identify content providers, keep their data (LCEN art. 6 II; décret 2021-1362) | Verified e-mail (sign-in link) required; identification form before the first upload: person or organisation, full name or company name, the person acting for an organisation, postal address, optional telephone, date and place of birth; per upload/gift/deletion the account, content id, operation, time and IP address (`content_log`) | **The field list is from memory of décret 2021-1362 and must be confirmed with counsel** (whether birth data, phone or a payment identifier are mandatory for this host; whether IDs must be verified). Retention periods too |
| Upload gate (research 09 §3.3, items 6–10) | Through `ebc-epub` only: type sniffed from the bytes (EPUB/PDF), DRM refused (Adobe `rights.xml`, LCP, `encryption.xml` beyond font obfuscation, encrypted PDF), size limit; mandatory declaration from the closed list (own work, public domain with the author's death year — life + 70 —, open licence from a closed CC list, licensed distributor with licensor and contract), with title, author and ISBN, time-stamped with account, name, e-mail, text version and file hash; blocklist by SHA-256 and by ISBN (declared **and** found in the OPF) before acceptance; quotas per account and global; files stored by hash | Retailer-watermark / ex-libris detection (item 9) and the ISBN-registrant check for `own_work` with a review state (item 10) are **not** implemented; translator's death year and "mort pour la France" not asked; NC licences are accepted (no paid tier here) |
| Notice and action (DSA art. 16) | Public form without account; the art. 16(2) elements (explanation, exact location — URL, book or gift id, or the code printed on a card, resolved automatically —, name and e-mail except under the art. 16(2)(c) exception, good-faith statement); acknowledgement by mail; operator review page; decision mailed to the notifier with redress | The 24 h target is a process, not code: someone must read `/moderation`; no notice by e-mail is parsed (the contact address must be watched) |
| Action on an upheld notice (research 09 item 18) | Every hosted copy of the file deleted; every gift cancelled at the core (codes answer `410`); hash and ISBNs blocklisted; links in flight dropped | — |
| Statement of reasons (DSA art. 17) | Mailed to the uploader (and on `/compte/decisions/{id}`): decision, facts, ground, no automated means, redress (internal complaint, art. 21 out-of-court body, courts) | Wording is a draft; transparency database submissions (art. 24(5)) do not apply to a host that is not an online platform — confirm |
| Internal complaint (DSA art. 20) | A simple appeal within 6 months; the operator grants (strike, blocklist entries and suspension lifted; the erased file is not restored) or dismisses, with a reasoned answer mailed | **Art. 20 binds online platforms only**; the shelf has no public dissemination, so it is probably a plain host — kept as good practice. Confirm the status |
| Repeat infringers (research 09 §3.3 item 7; DSA art. 23 for platforms) | Suspension after 2 upheld notices or 1 false declaration (configurable): no upload, no new gift (other gifts keep working); a further upheld notice while suspended bans: every book removed, gifts cancelled, sessions ended | Whether suspension should also stop existing gifts is a policy choice |
| Terms describing moderation (DSA art. 14) | `/conditions`, generated from the configuration (limits, thresholds, retention) | **Draft, not reviewed by counsel** |
| Points of contact (DSA arts 11–12) | `/contact`, the terms and the legal notice show `EBC_SHELF_CONTACT_EMAIL` and the languages | Set the address; designate a legal representative if established outside the EU (art. 13) |
| Legal notice (LCEN art. 6 III) | `/mentions-legales` from `EBC_SHELF_LEGAL_*` | **Draft**; fill in operator, director of publication and host |
| No public access (research 09 §3.3 item 5) | No catalogue, no search, no public book page, `noindex` everywhere; files only through 128-bit links that live 5 minutes | — |
| Purpose and wording (item 1) | Every page and the upload form say a bought eBook cannot be uploaded and point to the store's gift option; a test refuses "share/lend/resell" wording | — |

## Retention (research 09 §9.1; configurable, **to confirm with counsel**)

| Data | Kept | Variable (default) |
|---|---|---|
| Sign-in links, sessions | until they expire | `EBC_SHELF_LOGIN_TTL_MIN` (15), `EBC_SHELF_SESSION_DAYS` (7) |
| Identity and e-mail of an account | while the account exists, then erased after the period (décret 2021-1362 as recalled: five years after the end of the contract for civil identity) | `EBC_SHELF_RETAIN_IDENTITY_DAYS` (1 826) |
| IP address of content creation (upload, gift, deletion) | erased after the period (as recalled: one year) | `EBC_SHELF_RETAIN_CONNECTION_DAYS` (365) |
| Rights declaration (with the book record) | after the file is deleted: "life of the hosting plus the limitation period" — five years (Code civil art. 2224) | `EBC_SHELF_RETAIN_DECLARATION_DAYS` (1 826) |
| Notices, decisions, complaints | six years (research 09 item 18, proposal) | `EBC_SHELF_RETAIN_NOTICE_DAYS` (2 192) |
| Delivery log: client pseudonym | security log, six months | `EBC_SHELF_RETAIN_SECURITY_LOG_DAYS` (180) |
| Delivery log: gift and time | proof of supply, five years | `EBC_SHELF_RETAIN_DELIVERY_DAYS` (1 826) |
| Files | deleted at once on deletion, takedown, account closure or ban; when every gift of a book is cancelled, after an idle period | `EBC_SHELF_IDLE_FILE_DAYS` (30) |
| Names and message of a gift | deleted at once when the gift is cancelled (the core deletes its own copy 12 months after the first claim) | — |

### Record of processing (GDPR art. 30) — template

| Item | Content (to complete) |
|---|---|
| Controller | ‹operator name, address, contact›; DPO: probably not required at this scale (research 09 §9.3) |
| Purposes | hosting files for their rights holders and delivering them to card holders; LCEN identification duty; handling notices and complaints; security |
| Legal bases | contract with the uploader (art. 6(1)(b)); legal obligation for identification data and their retention (6(1)(c), LCEN art. 6 II); legitimate interest for security logs and notices (6(1)(f)) |
| Data subjects and data | uploaders: e-mail, identity, address, optional phone and birth data, declarations, IP of content creation; notifiers: name, e-mail (except art. 16(2)(c)); card holders: a pseudonym of their network at download, nothing else; any personal data inside uploaded files |
| Recipients | the core (gift id, order reference, title, author, ISBN, language, giver/recipient names and message entered for the card); the SMTP provider; authorities on legal request |
| Processors | hosting provider ‹name, country›, SMTP provider ‹name›; art. 28 contracts |
| Transfers outside the EEA | none intended (EU hosting) |
| Retention | the table above |
| Security | sign-in links and sessions stored as SHA-256, single-use links; `HttpOnly`, `SameSite=Strict`, `Secure`/`__Host-` cookie and form tokens; CSP without script; rate limits; files by hash, served only through 128-bit five-minute links; no codes, tokens, signed URLs or `Referer` in logs; time-outs and a connection cap |

## Security notes

- Sign-in links are 256-bit, single-use, 15 minutes, stored as SHA-256; the answer to a
  sign-in request is the same whether or not the address has an account (except in `own`
  mode, where the operator address is public anyway). Sessions are 256-bit cookies stored as
  SHA-256; every signed-in form carries a token (HMAC of the session) and a cross-site
  `Origin` is refused.
- The core's link requests are checked in constant time; download tokens are 128-bit and
  looked up by hash; the name in the link must match.
- Uploads are held in memory while checked (≤ `EBC_SHELF_MAX_FILE_MB`, at most
  `EBC_SHELF_MAX_CONCURRENT_UPLOADS` at once); a larger body is cut at the limit.
- Every page: `Content-Security-Policy: default-src 'none'; style-src 'self'; img-src 'self';
  form-action 'self'; frame-ancestors 'none'; base-uri 'none'`, `nosniff`, `no-referrer`,
  `DENY`, `no-store`, `noindex`. Templates escape every value (`maud`).
- Calls to the core never follow redirects, cap answers, and log only the kind of failure.

## Known limits

- Storage is one JSON snapshot rewritten on every change (single writer process), as in the
  core; rate limits and download links are in memory (a restart forgets them).
- Gift codes are stored in clear in the shelf's database: the rights holder needs them to
  print cards (the core keeps only a hash). They are the partner's own data, like a store's
  orders; protect the data directory.
- No print orders from the shelf (the core's partner print API exists, §4.6; not wired).
- No webhook receiver: the shelf counts deliveries itself and reads nothing else back.
- Strings of the shelf (`src/text.rs`, seven languages) are **not reviewed by native
  speakers**; legal wording is **not reviewed by counsel**.

## Tests

```sh
cargo test -p ebc-shelf      # unit tests + tests/http.rs (the shelf over HTTP against a mock core that checks every signature)
cargo test -p ebc-demo-integration --test it etagere   # with the real core, simulated Kobo and PocketBook
bash e2e/run.sh tests/shelf.test.mjs                    # Chromium, JavaScript off
```
