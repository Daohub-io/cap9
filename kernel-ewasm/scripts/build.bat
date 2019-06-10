@echo off

rustup target add wasm32-unknown-unknown
cargo install pwasm-utils-cli --bin wasm-build --version 0.6.0

set contract_name=cap9_kernel

cargo build --examples --target wasm32-unknown-unknown --release --features std

cargo build --release --target wasm32-unknown-unknown --no-default-features --features "panic_with_msg"
cap9-build set-mem --pages 3 .\target\wasm32-unknown-unknown\release\%contract_name%.wasm .\target\wasm32-unknown-unknown\release\%contract_name%.wasm
cargo run --package cap9-build -- set-mem --pages 3 .\target\wasm32-unknown-unknown\release\%contract_name%.wasm .\target\wasm32-unknown-unknown\release\%contract_name%.wasm
wasm-build --target=wasm32-unknown-unknown .\target kernel-ewasm

COPY .\target\wasm32-unknown-unknown\release\examples\*_test.wasm .\target\wasm32-unknown-unknown\release
wasm-build --target=wasm32-unknown-unknown .\target cap9-kernel

wasm-build --target=wasm32-unknown-unknown .\target writer_test
wasm-build --target=wasm32-unknown-unknown .\target entry_test
