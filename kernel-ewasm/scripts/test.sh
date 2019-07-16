#!/bin/bash
set -e
set -o pipefail

# Use the flag -p to run parallel tests using the "mocha-parallel-tests"
# package.

MOCHA_CMD=mocha

if [ "$1" = "-p" ]; then
    MOCHA_CMD=mocha-parallel-tests
fi

scripts/build.sh
cargo test
./node_modules/.bin/${MOCHA_CMD} -r ts-node/register --recursive tests/**.ts tests/**/**.ts
