# `call`
Call a smart contract on Starknet with the given parameters.

## `--contract-address, -a <CONTRACT_ADDRESS>`
Required.

The address of the contract being called in hex (prefixed with '0x') or decimal representation.

## `--function, -f <FUNCTION_NAME>`
Required.

The name of the function being called.

## `--url, -u <RPC_URL>`
Optional.

Starknet RPC node url address.

Overrides url from `snfoundry.toml`.

## `--network <NETWORK>`
Optional.

Use predefined network with public provider

Possible values: `mainnet`, `sepolia`.

## `--calldata, -c <CALLDATA>`
Optional.

Inputs to the function, represented by a list of space-delimited values, e.g. `0x1 2 0x3`.
Calldata arguments may be either 0x hex or decimal felts.

## `--arguments`
Optional.

Function arguments provided as a comma-separated string of Cairo expressions. This provides a more human-readable way to pass arguments compared to raw calldata.
For example: `--arguments '1, 2, MyStruct { x: 1, y: 2 }, MyEnum::Variant'`

## `--block-id, -b <BLOCK_ID>`
Optional.

Block identifier on which call should be performed.
Possible values: `pending`, `latest`, block hash (0x prefixed string), and block number (u64).
`pending` is used as a default value.
