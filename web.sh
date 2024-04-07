#!/bin/sh

set -e

cargo build --target wasm32-unknown-unknown $*
mkdir -p generated
wasm-bindgen target/wasm32-unknown-unknown/debug/curved-space.wasm --out-dir generated --target web
cp index.html generated
