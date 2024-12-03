#!/bin/bash

CAST_BINARY="$1"
URL="$2"
CONSTRUCTOR_WITH_PARAMS_CONTRACT_CLASS_HASH_SEPOLIA="$3"

$CAST_BINARY \
  --accounts-file \
  accounts.json \
  --account \
  my_account \
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
  strk
