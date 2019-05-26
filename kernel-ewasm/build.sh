#!/bin/bash

rustup target add wasm32-unknown-unknown
cargo install pwasm-utils-cli --bin wasm-build --force --version 0.6.0

cargo build --release --target wasm32-unknown-unknown
wasm-build --target=wasm32-unknown-unknown ./target kernel-ewasm

rm -dr ./build
mkdir -p ./build

cp ./target/*.wasm ./build
cp ./target/json/* ./build
