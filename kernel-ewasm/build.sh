#!/bin/bash

cargo build --release --target wasm32-unknown-unknown
cargo build --examples --target wasm32-unknown-unknown --release --features std

# Copy Examples
cp ./target/wasm32-unknown-unknown/release/examples/*_test.wasm ./target/wasm32-unknown-unknown/release
wasm-build --target=wasm32-unknown-unknown ./target kernel-ewasm

wasm-build --target=wasm32-unknown-unknown ./target writer_test
wasm-build --target=wasm32-unknown-unknown ./target entry_test

rm -dr ./build
mkdir -p ./build
mkdir -p ./build/examples

cp ./target/kernel-ewasm.wasm ./build
cp ./target/*_test.wasm ./build/examples

cp ./target/json/KernelInterface.json ./build
cp ./target/json/Test*.json ./build/examples
