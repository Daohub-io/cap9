First make sure that you've built the substrate node in this directory and have
it running in dev mode. You made need to purge any previous chain data. It's
also wise to split the `stdout` and `stderr`, as the output of our tests will be
on `stdout` and we want to split that from the general node logging. All of this
can be done via the `run` scripts in this directory.

Linux:

```bash
./run.sh
```

Windows:

```cmd
run
```

Both of these files do something along the lines of:

```bash
cargo build --release
cargo run --release -- purge-chain --dev -y
cargo run --release -- --dev > ../substrate.txt 2> ../substrate.err
```

Once this is running, the tests can be run in the `test-harness` directory by
simply executing `cargo run`. This will result in the test output being logged
in substrate.txt.
