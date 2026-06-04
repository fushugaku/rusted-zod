#!/usr/bin/env bash
set -euo pipefail

cargo build --release --target wasm32-unknown-unknown
mkdir -p web/pkg
wasm-bindgen \
  --target web \
  --out-dir web/pkg \
  target/wasm32-unknown-unknown/release/zod-source-rust.wasm
cp index.html web/index.html
mkdir -p web/assets web/maps
rsync -a --delete assets/ web/assets/
rsync -a --delete maps/ web/maps/
