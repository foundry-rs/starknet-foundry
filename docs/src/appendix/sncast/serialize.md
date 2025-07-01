# `serialize`
Serialize Cairo expressions into calldata.

## `--arguments`
Required.

Function arguments provided as a comma-separated string of Cairo expressions.
For example: `--arguments '1, 2, MyStruct { x: 1, y: 2 }, MyEnum::Variant'`

For more information on supported expressions and syntax, see [Calldata Transformation](../../starknet/calldata-transformation.md).

## `--class-hash, -c <CLASS-HASH>`
Optional. Required if neither `--contract-address` nor `--abi-file` is passed.

The class hash of the contract class which contains the function, in hex (prefixed with '0x') or decimal representation.

## `--contract-address, -d <CONTRACT_ADDRESS>`
Optional. Required if neither `--class-hash` nor `--abi-file` is passed.

The address of the contract which contains the function, in hex (prefixed with '0x') or decimal representation.

## `--abi-file, <ABI_FILE_PATH>`
Optional. Required if neither `--class-hash` nor `--contract-address` is passed.

Path to the file holding contract ABI.

## `--function, -f <FUNCTION_NAME>`
Required.

The name of the function whose calldata should be serialized.

## `--url, -u <RPC_URL>`
Optional.

Starknet RPC node url address.

Overrides url from `snfoundry.toml`.

## `--network <NETWORK>`
Optional.

Use predefined network with public provider

Possible values: `mainnet`, `sepolia`.
