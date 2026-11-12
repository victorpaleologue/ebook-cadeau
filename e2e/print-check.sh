#!/usr/bin/env bash
# Print check: composes sample cards with the `ebc` CLI for every format x print profile
# (typographic) and for every profile with a cover image, then checks each PDF with
#   ebc preflight   (our own preflight, against the format and profile it was made for)
#   qpdf --check    (structure)
#   pdffonts        (every font embedded)
#   gs              (an independent interpreter renders every page without complaint)
#   pdfinfo -box    (two pages; a bleed box larger than the trim box for professional files)
#
#   e2e/print-check.sh [OUT_DIR]       # default target/print-check
#
# Needs the print tools (scripts/install-print-tools.sh) and builds `ebc` (debug) unless
# EBC_E2E_SKIP_BUILD=1. Output is deterministic (fixed code, fixed creation time).
# Digest and A5 do not fit an A4 or Letter sheet with bleed and margins: `ebc card`
# refuses them, and this script checks that it still does (a silent shrink would print a
# card at the wrong size).
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."
out="${1:-target/print-check}"
mkdir -p "$out"
rm -f "$out"/*.pdf "$out"/*.txt

if [[ "${EBC_E2E_SKIP_BUILD:-0}" != "1" ]]; then
    cargo build -p ebc-cli --bin ebc
fi
ebc="${CARGO_TARGET_DIR:-target}/debug/ebc"
bash scripts/install-print-tools.sh --check

formats=(poche a6 digest a5)
profiles=(upload pro home-a4 home-letter home-a3)
lowres="crates/ebc-wasm/tests/fixtures/golden-cover.png"
common=(--author "Jeanne Roy" --code BCDF-GHJK-LMNL --base-url https://ebk.example
        --giver Alice --recipient Bob --message "Joyeux anniversaire ! Bonne lecture."
        --store "Septante (démo)" --created-unix 1794441600)

failures=0
checked=0
fail() {
    echo "FAIL  $*" >&2
    failures=$((failures + 1))
}

refused_on_sheet() {
    # format, profile → 0 when the card is expected not to fit the sheet
    [[ "$1" == digest || "$1" == a5 ]] && [[ "$2" == home-a4 || "$2" == home-letter ]]
}

check_pdf() {
    local pdf="$1" format="$2" profile="$3" what="$4"
    shift 4
    local report="${pdf%.pdf}.txt"
    : > "$report"
    if ! "$ebc" preflight --format "$format" --profile "$profile" "$@" "$pdf" >> "$report" 2>&1; then
        fail "$what: ebc preflight (see $report)"
    fi
    if ! qpdf --check "$pdf" >> "$report" 2>&1; then
        fail "$what: qpdf --check (see $report)"
    fi
    local fonts
    fonts="$(pdffonts "$pdf" 2>&1)" || fail "$what: pdffonts"
    echo "$fonts" >> "$report"
    # Columns: name type encoding emb sub uni object ID
    if echo "$fonts" | tail -n +3 | grep -Eq '[[:space:]]no[[:space:]]+(yes|no)[[:space:]]+(yes|no)[[:space:]]+[0-9]+[[:space:]]+[0-9]+[[:space:]]*$'; then
        fail "$what: a font is not embedded (see $report)"
    fi
    local gs_out
    if ! gs_out="$(gs -q -dNOPAUSE -dBATCH -dSAFER -sDEVICE=nullpage "$pdf" 2>&1)"; then
        fail "$what: ghostscript failed: $gs_out"
    elif [[ -n "$gs_out" ]]; then
        echo "$gs_out" >> "$report"
        fail "$what: ghostscript complained: $(echo "$gs_out" | head -3)"
    fi
    local info
    info="$(pdfinfo -box "$pdf")"
    echo "$info" >> "$report"
    if ! echo "$info" | grep -Eq '^Pages:[[:space:]]+2$'; then
        fail "$what: the card does not have two pages"
    fi
    if [[ "$profile" == upload || "$profile" == pro ]]; then
        local trim bleed
        trim="$(echo "$info" | awk '/^TrimBox:/ { print $2, $3, $4, $5; exit }')"
        bleed="$(echo "$info" | awk '/^BleedBox:/ { print $2, $3, $4, $5; exit }')"
        if [[ -z "$trim" || "$trim" == "$bleed" ]]; then
            fail "$what: no bleed around the trim box (TrimBox $trim, BleedBox $bleed)"
        fi
    fi
    checked=$((checked + 1))
    echo "ok    $what"
}

for format in "${formats[@]}"; do
    for profile in "${profiles[@]}"; do
        pdf="$out/typographic-$format-$profile.pdf"
        what="typographic $format $profile"
        set +e
        message="$("$ebc" card --title "Les Vagues du soir" --typographic "${common[@]}" \
            --format "$format" --profile "$profile" -o "$pdf" 2>&1)"
        status=$?
        set -e
        if refused_on_sheet "$format" "$profile"; then
            if [[ $status -eq 1 && "$message" == *"cannot be composed"* && ! -e "$pdf" ]]; then
                echo "ok    $what: refused as expected (does not fit the sheet)"
            else
                fail "$what: expected a refusal (does not fit the sheet), got status $status: $message"
            fi
            continue
        fi
        if [[ $status -ne 0 ]]; then
            fail "$what: ebc card exited with $status: $message"
            continue
        fi
        check_pdf "$pdf" "$format" "$profile" "$what"
    done
done

# A cover image with a declared basis, on every profile (poche). The image is drawn here
# with ghostscript at 300 ppi for 100 x 150 mm (no binary fixture of that size exists in
# the repository). A professional (PDF/X-4) card carries the cover converted to the
# output intent's CMYK, so it must pass the profile's own CMYK-only preflight: no
# --allow-rgb any more (product rule changed with the ICC conversion of covers).
cover="$out/cover-300ppi.png"
gs -q -dNOPAUSE -dBATCH -dSAFER -sDEVICE=png16m -r300 -g1181x1772 -o "$cover" -c \
    "0.13 0.32 0.27 setrgbcolor clippath fill 0.93 0.85 0.62 setrgbcolor 4 setlinewidth" \
    "20 20 243.5 385.2 rectstroke /Times-Bold findfont 34 scalefont setfont 40 250 moveto (Les Vagues) show" \
    "/Times-Roman findfont 18 scalefont setfont 40 215 moveto (Jeanne Roy - demo cover) show showpage"
for profile in "${profiles[@]}"; do
    pdf="$out/cover-poche-$profile.pdf"
    what="cover poche $profile"
    if ! message="$("$ebc" card --title "Les Vagues du soir" --cover "$cover" --cover-rights print_licence \
        --cover-credit "Éditions du Lac (démo)" "${common[@]}" --format poche --profile "$profile" -o "$pdf" 2>&1)"; then
        fail "$what: ebc card: $message"
        continue
    fi
    check_pdf "$pdf" poche "$profile" "$what"
done

# The other way round: a 91 ppi cover (the 360 x 540 px fixture of the wasm tests) must be
# refused, or the checks above prove nothing. Since the composer refuses to write a
# professional file below the profile's minimum resolution (product rule, research 03
# §4.2: offer the typographic card instead), `ebc card` refuses it; the same cover on a
# home sheet (where the composer only warns) must still be refused by preflight, whose
# home minimum is 100 ppi.
pdf="$out/lowres-poche-upload.pdf"
set +e
message="$("$ebc" card --title "Les Vagues du soir" --cover "$lowres" --cover-rights print_licence \
    --cover-credit "Éditions du Lac (démo)" "${common[@]}" --format poche --profile upload -o "$pdf" 2>&1)"
status=$?
set -e
if [[ $status -eq 1 && "$message" == *"91 ppi"* && ! -e "$pdf" ]]; then
    echo "ok    a 91 ppi cover is refused for a professional card"
else
    fail "a 91 ppi professional card was written (status $status): $message"
fi
home_pdf="$out/lowres-poche-home-a4.pdf"
"$ebc" card --title "Les Vagues du soir" --cover "$lowres" --cover-rights print_licence \
    --cover-credit "Éditions du Lac (démo)" "${common[@]}" --format poche --profile home-a4 -o "$home_pdf" >/dev/null 2>&1
set +e
verdict="$("$ebc" preflight --generic --profile home-a4 "$home_pdf" 2>&1)"
home_status=$?
set -e
if [[ $home_status -eq 2 && "$verdict" == *IMAGE_LOW_RES* ]]; then
    echo "ok    a 91 ppi cover is refused by preflight at home too (IMAGE_LOW_RES)"
else
    fail "a 91 ppi cover passed preflight (status $home_status): $verdict"
fi
rm -f "$pdf" "$home_pdf"

echo "print-check: $checked PDF(s) checked, $failures failure(s); files in $out"
[[ $failures -eq 0 ]]
