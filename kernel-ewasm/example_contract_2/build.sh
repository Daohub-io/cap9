@echo OFF

mkdir ./build

rustup target add wasm32-unknown-unknown
REM cargo install pwasm-utils-cli --bin wasm-build --force
cargo install --path ../../cap9-build --bin cap9-build --force

export contract_name=example_contract_2

cargo build --release --target wasm32-unknown-unknown
# We don't need to use cap9 build here as it contains no syscalls
wasm-build --target=wasm32-unknown-unknown ./target $contract_name

cp ./target/*.wasm ./build
cp ./target/json/* ./build
