#!/bin/bash

CAST_BINARY="$1"
URL="$2"
DATA_TRANSFORMER_CONTRACT_ADDRESS_SEPOLIA="$3"

$CAST_BINARY \
  --json \
  call \
  --url \
  "$URL" \
  --contract-address \
  "$DATA_TRANSFORMER_CONTRACT_ADDRESS_SEPOLIA" \
  --function \
  multiple_signed_fn \
  --arguments '-3_i32, -4'
