#!/bin/bash

cargo build --release --target wasm32-unknown-unknown
wasm-build --target=wasm32-unknown-unknown ./target kernel-ewasm

mkdir ./build
cp ./target/*.wasm ./build
cp ./target/json/* ./build