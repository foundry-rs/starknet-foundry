#!/bin/bash

CAST_BINARY="$1"
URL="$2"
CONSTRUCTOR_WITH_PARAMS_CONTRACT_CLASS_HASH_SEPOLIA="$3"

$CAST_BINARY \
  --accounts-file \
  accounts.json \
  --account \
  my_account \
  --json \
  deploy \
  --url \
  "$URL" \
  --class-hash \
  "$CONSTRUCTOR_WITH_PARAMS_CONTRACT_CLASS_HASH_SEPOLIA" \
  --arguments \
  '0x420, 0x2137_u256' \
  --l1-gas \
  100000 \
  --l1-gas-price \
  10000000000000 \
  --l2-gas \
  1000000000 \
  --l2-gas-price \
  100000000000000000000 \
  --l1-data-gas \
  100000 \
  --l1-data-gas-price \
  10000000000000
