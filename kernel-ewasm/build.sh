#!/bin/bash

rustup target add wasm32-unknown-unknown
cargo install pwasm-utils-cli --bin wasm-build --version 0.6.0
cargo install --path ../cap9-build --bin cap9-build --force

cargo build --release --target wasm32-unknown-unknown --no-default-features --features "panic_with_msg"
cap9-build set-mem --pages 3 ./target/wasm32-unknown-unknown/release/kernel_ewasm.wasm ./target/wasm32-unknown-unknown/release/kernel_ewasm.wasm
wasm-build --target=wasm32-unknown-unknown ./target kernel-ewasm

rm -dr ./build
mkdir -p ./build

cp ./target/*.wasm ./build
cp ./target/json/* ./build
