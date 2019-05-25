@echo OFF

mkdir .\build

rustup target add wasm32-unknown-unknown
REM cargo install pwasm-utils-cli --bin wasm-build --force
cargo install --path ..\..\cap9-build --bin cap9-build --force

set contract_name=example_contract_1

cargo build --release --target wasm32-unknown-unknown
wasm-build --target=wasm32-unknown-unknown .\target %contract_name% --public-api dummy_syscall
cap9-build .\target\%contract_name%.wasm .\build\%contract_name%.wasm

REM copy .\target\*.wasm .\build
REM copy .\target\json\* .\build
