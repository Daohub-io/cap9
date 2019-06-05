@echo OFF

mkdir .\build

rustup target add wasm32-unknown-unknown
REM cargo install pwasm-utils-cli --bin wasm-build --force
cargo install --path ..\..\cap9-build --bin cap9-build --force

set contract_name=example_contract_1

cargo build --release --target wasm32-unknown-unknown
REM TODO: this replaces the file in-place, but needs to be done to cooperate with wasm-build
cap9-build build-proc .\target\wasm32-unknown-unknown\release\%contract_name%.wasm .\target\wasm32-unknown-unknown\release\%contract_name%.wasm
wasm-build --target=wasm32-unknown-unknown .\target %contract_name%

REM copy .\target\*.wasm .\build
REM copy .\target\json\* .\build
