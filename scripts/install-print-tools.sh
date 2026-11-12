#!/usr/bin/env bash
# Installs the external PDF / barcode tools that the print tests shell out to, then verifies
# that each one is really on PATH.
#
# The tests skip a check when its tool is missing (so they pass on a bare laptop). CI must
# never skip silently, hence the verification at the end: a missing tool fails the job.
#
#   scripts/install-print-tools.sh            # install with apt-get (Debian/Ubuntu), then verify
#   scripts/install-print-tools.sh --check    # verify only (no sudo, no network)
#
# Used by .github/workflows/ci.yml (job `test`) and .github/workflows/claude-review.yml.
# This file is the ONE place that lists the packages.
set -euo pipefail

# apt package        -> commands the tests use
PACKAGES=(
    poppler-utils   # pdfinfo, pdffonts, pdfimages, pdftoppm, pdftotext
    qpdf            # qpdf --check
    ghostscript     # gs (independent PDF interpreter)
    zbar-tools      # zbarimg (decodes the QR code from the rendered page)
    mupdf-tools     # mutool (second renderer)
)
COMMANDS=(pdfinfo pdffonts pdfimages pdftoppm pdftotext qpdf gs zbarimg mutool)

if [[ "${1:-}" != "--check" ]]; then
    if ! command -v apt-get >/dev/null 2>&1; then
        echo "error: apt-get not found; install these packages by hand: ${PACKAGES[*]}" >&2
        exit 1
    fi
    SUDO=()
    if [[ "$(id -u)" -ne 0 ]]; then
        SUDO=(sudo)
    fi
    export DEBIAN_FRONTEND=noninteractive
    "${SUDO[@]}" apt-get update -qq
    "${SUDO[@]}" apt-get install -y -qq --no-install-recommends "${PACKAGES[@]}"
fi

missing=0
for tool in "${COMMANDS[@]}"; do
    if path="$(command -v "$tool")"; then
        printf '  ok       %-10s %s\n' "$tool" "$path"
    else
        printf '  MISSING  %s\n' "$tool" >&2
        missing=1
    fi
done
if [[ "$missing" -ne 0 ]]; then
    echo "error: some print tools are missing; the print tests would skip their checks." >&2
    exit 1
fi
