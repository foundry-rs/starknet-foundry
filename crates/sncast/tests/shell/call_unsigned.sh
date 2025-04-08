#!/bin/bash

CAST_BINARY="$1"
URL="$2"
DATA_TRANSFORMER_CONTRACT_ADDRESS_SEPOLIA="$3"

$CAST_BINARY \
  --int-format \
  --json \
  call \
  --url \
  "$URL" \
  --contract-address \
  "$DATA_TRANSFORMER_CONTRACT_ADDRESS_SEPOLIA" \
  --function \
  signed_fn_multiple \
  --arguments '-3_i32, -4'
