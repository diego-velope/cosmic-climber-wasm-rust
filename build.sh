#!/usr/bin/env bash
set -euo pipefail

echo "==> Building WASM (release)..."
cargo build --target wasm32-unknown-unknown --release

echo "==> Assembling dist/..."
mkdir -p dist/assets

cp target/wasm32-unknown-unknown/release/cosmic-climber-rs.wasm dist/
cp index.html dist/
cp assets/Astronaut.png dist/assets/
cp assets/Orbitron-Bold.ttf dist/assets/

# Download mq_js_bundle.js if it's missing
if [ ! -f dist/mq_js_bundle.js ]; then
  echo "==> Fetching mq_js_bundle.js..."
  curl -fsSL https://not-fl3.github.io/miniquad-samples/mq_js_bundle.js \
    -o dist/mq_js_bundle.js
fi

echo "✓ Done → dist/"