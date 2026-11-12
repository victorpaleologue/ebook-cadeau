# From paper to e-reader — redemption UX from first principles (QR, typed code, pairing, claim rules, card text)

Research notes, 2026-11-12. Angle: UX design. Companion notes: `ereader-ux.devices.md` (what each device can do) and `ereader-ux.ecosystems.md` (reseller rails). This note does not repeat them; facts reused from them are tagged **[sib]**.

## 0. Method caveat — read first

- **No web search ran.** The shared search budget was spent before this agent started (3 attempts, 3 refusals). Only `raw.githubusercontent.com` is reachable from the sandbox, so I read primary documents hosted there (about 100 fetched, about 45 cited): NIST SP 800-63B, the IETF OAuth cross-device security draft, WCAG 2.2 "Understanding" pages, OWASP, GOV.UK Design System, MDN, device-flow docs, QR and code libraries, user-agent databases, send2ereader source.
- Every link below was fetched in this session (HTTP 200). RFC 8628 itself, crockford.com, w3.org and vendor help sites are blocked; statements about them are tagged [M] unless a fetched document repeats them. Because no source is independent journalism or measurement, most facts are [V1].
- Markers: **[V2]** two independent fetched sources · **[V1]** one · **[M]** memory, unverified · **[calc]** computed in this session (segno 1.6.6 for QR; plain arithmetic otherwise) · **[sib]** taken from a sibling note · no marker = design reasoning.
- **Not found at all:** published QR-to-download conversion studies, BookFunnel support data, Libby / cloudLibrary onboarding write-ups. Section 11 therefore rests on step counting, not on measured conversion.

## 1. Bottom line

1. **The card code is already a pairing code.** With a 12-letter code, typing it on the e-reader takes 4 actions and needs no phone. A separate phone-to-e-reader pair code only earns its place when the phone did something the e-reader cannot (signing in to a reseller account).
2. **Use letters only in everything a person types** (domain and code). Digits cost a layer switch on phone soft keyboards and, I believe, on e-ink ones; they need Shift on French AZERTY keyboards; and they create every classic confusion (0/O, 1/I/L, 5/S, 8/B). The keyboard facts are [M]; the e-ink one needs a hands-on check.
3. **Recommended code: 12 symbols from the 20-consonant set `BCDFGHJKLMNPQRSTVWXZ`, printed `XXXX-XXXX-XXXX`** (11 random + 1 check, 47.5 bits). This is the default alphabet of an OAuth device-flow implementation [V1], and GitHub's sample code `WDJB-MJHT` fits it. The current spec (10 Crockford symbols, 45 bits) is workable; the comparison is in section 3.
4. **QR payload in upper case** (`HTTPS://HOST/K/CODE`) stays in alphanumeric mode: version 2 (25×25 modules) instead of version 3 for the same 29-character URL at level M or Q [calc]. Print the symbol at 20 mm or more, K100, with the same URL in clear text beside it.
5. **Gift codes are not tied to an account, so every guess is a guess against all live codes.** At 45–48 bits the scheme holds only with global and per-client throttling and a bounded number of live codes [calc].
6. **The 6-digit pair number in `docs/api.md` is a bearer secret typed on the receiving side**: with 50 live numbers and 10,000 guesses an hour, 0.5 download sessions an hour would be stolen [calc]. Use 6 letters, single use, 15 minutes, a global failure budget — or flip the direction (e-reader shows, phone types), as send2ereader and Kobo's own activation do.
7. **Reseller-integrated redemption is the shortest path for a dedicated e-reader**: 3–4 actions, nothing typed on e-ink. Open-file redemption on the e-reader is 4 actions and about 18 characters.
8. **Three printed steps, one fallback line, no price, no expiry.** Exact text in section 12.
9. **Scratch-off is not worth it.** The code is made to order and mailed, not hung on a rack; "inside the folded jacket" plus cancel-and-reissue by the giver is proportionate.

## 2. QR code on print

