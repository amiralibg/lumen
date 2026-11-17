#!/usr/bin/env bash
# Build the website into site/: compiles lumen-demos to wasm for the live demos.
# Requires: rustup target add wasm32-unknown-unknown
#           cargo install wasm-bindgen-cli --version 0.2.130
set -euo pipefail
cd "$(dirname "$0")/.."
PROFILE="${PROFILE:-release-web}"
cargo build -p lumen-demos --target wasm32-unknown-unknown --profile "$PROFILE"
wasm-bindgen --target web --no-typescript --out-dir site/demo \
  "target/wasm32-unknown-unknown/$PROFILE/lumen-demos.wasm"
if command -v wasm-opt >/dev/null; then
  wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int -o site/demo/lumen-demos_bg.wasm site/demo/lumen-demos_bg.wasm
fi
ls -lh site/demo
echo "Serve with: python3 -m http.server -d site 8080"
