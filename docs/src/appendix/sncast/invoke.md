# `invoke`
Send an invoke transaction to Starknet.

## `--contract-address, -a <CONTRACT_ADDRESS>`
Required.

The address of the contract being called in hex (prefixed with '0x') or decimal representation.

## `--function, -e <FUNCTION_NAME>`
Required.

The name of the function to call.

## `--calldata, -c <CALLDATA>`
Optional.

Inputs to the function, represented by a list of space-delimited values `0x1 2 0x3`.
Calldata arguments may be either 0x hex or decimal felts.

## `--max-fee, -m <MAX_FEE>`
Optional.

Max fee for the transaction. If not provided, it will be automatically estimated.

## `--nonce, -n <NONCE>`
Optional.

Nonce for transaction. If not provided, nonce will be set automatically.
