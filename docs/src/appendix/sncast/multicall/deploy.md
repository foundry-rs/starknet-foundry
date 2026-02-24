# `deploy`
Configure a deploy call as part of a multicall transaction.

## `--id <ID>`
Optional.

An optional identifier to reference this step in later steps. This is useful for referencing deployed contracts in later calls within the same multicall. Value can be later reference with `@id` syntax.

## `--class-hash, -g <CLASS_HASH>`
Required.

Class hash of contract to deploy.

## `--constructor-calldata, -c <CONSTRUCTOR_CALLDATA>`
Optional.
Conflicts with: [`--arguments`](#--arguments)

Calldata for the contract constructor.

## `--arguments`
Optional.
Conflicts with: [`--constructor-calldata`](#--constructor-calldata--c-constructor_calldata)

Constructor arguments provided as a comma-separated string of Cairo expressions.
For example: `--arguments '1, 2, MyStruct { x: 1, y: 2 }, MyEnum::Variant'`

For more information on supported expressions and syntax, see [Calldata Transformation](../../../starknet/calldata-transformation.md).

## `--salt, -s <SALT>`
Optional.

Salt for the contract address.

## `--unique`
Optional.

If passed, the salt will be additionally modified with an account address.
