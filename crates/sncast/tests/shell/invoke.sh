#!/bin/bash

CAST_BINARY="$1"
URL="$2"
DATA_TRANSFORMER_CONTRACT_ADDRESS_SEPOLIA="$3"

$CAST_BINARY \
  --accounts-file \
  accounts.json \
  --account \
  my_account \
  --json \
  invoke \
  --url \
  "$URL" \
  --contract-address \
  "$DATA_TRANSFORMER_CONTRACT_ADDRESS_SEPOLIA" \
  --function \
  complex_fn \
  --arguments 'array![array![1, 2], array![3, 4, 5], array![6]],'\
'12,'\
'-128_i8,'\
'"Some string (a ByteArray)",'\
"('a shortstring', 32_u32),"\
'true,'\
'0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff' \
  --max-fee \
  99999999999999999 \
