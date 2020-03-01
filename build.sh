#!/usr/bin/env bash

set -e

export RUSTFLAGS=""

# Got working with wasm-pack 0.9.1
wasm-pack build --target web

#wasm-opt to reduce size?

# Got working with rollup v1.32.0
rollup \
    ./main.js \
    --output.format iife \
    --output.file ./pkg/bundle.js
