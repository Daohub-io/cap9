#!/bin/bash

cargo build --release --target wasm32-unknown-unknown
wasm-build --target=wasm32-unknown-unknown ./target kernel-ewasm

rm -dr ./build
mkdir -p ./build

cp ./target/*.wasm ./build
cp ./target/json/* ./build