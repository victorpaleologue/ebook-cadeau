# ebc-cli — the `ebc` command

`ebc` is the command-line face of eBook Cadeau. Each subcommand is a thin layer over one
library crate:

| Subcommand  | Library         | What it does                                                        |
|-------------|-----------------|---------------------------------------------------------------------|
| `isbn`      | `ebc-isbn`      | validate, convert (ISBN-10 ↔ 13), hyphenate; `--extract` from text  |
| `lookup`    | `ebc-catalog`   | metadata lookup across catalogue providers, merged, with licences   |
| `cover`     | `ebc-covers`    | list and preview cover candidates; save one only in operator mode   |
| `card`      | `ebc-card`      | compose the print-ready card PDF (PDF/X-4 or plain PDF for home)    |
| `preflight` | `ebc-preflight` | check a PDF against a print profile and card format                 |
| `qr`        | `ebc-qr`        | write a QR code as SVG                                              |
| `fontid`    | `ebc-fontid`    | rank openly licensed fonts by resemblance to a cover's title        |

```sh
cargo build -p ebc-cli          # binary: target/debug/ebc
cargo run -p ebc-cli -- --help
```

## Streams and exit status

Results go to standard output; notices, warnings and progress to standard error, so
`ebc … --json | jq` always sees valid JSON. `-o -` writes a PDF or SVG to standard output
(the summary then moves to standard error).

| Status | Meaning                                                          |
|-------:|------------------------------------------------------------------|
| 0      | success (`preflight`: the PDF passed)                            |
| 1      | failure: I/O, network, input refused (bad ISBN, bad image, …)    |
| 2      | `preflight` ran and found errors                                 |
| 3      | nothing found (`lookup`, `cover`, `isbn --extract`, `fontid`)    |
| 64     | usage error (unknown flag, flags that do not go together)        |

Status 2 and 64 are distinct on purpose: a script can tell "the file has errors" from
"I mistyped a flag". Closing the pipe early (`ebc … | head`) is not an error.

## Examples

### ISBN

```console
$ ebc isbn 2-07-036002-4
isbn13      9782070360024
hyphenated  978-2-07-036002-4
isbn10      2070360024
            2-07-036002-4
group       French language (978-2)
registrant  07
publication 036002

$ ebc isbn --extract notes.txt          # one hyphenated ISBN per line; exit 3 if none
$ cat notes.txt | ebc isbn --extract - --json
```

### Metadata lookup

```sh
ebc lookup --list-providers                       # ids, licences, terms, defaults
ebc lookup 9782070360024                          # default cascade, stops when complete
ebc lookup 9782070360024 --provider bnf --provider openlibrary --all --json
EBC_GOOGLE_BOOKS_KEY=… ebc lookup 9780141182636   # enables Google Books (keys are
                                                  # never printed; prefer the env var)
```

Real HTTP needs the cargo feature `net` (on by default: `ebc-catalog/client`, reqwest +
rustls on a current-thread tokio runtime). Each provider's minimum interval between two
requests to one host is honoured. Without `net`, or for reproducible runs:

```sh
# a local fixture server (or any mirror): PROVIDER=URL, repeatable
ebc lookup 9782070360024 --provider openlibrary \
    --base-url openlibrary=http://127.0.0.1:8080 --no-proxy
# answers from a directory: DIR/index.json maps URLs to recorded bodies
ebc lookup 9782070360024 --offline-fixtures ./recorded
```

The `index.json` format (the one `cargo run -p ebc-catalog --example record-fixtures`
writes) is documented in `src/fixtures.rs`. File paths in it cannot leave the directory.

### Cover candidates

```console
$ ebc cover 9782070360024
 1. openlibrary  locate   risk medium https://covers.openlibrary.org/b/isbn/9782070360024.json
    fact request: is there a cover, how large
 2. openlibrary  preview  risk medium https://covers.openlibrary.org/b/isbn/9782070360024-M.jpg?default=false
    open in a browser to preview
 ...
Rights basis of every candidate: none. They help you find the cover; none may be printed as found.
To save an image you hold the rights to: --rights-basis <basis> -o FILE.
```

By default `cover` makes **no request**: it lists where the cover can be found and
previewed. `--probe` runs only the fact requests (is there a cover, how large).
Downloading and saving happens only in **operator mode**, with an explicit rights basis:

```console
$ ebc cover 9782070360024 --rights-basis print_licence -o cover.jpg --min-ppi 250 --format poche
notice: operator mode: you are responsible for holding the rights (`print_licence`) to any image this command saves.
...
```

