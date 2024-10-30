#!/bin/bash

CAST_BINARY="$1"
ACCOUNT_FILE_PATH="$2"
URL="$3"
CONSTRUCTOR_WITH_PARAMS_CONTRACT_CLASS_HASH_SEPOLIA="$4"

$CAST_BINARY \
  --accounts-file \
  "$ACCOUNT_FILE_PATH" \
  --account \
  user5 \
  --int-format \
  --json \
  deploy \
  --url \
  "$URL" \
  --class-hash \
  "$CONSTRUCTOR_WITH_PARAMS_CONTRACT_CLASS_HASH_SEPOLIA" \
  --arguments \
  '0x420, 0x2137_u256' \
  --max-fee \
  99999999999999999 \
  --fee-token \
  eth
