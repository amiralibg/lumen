#!/usr/bin/env bash
# Build the gallery for the browser into web/dist.
# Requires: rustup target add wasm32-unknown-unknown
#           cargo install wasm-bindgen-cli --version <matching wasm-bindgen>
set -euo pipefail
cd "$(dirname "$0")/.."
PROFILE="${PROFILE:-release-web}"
cargo build -p lumen-gallery --target wasm32-unknown-unknown --profile "$PROFILE"
mkdir -p web/dist
wasm-bindgen --target web --no-typescript --out-dir web/dist \
  "target/wasm32-unknown-unknown/$PROFILE/lumen-gallery.wasm"
cp web/index.html web/dist/
if command -v wasm-opt >/dev/null; then
  wasm-opt -Oz -o web/dist/lumen-gallery_bg.wasm web/dist/lumen-gallery_bg.wasm
fi
ls -lh web/dist
echo "Serve with: python3 -m http.server -d web/dist 8080"