`robots.txt` (including `Crawl-delay`) and per-host delays are honoured on every request;
placeholder images ("no cover available") are detected and never saved; `--min-ppi`
refuses an image too small for the front panel of `--format`. `--json` reports every
candidate tried with its outcome (skipped by robots.txt, placeholder, too small, saved, …)
and, for a saved image, its effective resolution and print-quality verdict.

### The card

```sh
# typographic card (no cover image), PDF/X-4 for a print portal
ebc card --title "Les Vagues du soir" --author "Jeanne Roy" \
    --random-code --base-url https://ebk.example \
    --giver Alice --recipient Bob --message "Joyeux anniversaire !" \
    --locale fr --format poche --profile upload -o card.pdf

# with a cover you hold the rights to, crop and fold marks, previews
ebc card --title "Les Vagues du soir" --author "Jeanne Roy" --code BCDF-GHJK-LMNP \
    --cover cover.jpg --cover-rights rights_holder_declared \
    --base-url https://ebk.example --profile pro --preview-svg previews/ -o card.pdf

# home printing: a plain PDF on an A4 sheet, with a duplex hint and cut/fold guides
ebc card --title "Candide" --author Voltaire --random-code \
    --base-url https://ebk.example --profile home-a4 -o candide-a4.pdf

# a given output intent for a PDF/X file
ebc card … --profile pro --icc ISOcoated_v2_300_bas.icc --output-condition FOGRA39 -o card.pdf
```

- `--cover FILE` requires `--cover-rights <basis>` (`print_licence`,
  `rights_holder_declared`, `public_domain`; `promo_all_media` and `user_private_copy`
  only on home profiles). Without one the command stops and explains; `--typographic`
  composes the card without the image instead.
- Formats: `poche`, `a6`, `digest`, `a5`, or `WIDTHxHEIGHT[xSPINE]` in millimetres.
  Profiles: `upload`, `pro`, `home-a4`, `home-letter`, `home-a3`, `home-sra3` (or any
  `ebc-layout` profile name).
- The gift code is a bearer secret: it is echoed only when `--random-code` made it.
- `--created-unix N` (or `SOURCE_DATE_EPOCH`) makes the PDF byte-identical across runs.
- `--json` prints a summary: file, size, profile, print route, warnings.

### Preflight

```console
$ ebc preflight card.pdf --profile upload --format poche
card.pdf: PASS — 0 error(s), 0 warning(s), 2 page(s), profile upload_eu
$ ebc preflight card.pdf --profile pro --json > report.json; echo $?
0
```

Exit status 0 when the file passes, 2 when it has errors (each listed with its code and
page). `--generic` checks the profile only (no trim size, spine or QR-module
expectation); `--allow-rgb` accepts the tagged-sRGB cover image of a PDF/X-4 card.

### QR codes

```sh
ebc qr "https://ebk.example/c/BCDFGHJKLMNP" -o code.svg --ec M
ebc qr "https://ebk.example/c/BCDFGHJKLMNP" --compact-url -o - > code.svg
```

`--compact-url` upper-cases scheme and host so the URL fits the alphanumeric mode (a
smaller symbol); the code then decodes to that equivalent URL. The payload is never
echoed to the terminal.

### Font identification

```sh
ebc fontid cover.jpg --title "Les Vagues du soir" --top 5
ebc fontid cover.jpg --title "Les Vagues du soir" --region 40,60,520,120 --json
ebc fontid cover.jpg --title "…" --index my-index.bin --fonts ./fonts   # re-rank by rendering
```

The answer is the nearest font *of the index* (openly licensed fonts), never a claim
about the font actually used on the cover. Exit 3 when no lettering is found.

## Cargo features

| Feature  | Default | Effect                                                              |
|----------|---------|---------------------------------------------------------------------|
| `net`    | on      | real HTTP for `lookup` and `cover`; without it, `--offline-fixtures` only |
| `fontid` | on      | the `fontid` subcommand (pulls in `ebc-fontid` with its bundled index) |

## Tests

`cargo test -p ebc-cli` runs one integration-test binary (`tests/cli/`) that drives the
built `ebc` through `assert_cmd`. Everything is offline: lookups and cover downloads go to
a small HTTP server on the loopback interface started by the test (`--base-url`,
`--no-proxy`) or to `--offline-fixtures`; images are generated in the tests; cards are
round-tripped through `ebc preflight` for every profile; QR codes are decoded back with
`rqrr`. The catalogue fixtures are hand-written to the documented schemas, not recorded
(see `tests/fixtures/README.md`).
