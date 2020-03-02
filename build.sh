#!/usr/bin/env bash

set -e

export RUSTFLAGS=""

if ! command -v wasm-pack > /dev/null
then
    echo "Install wasm-pack: https://rustwasm.github.io/wasm-pack/installer/" >&2
    exit 1
fi

if ! command -v rollup > /dev/null
then
    echo "Install rollup: npm install --global rollup" >&2
    exit 1
fi

# Got working with wasm-pack 0.9.1
wasm-pack build --target web

# Got working with rollup v1.32.0
rollup \
    ./main.js \
    --output.format iife \
    --output.file ./pkg/bundle.js
