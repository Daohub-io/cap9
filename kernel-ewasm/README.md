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
cargo install pwasm-utils-cli --bin wasm-build --version 0.6.0

# We need to install all npm modules - use stable (v11)(2019)
npm install
```

### Build

```bash
# Enter Dir
cd kernel-ewasm
# Compile lib to ewasm then as pwasm contract
./build.sh
```

### Test

#### Unit Tests

```bash
# Enter Dir
cd kernel-ewasm

# For Unit Tests:
# Do cargo test with "std" feature
cargo test --package cap9-kernel --features std
```

#### Integration Tests

```bash
# Reset Parity Dev Chain State
parity  --config dev --chain ./wasm-dev-chain.json db kill

# Run Parity Dev Chain in seperate shell
parity  --config dev --chain ./wasm-dev-chain.json --jsonrpc-apis=all --ws-apis=all --reseal-min-period 0 --gasprice 0 --geth

# Setup Test Account (if not created)
curl --data '{"jsonrpc":"2.0","method":"parity_newAccountFromPhrase","params":["user", "user"],"id":0}' -H "Content-Type: application/json" -X POST localhost:8545

# Run Npm
npm test

```