| Fact | Source | Conf. |
|---|---|---|
| Error correction levels recover about 7 % (L), 15 % (M), 25 % (Q), 30 % (H) | [python-qrcode README](https://raw.githubusercontent.com/lincolnloop/python-qrcode/main/README.rst) | [V1] |
| Quiet zone: 4 modules is "the minimum according to the specs"; segno also defaults to four light modules | same; [segno docs](https://raw.githubusercontent.com/heuer/segno/master/docs/make.rst) | [V2] |
| Alphanumeric mode covers digits, upper-case A–Z, space and `$%*+-./:`; lower case forces byte mode | [segno modes](https://raw.githubusercontent.com/heuer/segno/master/docs/qrcode-modes.rst) | [V1] |
| "Inside QR codes uppercase SHOULD be used", alphanumeric mode being "45% more compact" than byte mode | [BIP-173](https://raw.githubusercontent.com/bitcoin/bips/master/bip-0173.mediawiki) | [V1] |
| Libraries can raise the error-correction level for free when the version does not change | [Nayuki README](https://raw.githubusercontent.com/nayuki/QR-Code-generator/master/Readme.markdown), segno docs | [V2] |
| ZXing accepts any scheme matching `[a-zA-Z][a-zA-Z0-9+-.]+:`, so `HTTPS://` parses as a URL | [URIResultParser.java](https://raw.githubusercontent.com/zxing/zxing/master/core/src/main/java/com/google/zxing/client/result/URIResultParser.java) | [V1] |
| iOS and Android native cameras open upper-case `HTTPS://` URLs | — | [M] test on real phones |
| Identifiers in URLs leak through logs, history, bookmarks and the Referer header | [OWASP Session Management](https://raw.githubusercontent.com/OWASP/CheatSheetSeries/master/cheatsheets/Session_Management_Cheat_Sheet.md) | [V1] |

Computed symbol sizes [calc]. `XXX.FR` stands for a 6-character host; the code is 12 symbols.

| Payload | Chars | Mode | Version at L / M / Q / H | Modules at Q |
|---|---|---|---|---|
| `HTTPS://XXX.FR/K/` + code | 29 | alphanumeric | 2 / 2 / 2 / 3 | 25×25 |
| same, lower case | 29 | byte | 2 / 3 / 3 / 4 | 29×29 |
| `HTTPS://EBOOK-CADEAU.FR/K/` + code | 38 | alphanumeric | 2 / 2 / 3 / 4 | 29×29 |
| same, lower case | 38 | byte | 3 / 3 / 4 / 5 | 33×33 |
| `HTTPS://WWW.FNAC.COM/EBOOK-CADEAU/` + code | 46 | alphanumeric | 2 / 3 / 3 / 4 | 29×29 |
| same, lower case | 46 | byte | 3 / 4 / 4 / 6 | 33×33 |
| 98-character URL with tracking parameters | 98 | byte | 5 / 6 / 8 / 9 | 49×49 |

Capacities measured with segno match the figures the prepress note held from memory (v2-M 38 alphanumeric / 26 bytes, v3-M 61/42, v3-Q 47/32, v4-M 90/62, v4-Q 67/46) [calc].

Rules that follow:

- **Payload** = `HTTPS://` + shortest host + `/K/` + code, all upper case, no query string. The server must match the path case-insensitively. A redirect on our side keeps the flow changeable after printing.
- **Size**: version 2 at 0.8 mm per module is a 20.0 mm symbol (26.4 mm with quiet zone); version 3 at 0.7 mm is 20.3 mm (25.9 mm) [calc]. A low version with large modules is the robust choice for old phones and dim rooms. Preflight floor: 0.5 mm per module (about 6 printer dots at 300 dpi [calc]); the 0.4 mm floor and the "width ≈ distance / 10" rule are [M] here and in the prepress note.
- **Level**: ask for M, let the encoder boost to Q when the version is unchanged. H is for logo overlays, which we do not use.
- **Ink and place**: K100 vector, at least 5 mm from folds and trim [sib, V2 for K-only]. Never across the spine creases.
- **Print the host and code in clear next to the symbol.** It is the typed fallback, and it lets the recipient see where the code leads. The cross-device draft says the user experience SHOULD make the context of a request clear, and MAY tell users where a legitimate QR code appears, for instance only on a specific domain [V1: [draft](https://raw.githubusercontent.com/oauth-wg/oauth-cross-device-security/main/draft-ietf-oauth-cross-device-security.md)].
- **After the scan**: the page sends `Referrer-Policy: no-referrer`, loads nothing from third parties, and swaps the code for a short-lived download session (already in `docs/api.md`).

## 3. The typed code

### 3.1 Evidence

| Fact | Source | Conf. |
|---|---|---|
| Crockford base32: 10 digits + 22 letters, no I, L, O, U; decoding ignores case and hyphens, maps I/L to 1 and O to 0 | [base32-crockford README](https://raw.githubusercontent.com/jbittel/base32-crockford/master/README.rst); purpose confirmed by the [crockford crate](https://raw.githubusercontent.com/archer884/crockford/master/README.md) | [V1] |
| Its check symbol is the value mod 37 and needs five extra symbols `* ~ $ = U` — not typeable on e-ink | [source](https://raw.githubusercontent.com/jbittel/base32-crockford/master/base32_crockford.py) | [V1] |
| Device-flow user codes are 8 symbols with a hyphen in the middle: default mask `****-****`; GitHub: "8 characters with a hyphen in the middle" (`WDJB-MJHT`) | [node-oidc-provider docs](https://raw.githubusercontent.com/panva/node-oidc-provider/main/docs/README.md), [GitHub docs](https://raw.githubusercontent.com/github/docs/main/content/apps/oauth-apps/building-oauth-apps/authorizing-oauth-apps.md) | [V2] |
| Default charset `BCDFGHJKLMNPQRSTVWXZ` ("excludes easily confused characters"); the alternative offered is digits only | node-oidc-provider docs | [V1]; that RFC 8628 recommends this set is [M] |
| Open Location Code uses 20 symbols "selected to reduce writing errors and prevent accidentally spelling words" | [OLC spec](https://raw.githubusercontent.com/google/open-location-code/main/Documentation/Specification/specification.md) | [V1] |
| Bech32: charset without 1, b, i, o, chosen from visual-similarity data; checksum detects any error touching up to 4 characters; mixed case rejected | [BIP-173](https://raw.githubusercontent.com/bitcoin/bips/master/bip-0173.mediawiki) | [V1] |
| send2ereader key: 4 symbols from `23456789ACDEFGHJKLMNPRSTUVWXYZ`, dead 30 s after the page closes, 1 h cap | [index.js](https://raw.githubusercontent.com/daniel-j/send2ereader/master/index.js) | [V1] |
| Printed "look-up secrets": at least 20 bits; under 64 bits the verifier SHALL rate-limit; stored salted and hashed; each secret used once | [NIST SP 800-63B §5.1.2](https://raw.githubusercontent.com/usnistgov/800-63-3/nist-pages/sp800-63b/sec5_authenticators.md) | [V1] |
| Throttle: at most 100 consecutive failures per account; growing delays (30 s up to 1 h) and CAPTCHA are allowed tools | same, §5.2.2 | [V1] |
| Usability: allow at least 10 entry attempts; say how many remain and how long to wait; typing on small devices is "significantly more error prone" | [NIST §10](https://raw.githubusercontent.com/usnistgov/800-63-3/nist-pages/sp800-63b/sec10_usability.md) | [V1] |
| Lockout per account invites denial of service; per-IP limits are evaded with proxy lists | [OWASP brute force](https://raw.githubusercontent.com/OWASP/www-community/master/pages/controls/Blocking_Brute_Force_Attacks.md) | [V1] |
| Codes must be random (CSPRNG), long enough, single use, expiring | [OWASP Forgot Password](https://raw.githubusercontent.com/OWASP/CheatSheetSeries/master/cheatsheets/Forgot_Password_Cheat_Sheet.md) | [V1] |
| Entry field: accept extra spaces and hyphens; no `maxlength`; never block paste; letter-space codes so the user can check them | [GOV.UK security code](https://raw.githubusercontent.com/alphagov/govuk-design-system/main/src/patterns/confirm-a-phone-number/index.md), [GOV.UK text input](https://raw.githubusercontent.com/alphagov/govuk-design-system/main/src/components/text-input/index.md) | [V1] |
| `autocapitalize="characters"` acts on virtual keyboards only | [MDN](https://raw.githubusercontent.com/mdn/content/main/files/en-us/web/html/reference/global_attributes/autocapitalize/index.md) | [V1] |

### 3.2 Brute force: the right model

A gift code has no username. An attacker does not attack one code; each guess succeeds with probability *live codes / code space*. Per-code lockout is therefore useless; the controls are entropy, a per-client limit, a **global** failure budget with alerting, and keeping the number of live codes bounded (dead after cancellation, fulfilment or the last allowed download).

Expected successful guesses, assuming an attacker sustains 200,000 guesses a day through 10,000 addresses (an assumption, not a measurement) [calc]:

| Scheme | Bits | 10,000 live codes | 100,000 | 1,000,000 |
|---|---|---|---|---|
| Crockford, 9 random + check (current spec) | 45.0 | 1 per 48 years | 1 per 4.8 years | 1 per 6 months |
| 20 consonants, 11 random + check (**proposed**) | 47.5 | 1 per 280 years | 1 per 28 years | 1 per 2.8 years |
| 20 consonants, 12 random + check | 51.9 | 1 per 5,600 years | 1 per 560 years | 1 per 56 years |
| Crockford, 11 random + check | 55.0 | 1 per 49,000 years | 1 per 4,900 years | 1 per 490 years |
| 12 digits | 39.9 | 1 per 1.4 years | 1 per 7 weeks | 1 per 5 days |

The prize is one eBook, so the incentive is weak. The real damage is reputational, and with watermarked files a stolen copy is traced to an innocent order. Build length agility in now: the parser can tell 12 from 15 symbols.

### 3.3 Which alphabet

Taps on a keyboard that hides digits behind a layer key, counting each layer change [calc]:

| Scheme | Symbols | Taps | Bits per tap |
|---|---|---|---|
| Crockford (31 % digits) | 10 | 14.2 | 3.2 |
| Crockford | 12 | 17.0 | 3.2 |
| 20 consonants | 12 | 12.0 | 4.0 |

| Keyboard | Digits cost | Conf. |
|---|---|---|
| iOS / Android default soft keyboard | layer switch | [M] high |
| Kobo, Kindle, PocketBook on-screen keyboards | layer switch believed; **not verified on hardware** | [M] low |
| French AZERTY physical keyboard | Shift for every digit unless a numeric keypad is present | [M] high |
| QWERTY physical keyboard | none | — |

Letters-only wins or ties everywhere except QWERTY desktops, removes every digit/letter confusion, and has no vowels, so it cannot print a word on a gift. Cost: two more printed characters than the current spec for 2.5 more bits.

**Recommendation.** `BCDFGHJKLMNPQRSTVWXZ`, 12 symbols, `XXXX-XXXX-XXXX`; the last symbol is a check symbol from the same alphabet. A Damm-style check works at order 20: write each symbol index as 5·h + l with h in GF(4) and l in ℤ/5, combine states with x∘y = a·x + y in each part (a = 2 in ℤ/5; a = the element 10₂ in GF(4), modulus x²+x+1), and pick the check symbol that brings the state to 0. Verified here: no violation of the anti-symmetry condition over all 8,000 triples, and 456,000 substitutions plus 20,859 adjacent swaps on random codes all detected [calc]. The existing requirement in `docs/architecture.md` carries over unchanged. Input is case-insensitive and ignores spaces, hyphens and dots. A typo caught by the check symbol is reported as a typo and never counts as an attempt (already in `docs/api.md`). Keep a short blocklist of acronyms. If Crockford is kept instead: lengthen to 12, add a word blocklist (vowels A and E remain), and test AZERTY entry.

## 4. Pairing a phone with an e-reader

Evidence for the pattern:

| Fact | Source | Conf. |
|---|---|---|
| Device flow: the constrained device shows a code and a short URL; the user enters it on a second device; the device polls | [GitHub docs](https://raw.githubusercontent.com/github/docs/main/content/apps/oauth-apps/building-oauth-apps/authorizing-oauth-apps.md), [Microsoft](https://raw.githubusercontent.com/MicrosoftDocs/entra-docs/main/docs/identity-platform/v2-oauth2-device-code.md) | [V2] |
| Codes live 15 minutes (GitHub `expires_in` 900, Microsoft default); GOV.UK uses 15 minutes for SMS codes | same; [GOV.UK](https://raw.githubusercontent.com/alphagov/govuk-design-system/main/src/patterns/confirm-a-phone-number/index.md) | [V2] |
| Polling every 5 s; 50 code submissions per hour per application | GitHub docs | [V1] |
| "You could define a short verification_uri" that redirects to the real one | [Keycloak docs](https://raw.githubusercontent.com/keycloak/keycloak/main/docs/documentation/server_admin/topics/sso-protocols/con-oidc-auth-flows.adoc) | [V1] |
| The channel between the two devices is unauthenticated; mitigations: short-lived codes, one-time or limited use ("MAY allow the same code to be presented a few times"), showing the user who asks and for what, an obvious way to decline | [IETF cross-device draft](https://raw.githubusercontent.com/oauth-wg/oauth-cross-device-security/main/draft-ietf-oauth-cross-device-security.md) | [V1] |
| The implementation exposes the requesting device's IP and user agent to the confirmation screen | [node-oidc-provider](https://raw.githubusercontent.com/panva/node-oidc-provider/main/docs/README.md) | [V1] |
| **Kobo e-readers already sign in this way**: "web-based activation… open an activation link… and enter the code" | [kobo-book-downloader README](https://raw.githubusercontent.com/TnS-hun/kobo-book-downloader/master/README.md) | [V1] |
| send2ereader: the e-reader displays the key, the other device types it; the download is refused if the user agent differs from the one that created the key | [index.js](https://raw.githubusercontent.com/daniel-j/send2ereader/master/index.js) | [V1] |
| Libby uses a "setup code" to bring a new device into an existing account | [odmpy README](https://raw.githubusercontent.com/ping/odmpy/master/README.md) | [V1]; 8 digits and about one minute of validity are [M] |
| "Some e-ink devices have browsers that do not handle authentication" | [calibre manual](https://raw.githubusercontent.com/kovidgoyal/calibre/master/manual/server.rst) | [V1] |

Direction matters:

| | A. Phone shows, e-reader types (`docs/api.md` today) | B. E-reader shows, phone types (device-flow direction) |
|---|---|---|
| Typed on e-ink | host + code | host only |
| E-reader page | plain form, no polling | needs refresh: 5 s meta refresh or a "Continuer" button |
| What a guessed code yields | **someone's download** | nothing: the guesser must hold a gift |
| Residual attack | brute force on `/e` | tricking a victim into typing the attacker's code |
| Flow shape | linear (phone, then e-reader) | back and forth |

A is simpler to follow and works without JavaScript; it needs a stronger code than 6 digits: 6 letters (64 million values) cut the theft rate in the example above from 0.5 to 0.008 an hour [calc]. Add single use, 15-minute life with an absolute expiry time on screen and one-tap renewal, and a global failure budget on `/e`. B is the safer design where refresh works; the confirmation screen must then name the device class ("une liseuse Kobo demande ce livre").

**Where pairing is needed at all**: after an account sign-in on the phone, when the e-reader must fetch a file or licence (`reseller_licence`). For a plain hosted file, typing the card code on the e-reader is shorter in actions (section 11).

## 5. Claim rules

| Question | Recommendation | Basis |
|---|---|---|
| States | created → (reveal date reached) → opened → claimed → exhausted; plus cancelled, reissued | legal note asks for revoked and reissued states [sib] |
| Hosted file | The card is the key: re-downloadable by its holder, capped (10 sessions in `docs/api.md`). The gift page shows "téléchargé n fois, dernière fois le …" so misuse is visible | Bookwire lets the publisher set redemptions and validity; BookFunnel print codes are single use [sib] |
| Reseller library | One-time, bound to the account that claims. Afterwards the page says the book is already in an account, without showing the address | — |
| Expiry | None, or at least 5 years; print nothing about expiry unless there is one | legal note [sib, M] |
| Lost before claim | The giver's manage link cancels the code and issues a new card PDF | 14-day withdrawal also needs revocation [sib, M] |
| Lost after claim | Account: nothing to do. Hosted file: the copy is on the device; the giver can reissue while sessions remain | — |
| Storage | Code stored salted and hashed, never logged in clear | NIST §5.1.2 [V1] |

**E-mail-less redemption.** For hosted files no account, e-mail or cookie is needed: possession of the card is the credential. Watermark with the gift or order reference rather than the recipient's identity; offer a name field only as an optional ex-libris. Optional extras, both opt-in: "send me a backup link", and notifying the giver that the gift was opened (say so on the gift page before the recipient acts).

## 6. Landing page: progressive disclosure

- **First screen is the gift, not a form**: cover, title, "De la part de …", the message, one primary button chosen for the device, and a visible "Autres façons de lire" list.
- **Device hint from the user agent, never a gate.** Public parsers key on `Kindle/x.y` and `Kobo` (both databases), `PocketBook/`, `tolino`, `TOLINO_BROWSER` (one each) [V2 for Kindle and Kobo: [uap-core](https://raw.githubusercontent.com/ua-parser/uap-core/master/regexes.yaml), [Matomo devices](https://raw.githubusercontent.com/matomo-org/device-detector/master/regexes/device/mobiles.yml), [Matomo browsers](https://raw.githubusercontent.com/matomo-org/device-detector/master/regexes/client/browsers.yml)]. Sample strings: `…Kindle/3.0 (screen 600x800; rotate)`, `…Mobile Safari/533.1 (Kobo Touch)` [V1: [uap-core tests](https://raw.githubusercontent.com/ua-parser/uap-core/master/tests/test_device.yaml)]; `…AppleWebKit/534.34 PocketBook/623 (screen 758x1024; FW T623.4.4.616) Mobile` [V1: [Matomo fixtures](https://raw.githubusercontent.com/matomo-org/device-detector/master/Tests/fixtures/tablet-3.yml)]. Recent Kobo and Kindle firmware changed these strings (`eReader`, missing `Kindle`) [sib], so the typed entry point must work with no detection at all.
- **Engine age**: the sampled engines report WebKit 528 to 538, which I date to roughly 2009–2014 [V1 for the strings; the dating and whether current firmware is newer are M]. Server-rendered HTML, forms, no script needed, port 443 only (old Kindles cannot reach port 8080 and Kobo browsers have trouble on non-standard ports [V1 each: [calibre FAQ](https://raw.githubusercontent.com/kovidgoyal/calibre/master/manual/faq.rst), [Calibre-Web FAQ](https://raw.githubusercontent.com/wiki/janeczku/calibre-web/FAQ.md)]). TLS versions and root stores on old firmware are unknown [M]: test before choosing a certificate chain.
- **Code field**: `type="text"`, `autocapitalize="characters"`, `autocomplete="off"`, `spellcheck="false"`, no `maxlength`, letter-spaced, paste allowed. `autocomplete="one-time-code"` is defined for sign-in OTPs, not for this [V1: [MDN](https://raw.githubusercontent.com/mdn/content/main/files/en-us/web/html/reference/attributes/autocomplete/index.md)].
- **Auto-refresh** on the e-reader is acceptable only on the waiting page; MDN warns that refresh disorients assistive-technology users [V1: [MDN](https://raw.githubusercontent.com/mdn/content/main/files/en-us/web/html/reference/elements/meta/http-equiv/index.md)], so pair it with a manual button.
- **Read now** on phones and computers: an in-browser reader is feasible for unprotected EPUB (foliate-js handles EPUB, MOBI, KF8, and "does not care about older browsers" [V1: [README](https://raw.githubusercontent.com/johnfactotum/foliate-js/main/README.md)]) — never on e-ink.

## 7. Accessibility

| Requirement | Source | Conf. |
|---|---|---|
| Retyping a code is a "cognitive function test"; a compliant flow offers a path without transcription and must allow paste. The criterion is scoped to authentication; applied here by analogy: the QR code is the no-transcription path | [WCAG 3.3.8](https://raw.githubusercontent.com/w3c/wcag/main/understanding/22/accessible-authentication-minimum.html) | [V1] |
| Targets at least 24×24 CSS px, or spaced | [WCAG 2.5.8](https://raw.githubusercontent.com/w3c/wcag/main/understanding/22/target-size-minimum.html) | [V1] |
| Contrast 4.5:1; 3:1 for text from 18 pt or 14 pt bold | [WCAG 1.4.3](https://raw.githubusercontent.com/w3c/wcag/main/understanding/20/contrast-minimum.html) | [V1] |
| Users must be able to turn off, adjust or extend time limits; limits over 20 hours are exempt → the pair code needs one-tap renewal | [WCAG 2.2.1](https://raw.githubusercontent.com/w3c/wcag/main/understanding/20/timing-adjustable.html) | [V1] |
| Do not ask again for what was already entered (the code survives a sign-in redirect) | [WCAG 3.3.7](https://raw.githubusercontent.com/w3c/wcag/main/understanding/22/redundant-entry.html) | [V1] |
| Help in the same place on every page | [WCAG 3.2.6](https://raw.githubusercontent.com/w3c/wcag/main/understanding/22/consistent-help.html) | [V1] |
| Print: black on white, at least 12 pt, a typeface that separates confusable glyphs; serif suggested for print | [NIST §10](https://raw.githubusercontent.com/usnistgov/800-63-3/nist-pages/sp800-63b/sec10_usability.md) | [V1] |
| European Accessibility Act: Directive (EU) 2019/882, public and private sector | [W3C WAI policy list](https://raw.githubusercontent.com/w3c/wai-policies-prototype/master/_policies/european-union.md) | [V1]; that it covers e-books and e-commerce since 28 June 2025, with a micro-enterprise exemption for services, is [M] |

For the card: instructions at 12 pt or more, the code at 16 pt bold in a face with open shapes, groups of four. Blind recipients cannot locate a printed QR code: let the giver also send the same link by e-mail or SMS.

## 8. Anti-fraud for printed codes

| Threat | Applies here? | Measure |
|---|---|---|
| Guessing codes | Yes | Section 3.2; alert on global failure rate |
| Tampering on a retail rack (reading or swapping codes before sale) | No: cards are made to order; racks failed commercially anyway [sib] | If ever needed: activation at the till and sealed packaging |
| Printer or postal staff reading the code | Low | Code on the inside panel; optional seal sticker; reveal date ("ne s'ouvre pas avant le 25/12") keeps the code inert until the day; giver can cancel and reissue |
| Photo of the open card posted online | Low | Download cap; claim history on the gift page |
| QR sticker over the real code | No for a mailed card | Host printed in clear beside the symbol |
| Pair-code theft or phishing | Yes if pairing ships | Section 4 |

Scratch-off panels: extra finishing cost, impossible for print-at-home, hard for people with limited dexterity, and they make a keepsake look like a lottery ticket [M for cost]. Not recommended.

## 9. Gift moment

- The object says "book", not "voucher": headline "Ce livre est un cadeau", never a price [sib].
- Leave a blank area for a handwritten note; print the typed message only if the giver wrote one.
- The scan reveals the cover and message first; the technical choice comes second.
- Optional reveal date chosen by the giver. Before it, the page says when the gift opens and nothing else.
- The card stays useful: a later scan shows "Vous lisez ce livre depuis le …" and a way back to the file or library.

## 10. Failure recovery

| Failure | What the recipient sees | Way out |
|---|---|---|
| QR will not scan | — | Host and code in clear on the card |
| Typo (check symbol fails) | "Il y a une faute de frappe dans le code. Cet essai ne compte pas." | Retype; O/0-type confusion cannot occur with letters only |
| Unknown code | Neutral message, attempts left, waiting time [NIST, V1] | Help link |
| Wrong device guessed | "Autres façons de lire" always visible | Manual choice |
| Download fails on the e-reader | Session can be minted again | USB route from a computer |
| Send to Kindle rejects the file (Amazon's EPUB intake is "very flawed" [V1: [calibre FAQ](https://raw.githubusercontent.com/kovidgoyal/calibre/master/manual/faq.rst)]) | — | Kindle app share sheet; USB |
| Already claimed by someone else | Date and count, no identity | Support contact; giver can check the manage link |
| Title withdrawn from sale | — | Substitute or refund rule [sib] |

One help address, printed on the card and present at the same place on every page; NIST asks to "clearly communicate how and where to acquire technical assistance" [V1].

## 11. Recommended flow and step counts

An action is one deliberate act: a scan, a tap, a field typed, a sign-in. "E-ink characters" are typed on the e-reader. Device capabilities are from the devices note [sib]; counts are reasoning, not measurements.

| Situation | Path | Actions | E-ink characters |
|---|---|---|---|
| Reads on phone or tablet, open file | scan → "Lire sur ce téléphone" → open in the reading app | 3 | 0 |
| Same, in-browser reader | scan → "Lire maintenant" | 2 | 0 |
| Kobo/Fnac, Vivlio-network, tolino — reseller-integrated | scan → "Ajouter à ma bibliothèque" → sign in → sync the e-reader | 3–4 (+1 form if no account) | 0 |
| Kobo, PocketBook, Vivlio, tolino — open file, on the e-reader | open browser → type host → type code → "Télécharger" | 4 | about 18 |
| Same, after a sign-in on the phone (pair code) | scan → sign in → "Envoyer sur ma liseuse" → open browser → host → 6 letters | 6 | about 14 |
| Kindle — open EPUB, Kindle app on the phone | scan → "Envoyer à mon Kindle" → pick Kindle in the share sheet → send | 4 | 0 — [M] |
| Kindle — e-mail route | scan → "Envoyer à mon Kindle" → type the `@kindle.com` address → send; once: approve our sender address in the Amazon account | 4 + setup | 0 — fragile [sib] |
| Kindle — browser download (only if a Kindle-format file exists) | open browser → host → code → download → OK | 5 | about 18; no cover, not in the library [V1: [Standard Ebooks](https://raw.githubusercontent.com/standardebooks/web/master/www/help/how-to-use-our-ebooks.php)] |
| Kindle — Amazon-issued gift wrapped by the card | scan → Amazon sign-in → accept | 3 | 0 — [M] |
| No smartphone, computer and cable | host → code → download → plug in → copy → eject | 6 | 0 |
| No reading device | host → code → read in the browser | 3 | 0 |

Order of preference on the gift page: reseller library (if the gift is of that type) → read on this device → e-reader instructions (type the card code there) → Kindle app → pair code → USB.

## 12. What to print

Inside panel, French, infinitive form so the card does not choose tu or vous (terminology note [sib]). Braces are variables.

```
Ce livre est un cadeau
De la part de {prénom}
{message, ou espace libre pour écrire à la main}

POUR LE LIRE
1  Scanner ce code avec l'appareil photo d'un téléphone.
2  Choisir où lire : liseuse, tablette ou téléphone.
3  Le livre arrive. Bonne lecture !

[QR 20 mm min.]     {domaine}
                    BCDF-GHJK-LMNP

Sans téléphone ? Sur la liseuse ou un ordinateur, ouvrir le navigateur,
aller sur {domaine} et saisir le code ci-dessus.

{Titre}, {Auteur}. Un exemplaire numérique, déjà payé : rien à régler.
{Format : « EPUB sans DRM » | « à lire avec un compte {revendeur} »}.
Sans date limite. Une aide ? {domaine}/aide · {contact}
```

Reseller-integrated variant, steps 2 and 3: "2  Se connecter à son compte {revendeur}, ou en créer un." / "3  Le livre apparaît dans la bibliothèque, sur la liseuse et l'application."

Constraints: `{domaine}` is letters and one dot, no hyphen, no digit, as short as the budget allows; printed in lower case, encoded in upper case in the QR. Steps at 12 pt or more; code at 16 pt bold; all text K100. The legal line carries what the legal note lists as required information (what the card gives, single copy, validity, format, contact) [sib, M].

## 13. Suggested changes to `docs/api.md` v0.1

1. `GiftCode`: 12 symbols from the 20-consonant alphabet (or Crockford lengthened to 12); length-agile parser.
2. QR payload upper case; `/K/` and `/k/` equivalent.
3. Pair number: 6 letters, single use, 15 minutes, global failure budget on `POST /e`; consider the reversed direction.
4. Make "type the card code at `/` on the e-reader" the documented default; pairing only after a phone-side sign-in.
5. `mobile` class: offer the Kindle app share route before the e-mail form.
6. Gift page: claim count and last date; remaining attempts and waiting time on errors.
7. Optional reveal date; opt-in giver notification.

## Sources

All fetched 2026-11-12 from `raw.githubusercontent.com`.

- Device flow and cross-device security: [GitHub OAuth docs](https://raw.githubusercontent.com/github/docs/main/content/apps/oauth-apps/building-oauth-apps/authorizing-oauth-apps.md) · [Microsoft device code](https://raw.githubusercontent.com/MicrosoftDocs/entra-docs/main/docs/identity-platform/v2-oauth2-device-code.md) · [IETF OAuth cross-device security draft](https://raw.githubusercontent.com/oauth-wg/oauth-cross-device-security/main/draft-ietf-oauth-cross-device-security.md) · [node-oidc-provider](https://raw.githubusercontent.com/panva/node-oidc-provider/main/docs/README.md) · [Keycloak](https://raw.githubusercontent.com/keycloak/keycloak/main/docs/documentation/server_admin/topics/sso-protocols/con-oidc-auth-flows.adoc)
- Codes and throttling: [NIST SP 800-63B §5](https://raw.githubusercontent.com/usnistgov/800-63-3/nist-pages/sp800-63b/sec5_authenticators.md) · [NIST SP 800-63B §10](https://raw.githubusercontent.com/usnistgov/800-63-3/nist-pages/sp800-63b/sec10_usability.md) · [OWASP Forgot Password](https://raw.githubusercontent.com/OWASP/CheatSheetSeries/master/cheatsheets/Forgot_Password_Cheat_Sheet.md) · [OWASP Blocking Brute Force](https://raw.githubusercontent.com/OWASP/www-community/master/pages/controls/Blocking_Brute_Force_Attacks.md) · [OWASP Session Management](https://raw.githubusercontent.com/OWASP/CheatSheetSeries/master/cheatsheets/Session_Management_Cheat_Sheet.md) · [base32-crockford README](https://raw.githubusercontent.com/jbittel/base32-crockford/master/README.rst) and [source](https://raw.githubusercontent.com/jbittel/base32-crockford/master/base32_crockford.py) · [crockford crate](https://raw.githubusercontent.com/archer884/crockford/master/README.md) · [BIP-173](https://raw.githubusercontent.com/bitcoin/bips/master/bip-0173.mediawiki) · [Open Location Code](https://raw.githubusercontent.com/google/open-location-code/main/Documentation/Specification/specification.md)
- QR: [python-qrcode](https://raw.githubusercontent.com/lincolnloop/python-qrcode/main/README.rst) · [segno make](https://raw.githubusercontent.com/heuer/segno/master/docs/make.rst) · [segno modes](https://raw.githubusercontent.com/heuer/segno/master/docs/qrcode-modes.rst) · [Nayuki](https://raw.githubusercontent.com/nayuki/QR-Code-generator/master/Readme.markdown) · [ZXing URIResultParser](https://raw.githubusercontent.com/zxing/zxing/master/core/src/main/java/com/google/zxing/client/result/URIResultParser.java)
- Devices and pairing precedents: [send2ereader index.js](https://raw.githubusercontent.com/daniel-j/send2ereader/master/index.js) · [kobo-book-downloader](https://raw.githubusercontent.com/TnS-hun/kobo-book-downloader/master/README.md) · [odmpy](https://raw.githubusercontent.com/ping/odmpy/master/README.md) · [calibre server manual](https://raw.githubusercontent.com/kovidgoyal/calibre/master/manual/server.rst) · [calibre FAQ](https://raw.githubusercontent.com/kovidgoyal/calibre/master/manual/faq.rst) · [Calibre-Web FAQ](https://raw.githubusercontent.com/wiki/janeczku/calibre-web/FAQ.md) · [Standard Ebooks help](https://raw.githubusercontent.com/standardebooks/web/master/www/help/how-to-use-our-ebooks.php) · [foliate-js](https://raw.githubusercontent.com/johnfactotum/foliate-js/main/README.md)
- User agents: [uap-core regexes](https://raw.githubusercontent.com/ua-parser/uap-core/master/regexes.yaml) · [uap-core tests](https://raw.githubusercontent.com/ua-parser/uap-core/master/tests/test_device.yaml) · [Matomo devices](https://raw.githubusercontent.com/matomo-org/device-detector/master/regexes/device/mobiles.yml) · [Matomo browsers](https://raw.githubusercontent.com/matomo-org/device-detector/master/regexes/client/browsers.yml) · [Matomo fixtures](https://raw.githubusercontent.com/matomo-org/device-detector/master/Tests/fixtures/tablet-3.yml)
- Accessibility and forms: WCAG 2.2 Understanding — [3.3.8](https://raw.githubusercontent.com/w3c/wcag/main/understanding/22/accessible-authentication-minimum.html) · [2.5.8](https://raw.githubusercontent.com/w3c/wcag/main/understanding/22/target-size-minimum.html) · [1.4.3](https://raw.githubusercontent.com/w3c/wcag/main/understanding/20/contrast-minimum.html) · [2.2.1](https://raw.githubusercontent.com/w3c/wcag/main/understanding/20/timing-adjustable.html) · [3.3.7](https://raw.githubusercontent.com/w3c/wcag/main/understanding/22/redundant-entry.html) · [3.2.6](https://raw.githubusercontent.com/w3c/wcag/main/understanding/22/consistent-help.html) · [W3C WAI EU policies](https://raw.githubusercontent.com/w3c/wai-policies-prototype/master/_policies/european-union.md) · [GOV.UK security code pattern](https://raw.githubusercontent.com/alphagov/govuk-design-system/main/src/patterns/confirm-a-phone-number/index.md) · [GOV.UK text input](https://raw.githubusercontent.com/alphagov/govuk-design-system/main/src/components/text-input/index.md) · MDN [autocapitalize](https://raw.githubusercontent.com/mdn/content/main/files/en-us/web/html/reference/global_attributes/autocapitalize/index.md), [autocomplete](https://raw.githubusercontent.com/mdn/content/main/files/en-us/web/html/reference/attributes/autocomplete/index.md), [meta refresh](https://raw.githubusercontent.com/mdn/content/main/files/en-us/web/html/reference/elements/meta/http-equiv/index.md)
- Sibling notes: `ereader-ux.devices.md`, `ereader-ux.ecosystems.md`, `print-requirements.prepress.md`, `print-requirements.folding.md`, `legal-gifting-commerce.commerce.md`, `market-business.competitors-costs.md`, `terminology.naming.md`; project contract `docs/api.md`.

## Could not verify

1. **Any measured conversion data**: QR scan-to-download rates, drop-off per step, BookFunnel support statistics, Libby or cloudLibrary onboarding findings. No source reachable.
2. **RFC 8628 text** (user-code recommendations, brute-force and remote-phishing sections, `verification_uri_complete`): known only through the implementations and the IETF draft cited above.
3. **On-screen keyboards of Kobo, Kindle, PocketBook and Vivlio**: whether digits sit on a second layer, whether a French layout is offered, typing speed. The letters-only recommendation leans on this; a hands-on test on two devices settles it.
4. **AZERTY digit entry** and soft-keyboard layer costs: from memory, not sourced.
5. **Phone cameras and upper-case `HTTPS://` URLs**: only ZXing's parser was read. Test on iOS and Android stock cameras before printing.
6. **Physical QR sizing rules** (0.4 mm module floor, distance / 10): memory in both this note and the prepress note; no standard or printer guide was read.
7. **Share-sheet route to the Kindle app** and its current limits; Amazon gift links for Kindle books in France.
8. **TLS versions, certificate stores and JavaScript level** of current e-reader browsers; the engine dates given are inferred from user-agent samples that may be years old.
9. **Libby setup-code details** (8 digits, validity) and **Kobo's activation URL and code format**.
10. **European Accessibility Act scope and dates** beyond the directive's existence; French transposition.
11. **Attack-rate assumption** (200,000 guesses a day): a modelling choice. Real limits depend on the throttling actually deployed.
12. **Scratch-off and seal-sticker costs**, and any published gift-card fraud figures: not researched.
