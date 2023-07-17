# `invoke`
Send an invoke transaction to Starknet.

## `--contract-address, -a <CONTRACT_ADDRESS>`
Required.

The address of the contract being called in hex (prefixed with '0x') or decimal representation.

## `--entry-point-name, -e <FUNCTION_NAME>`
Required.

The name of the function to call.

## `--calldata, -c <CALLDATA>`
Optional.

Inputs to the function, represented by a list of space-delimited values `0x1 2 0x3`.
Calldata arguments may be either hex or decimal felts.

## `--max-fee <MAX_FEE>`
Optional.

Max fee for transaction. If not provided, max fee will be automatically estimated.
