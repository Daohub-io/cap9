..\parity-ethereum\target\debug\parity  --config dev --chain ./kernel-ewasm/wasm-dev-chain.json db kill
..\parity-ethereum\target\debug\parity  --config dev --chain ./kernel-ewasm/wasm-dev-chain.json --jsonrpc-apis=all --ws-apis=all --reseal-min-period 0
