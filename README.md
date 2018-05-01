# Beaker Kernel
[![CircleCI](https://circleci.com/gh/Daolab/beakeros.svg?style=svg&circle-token=03f33d77aa4de144ba10274b9e4020ffb82f7c95)](https://circleci.com/gh/Daolab/beakeros)

## Development

### Requirements:
Requires Parity, see: https://wiki.parity.io/Setup
Requires Npm: see: https://github.com/creationix/nvm
Requires Rust: see https://rustup.rs

See (pwasm-tutorial)[https://github.com/paritytech/pwasm-tutorial] for more information.

### Rust
```bash
# Pwasm currently requires nightly
rustup install nightly
# Install Wasm toolchain
rustup target add wasm32-unknown-unknown

# Install wasm-build command from the pwasm-utils crate
cargo install pwasm-utils
```

### Building
```bash
# Compilation requires two steps:
cargo build --release --target wasm32-unknown-unknown
wasm-build --target=wasm32-unknown-unknown ./target beaker_core
```

### Testing
```bash
# Install Test Dependencies
npm install

# Run Seperately
parity --chain wasm-dev-chain.json --jsonrpc-apis=all
# Run Tests
npm test

```

