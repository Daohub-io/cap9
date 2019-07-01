#!/bin/bash
set -e
set -o pipefail

scripts/build.sh
cargo test
./node_modules/.bin/mocha -r ts-node/register --recursive tests/**.ts tests/**/**.ts
