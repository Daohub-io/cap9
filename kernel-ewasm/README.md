# Cap9 Ewasm Kernel Target

## Development

### Setup
You will need to install

* [npm](https://nodejs.org/en/)
* [rustup](https://rustup.rs/)
* [parity-ethereum](https://wiki.parity.io/Setup)

```bash
# We need a nightly toolchain
rustup install nightly-2019-03-05
# We need wasm32-unknown-unknown toolchain
rustup target add wasm32-unknown-unknown
# We need to override to the nightly toolchain
rustup override set nightly-2019-03-05

# We need the pwasm-utils for packaging wasm to ewasm
cargo install pwasm-utils-cli --bin wasm-build
```

### Build

```bash
# Compile lib to ewasm then as pwasm contract
./build.sh
```