
cargo build --release --target wasm32-unknown-unknown
cargo build --examples --target wasm32-unknown-unknown --release --features std

COPY ./target/wasm32-unknown-unknown/release/examples/*_test.wasm ./target/wasm32-unknown-unknown/release
wasm-build --target=wasm32-unknown-unknown ./target cap9-kernel

wasm-build --target=wasm32-unknown-unknown ./target writer_test
wasm-build --target=wasm32-unknown-unknown ./target entry_test
