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

## `--calldata, -c <CALLDATA>`
Optional.

Inputs to the function, represented by a Cairo-like expression.  Should be in format `"{ arguments }"`.
Supported argument types:

| Argument type                       | Valid expressions                                                  |
|-------------------------------------|--------------------------------------------------------------------|
| numerical value (felt, u8, i8 etc.) | `0x1`, `2_u8`, `-3`                                                |
| shortstring                         | `'value'`                                                          |
| string (ByteArray)                  | `"value"`                                                          |
| boolean value                       | `true`, `false`                                                    |
| struct                              | `Struct { field_one: 0x1 }`, `path::to::Struct { field_one: 0x1 }` |
| enum                                | `Enum::One`, `Enum::Two(123)`, `path::to::Enum::Three`             |
| array                               | `array![0x1, 0x2, 0x3]`                                            |
| tuple                               | `(0x1, array![2], Struct { field: 'three' })`                      |

## `--block-id, -b <BLOCK_ID>`
Optional.

Block identifier on which call should be performed.
Possible values: `pending`, `latest`, block hash (0x prefixed string), and block number (u64).
`pending` is used as a default value.
