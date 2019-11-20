#!/bin/bash
set -e
pushd substrate
# Run substrate node
cargo build --release
cargo run --release -- purge-chain --dev -y
cargo run --release -- --dev > ../substrate.txt 2> ../substrate.err
