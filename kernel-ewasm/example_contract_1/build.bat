@echo OFF

rustup target add wasm32-unknown-unknown
REM cargo install pwasm-utils-cli --bin wasm-build --force

cargo build --release --target wasm32-unknown-unknown
wasm-build --target=wasm32-unknown-unknown .\target pwasm_token_contract

mkdir .\build
copy .\target\*.wasm .\build
copy .\target\json\* .\build
