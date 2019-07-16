#!/bin/bash
set -e
set -o pipefail

# Make sure wasm32 is an available compile target for rust.
rustup target add wasm32-unknown-unknown
# Install wasm-build if it is not installed.
if wasm-build --version; then
    echo "wasm-build already installed"
else
    cargo install pwasm-utils-cli --bin wasm-build --version 0.6.0
fi

# Compile all of the examples.
cargo build --examples --target wasm32-unknown-unknown --release --features std

# Compile everything else.
cargo build --release --target wasm32-unknown-unknown --no-default-features --features "panic_with_msg"
pushd cap9-kernel
# Recompile the kernel with panic messages.
cargo build --release --target wasm32-unknown-unknown --no-default-features --features "panic_with_msg"
popd
# Increase the number of memory pages in the kernel.
cargo run --package cap9-build -- set-mem --pages 4 ./target/wasm32-unknown-unknown/release/cap9_kernel.wasm ./target/wasm32-unknown-unknown/release/cap9_kernel.wasm
# Pass the raw WASM output through the wasm-build post-processor.
wasm-build --target=wasm32-unknown-unknown ./target cap9-kernel

# Copy Examples
cp ./target/wasm32-unknown-unknown/release/examples/*.wasm ./target/wasm32-unknown-unknown/release

# Pass example contracts through the procedure build process.
function build_procedure {
    echo "Building $1"
    cargo run --package cap9-build -- build-proc ./target/wasm32-unknown-unknown/release/$1.wasm ./target/wasm32-unknown-unknown/release/$1.wasm
    cargo run --package cap9-build -- set-mem --pages 4 ./target/wasm32-unknown-unknown/release/$1.wasm ./target/wasm32-unknown-unknown/release/$1.wasm
    wasm-build --target=wasm32-unknown-unknown ./target $1
}

build_procedure writer_test
build_procedure entry_test
build_procedure caller_test
build_procedure logger_test
build_procedure register_test
build_procedure logger_test
build_procedure register_test
build_procedure delete_test
build_procedure account_call_test
build_procedure acl_entry
build_procedure acl_admin
build_procedure acl_bootstrap
build_procedure acl_group_5
build_procedure storage_vec_test

# external_contract is just a regular contract and does not need to go through
# the cap9 procedure build process.
wasm-build --target=wasm32-unknown-unknown ./target external_contract
