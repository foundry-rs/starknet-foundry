#!/bin/bash

CAST_BINARY="$1"
ACCOUNT_FILE_PATH="$2"
URL="$3"
DATA_TRANSFORMER_CONTRACT_ADDRESS_SEPOLIA="$4"

$CAST_BINARY \
  --accounts-file \
  "$ACCOUNT_FILE_PATH" \
  --account \
  user12 \
  --int-format \
  --json \
  call \
  --url \
  "$URL" \
  --contract-address \
  "$DATA_TRANSFORMER_CONTRACT_ADDRESS_SEPOLIA" \
  --function \
  nested_struct_fn \
  --arguments \
  'NestedStructWithField { a: SimpleStruct { a: 0x24 }, b: 96 }' \

$CAST_BINARY \
  --accounts-file \
  "$ACCOUNT_FILE_PATH" \
  --account \
  user12 \
  --int-format \
  --json \
  call \
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
