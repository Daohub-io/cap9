#!/bin/bash
cargo install pwasm-utils-cli --bin wasm-build --version 0.6.0

cargo build --release --target wasm32-unknown-unknown --no-default-features --features "panic_with_msg"
cargo build --examples --target wasm32-unknown-unknown --release --features std

cargo run --package cap9-build -- set-mem --pages 3 ./target/wasm32-unknown-unknown/release/cap9_kernel.wasm ./target/wasm32-unknown-unknown/release/cap9_kernel.wasm

# Copy Examples
cp ./target/wasm32-unknown-unknown/release/examples/*_test.wasm ./target/wasm32-unknown-unknown/release
wasm-build --target=wasm32-unknown-unknown ./target cap9-kernel

wasm-build --target=wasm32-unknown-unknown ./target writer_test
wasm-build --target=wasm32-unknown-unknown ./target entry_test
