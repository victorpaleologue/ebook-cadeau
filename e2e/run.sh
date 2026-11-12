#!/usr/bin/env bash
# Builds what the browser end-to-end tests need, then runs them (see e2e/README.md).
#
#   e2e/run.sh                         # build, then every scenario
#   e2e/run.sh tests/septante.test.mjs # build, then one file (paths relative to e2e/)
#
# Builds (debug profile, like the Rust journeys):
#   ebc-demo   (demo/integration)  the constellation: server, two stores, printer, mail sink
#   ebc-server (crates/ebc-server) serves the card designer at /app/ (see README: ebc-demo
#                                  does not read EBC_WEB_DIR yet)
#   ebc        (crates/ebc-cli)    `ebc preflight` on the designer's PDF
#   web/pkg                        scripts/build-wasm.sh (profile wasm-release)
#
# Environment:
#   EBC_E2E_SKIP_BUILD=1     use the binaries and web/pkg as they are
#   EBC_E2E_REQUIRE_TOOLS=1  a missing pdftoppm/zbarimg/pdftotext fails instead of skipping
#   EBC_E2E_ARTIFACTS=DIR    screenshots, PDFs and EPUBs (default e2e/artifacts, cleared first)
#   CARGO_TARGET_DIR         honoured
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."
started=$SECONDS

if [[ "${EBC_E2E_SKIP_BUILD:-0}" != "1" ]]; then
    echo "+ cargo build -p ebc-demo-integration -p ebc-server -p ebc-cli --bins" >&2
    cargo build -p ebc-demo-integration -p ebc-server -p ebc-cli --bins
    echo "+ scripts/build-wasm.sh" >&2
    bash scripts/build-wasm.sh
fi

for tool in pdftoppm zbarimg pdftotext; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "note: $tool is missing: the card's QR code / text checks will be skipped (scripts/install-print-tools.sh)" >&2
    fi
done

artifacts="${EBC_E2E_ARTIFACTS:-e2e/artifacts}"
rm -rf "$artifacts"
mkdir -p "$artifacts"
EBC_E2E_ARTIFACTS="$(cd "$artifacts" && pwd)"
export EBC_E2E_ARTIFACTS

cd e2e
if [[ $# -eq 0 ]]; then
    set -- tests/*.test.mjs
fi
# Each file starts its own constellation on ports chosen by the system: files run in
# parallel, the scenarios inside a file in order.
status=0
node --test --test-reporter=spec --test-concurrency=4 --test-timeout=180000 "$@" || status=$?
echo "e2e: $((SECONDS - started)) s in total; artifacts in ${EBC_E2E_ARTIFACTS}" >&2
exit $status
