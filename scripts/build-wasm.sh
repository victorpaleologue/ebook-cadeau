#!/usr/bin/env bash
# Builds the browser package of eBook Cadeau into web/pkg/.
#
#   scripts/build-wasm.sh            # release build, bindings, size report, budget check
#   scripts/build-wasm.sh --no-opt   # skip wasm-opt even if it is installed
#
# Steps:
#   1. cargo build -p ebc-wasm --profile wasm-release --target wasm32-unknown-unknown
#   2. wasm-bindgen --target web → web/pkg/ebc_wasm.js + ebc_wasm_bg.wasm (+ .d.ts)
#   3. wasm-opt -Os, when it is on PATH (optional: the profile already optimises for size)
#   4. copies the FOGRA39 output-intent profile next to the package: the page fetches it
#      only for the professional (PDF/X) profiles, so it is not inside the .wasm
#   5. prints the size of the .wasm, raw and gzip'd, and fails above the budget below
#
# Requirements: the wasm32-unknown-unknown target and a wasm-bindgen CLI of exactly the
# version of the `wasm-bindgen` crate in Cargo.lock (the two must match).
#
# Size budget. Measured on 2026-11-12 (rustc 1.9x, no wasm-opt): 4.1 MB raw, 1.9 MB gzip.
# What is inside, roughly: the five bundled OFL fonts (0.5 MB, barely compressible), text
# shaping (harfrust) and font parsing, the JPEG/PNG/WebP decoders, the PDF writer with its
# font subsetter, the thirteen catalogue parsers, the ISBN range table and the font index.
# The budget leaves about 20 % of headroom over that measurement, so that an ordinary
# change passes and a new heavy dependency (or the 1 MB ICC profile slipping back in
# through ebc-card's default features) fails the build:
MAX_RAW_BYTES=$((5 * 1024 * 1024))          # 5.0 MiB
MAX_GZIP_BYTES=$((2400 * 1024))             # 2.4 MiB — what a visitor downloads
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."
OUT_DIR="web/pkg"
TARGET="wasm32-unknown-unknown"
PROFILE="wasm-release"
use_opt=1
for arg in "$@"; do
    case "$arg" in
        --no-opt) use_opt=0 ;;
        *) echo "unknown argument: $arg" >&2; exit 2 ;;
    esac
done

# The CLI and the crate must be the same version, or the bindings do not load.
crate_version="$(awk '/^name = "wasm-bindgen"$/ { getline; gsub(/version = |"/, ""); print; exit }' Cargo.lock)"
if ! command -v wasm-bindgen >/dev/null; then
    echo "wasm-bindgen CLI not found; install version ${crate_version}:" >&2
    echo "  cargo install wasm-bindgen-cli --version ${crate_version} --locked" >&2
    exit 1
fi
cli_version="$(wasm-bindgen --version | awk '{ print $2 }')"
if [[ "$cli_version" != "$crate_version" ]]; then
    echo "wasm-bindgen CLI is ${cli_version} but the crate is ${crate_version}: they must match" >&2
    exit 1
fi

echo "+ cargo build -p ebc-wasm --profile ${PROFILE} --target ${TARGET}" >&2
cargo build -p ebc-wasm --profile "$PROFILE" --target "$TARGET"

target_dir="${CARGO_TARGET_DIR:-target}"
wasm_in="${target_dir}/${TARGET}/${PROFILE}/ebc_wasm.wasm"
mkdir -p "$OUT_DIR"
rm -f "$OUT_DIR"/ebc_wasm*
echo "+ wasm-bindgen --target web --out-dir ${OUT_DIR}" >&2
wasm-bindgen --target web --out-dir "$OUT_DIR" "$wasm_in"
wasm_out="${OUT_DIR}/ebc_wasm_bg.wasm"

if [[ $use_opt -eq 1 ]] && command -v wasm-opt >/dev/null; then
    echo "+ wasm-opt -Os" >&2
    wasm-opt -Os -o "${wasm_out}.opt" "$wasm_out"
    mv "${wasm_out}.opt" "$wasm_out"
else
    echo "wasm-opt not used (not on PATH, or --no-opt)" >&2
fi

# The output intent of the professional profiles, fetched on demand by the page.
cp crates/ebc-card/assets/icc/ISOcoated_v2_300_bas.icc "${OUT_DIR}/fogra39.icc"
cp crates/ebc-card/assets/icc/ISOcoated_v2_300_bas.LICENSE.txt "${OUT_DIR}/fogra39.LICENSE.txt"

raw_bytes=$(wc -c < "$wasm_out")
gzip_bytes=$(gzip -9 -c "$wasm_out" | wc -c)
printf 'ebc_wasm_bg.wasm: %d bytes raw (%.2f MiB), %d bytes gzip -9 (%.2f MiB)\n' \
    "$raw_bytes" "$(echo "$raw_bytes / 1048576" | bc -l)" \
    "$gzip_bytes" "$(echo "$gzip_bytes / 1048576" | bc -l)"

status=0
if (( raw_bytes > MAX_RAW_BYTES )); then
    echo "size budget exceeded: ${raw_bytes} bytes raw > ${MAX_RAW_BYTES}" >&2
    status=1
fi
if (( gzip_bytes > MAX_GZIP_BYTES )); then
    echo "size budget exceeded: ${gzip_bytes} bytes gzip > ${MAX_GZIP_BYTES}" >&2
    status=1
fi
exit $status
