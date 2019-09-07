cargo run --bin cap9-cli -- new --acl new-project
cd new-project
cargo run --bin cap9-cli -- deploy
cargo run --bin cap9-cli -- new-procedure hello_world
cd hello_world
cargo build --target wasm32-unknown-unknown --release
cd ..
cargo run --bin cap9-cli -- deploy-procedure hello_world
cargo run --bin cap9-cli -- fetch all-logs
cargo run --bin cap9-cli -- call-any hello_world say_hello
cargo run --bin cap9-cli -- fetch all-logs
