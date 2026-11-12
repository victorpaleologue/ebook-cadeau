#!/usr/bin/env bash
# Runs a cargo command on every workspace crate that must compile to WebAssembly.
#
# This file is the ONE place that knows which crates are *not* wasm-compatible. CI
# (.github/workflows/ci.yml, job `wasm`), the docs (docs/ci.md) and humans all go through it.
#
#   scripts/wasm-crates.sh                      # cargo build  … --target wasm32-unknown-unknown
#   scripts/wasm-crates.sh check                # cargo check  …
#   scripts/wasm-crates.sh build --locked       # extra arguments are passed to cargo
#   scripts/wasm-crates.sh clippy -- -D warnings
#   scripts/wasm-crates.sh --print-excludes     # just print the excluded package names
#
# Rule (docs/architecture.md §3): every library crate builds for wasm32-unknown-unknown except
# the server, the CLI and the demos. When you add a host-only crate (sockets, files, threads),
# add its *package name* (the `name` in its Cargo.toml, not its directory) below.
set -euo pipefail

TARGET="wasm32-unknown-unknown"

# Package names of the crates that are host-only.
NOT_WASM=(
    ebc-server        # crates/ebc-server  — axum service
    ebc-cli           # crates/ebc-cli     — command line, file and network I/O
    ebc-demo-stores   # demo/stores        — fictional eBook stores (HTTP servers)
    ebc-demo-printer  # demo/printer       — fictional print-on-demand API (HTTP server)
    ebc-demo-ereaders # demo/ereaders      — simulated e-readers (HTTP clients)
    ebc-demo-integration # demo/integration — end-to-end journeys and the ebc-demo binary (tokio)
)

if [[ "${1:-}" == "--print-excludes" ]]; then
    printf '%s\n' "${NOT_WASM[@]}"
    exit 0
fi

subcommand="${1:-build}"
if [[ $# -gt 0 ]]; then
    shift
fi

args=(--workspace --target "$TARGET")
for package in "${NOT_WASM[@]}"; do
    args+=(--exclude "$package")
done

cd "$(dirname "${BASH_SOURCE[0]}")/.."
echo "+ cargo $subcommand ${args[*]} $*" >&2
exec cargo "$subcommand" "${args[@]}" "$@"
