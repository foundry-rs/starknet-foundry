#!/bin/sh

set -e

cd -P -- "$(dirname -- "$0")"

./prepare_for_tests.sh
cargo test -p cast
