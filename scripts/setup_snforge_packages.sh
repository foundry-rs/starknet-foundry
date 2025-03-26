#!/usr/bin/env bash

SNFOUNDRY_ROOT="$(git rev-parse --show-toplevel)"

pushd "$SNFOUNDRY_ROOT" || exit

target/release/snforge new --template cairo-program cairo_program_test
target/release/snforge new --template balance-contract balance_contract_test
target/release/snforge new --template erc20-contract erc20_contract_test

exit 0