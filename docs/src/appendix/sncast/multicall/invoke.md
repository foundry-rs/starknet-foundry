# `invoke`
Configure an invoke call as part of a multicall transaction.

## `--contract-address, -d <CONTRACT_ADDRESS>`
Required.

The address of the contract being called in hex (prefixed with '0x') or decimal representation.

## `--function, -f <FUNCTION_NAME>`
Required.

The name of the function to call.

## `--calldata, -c <CALLDATA>`
Optional.
Conflicts with: [`--arguments`](#--arguments)

Inputs to the function, represented by a list of space-delimited values `0x1 2 0x3`.
Calldata arguments may be either 0x hex or decimal felts.

## `--arguments`
Optional.
Conflicts with: [`--calldata`](#--calldata--c-calldata)

Function arguments provided as a comma-separated string of Cairo expressions.
For example: `--arguments '1, 2, MyStruct { x: 1, y: 2 }, MyEnum::Variant'`

For more information on supported expressions and syntax, see [Calldata Transformation](../../../starknet/calldata-transformation.md).
