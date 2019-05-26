rustup target add wasm32-unknown-unknown
cargo install pwasm-utils-cli --bin wasm-build --force --version 0.6.0

cargo build --release --target wasm32-unknown-unknown
wasm-build --target=wasm32-unknown-unknown .\target kernel-ewasm
REM ..\..\wasm-utils\target\debug\wasm-build.exe --target=wasm32-unknown-unknown .\target kernel-ewasm

mkdir .\build
copy .\target\*.wasm .\build
copy .\target\json\* .\build
