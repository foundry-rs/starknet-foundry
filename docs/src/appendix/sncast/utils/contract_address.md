# `contract-address`
Calculate the address of a not yet deployed contract.

## `--class-hash, -g <CLASS_HASH>`
Optional.
Conflicts with: [`--contract-name`](#--contract-name)

The class hash of the contract to deploy, in hex (prefixed with '0x') or decimal representation.

## `--contract-name <CONTRACT_NAME>`
Optional.
Conflicts with: [`--class-hash`](#--class-hash)

The name of the contract. The contract name is the part after the `mod` keyword in your contract file.
The class hash is derived from the locally built artifact.

## `--constructor-calldata <CONSTRUCTOR_CALLDATA>`
Optional.
Conflicts with: [`--arguments`](#--arguments)

Constructor calldata as a series of felts.

## `--arguments <ARGUMENTS>`
Optional.
Conflicts with: [`--constructor-calldata`](#--constructor-calldata)

Constructor arguments as a comma-separated string of Cairo expressions.
For example: `--arguments '1, 2, MyStruct { x: 1, y: 2 }'`

Requires `--url` or `--network` to fetch the contract ABI when used together with `--class-hash`.

For more information on supported expressions and syntax, see [Calldata Transformation](../../../starknet/calldata-transformation.md).

## `--salt, -s <SALT>`
Optional.

Salt for the address. If not provided, a random salt is generated.

## `--unique`
Optional.

If set, the salt is modified with the deployer account address, making the address unique per deployer.
Requires [`--account-address`](#--account-address).

## `--account-address <ACCOUNT_ADDRESS>`
Optional. Required when `--unique` is set.

The deployer account address used to modify the salt when `--unique` is set.
Defaults to zero.

## `--package <PACKAGE>`
Optional.
Conflicts with: [`--class-hash`](#--class-hash)

Specifies the Scarb package to use when looking up the contract by name.

## `--url, -u <RPC_URL>`
Optional.

Starknet RPC node url address. Required when using `--arguments` together with `--class-hash`.

Overrides url from `snfoundry.toml`.

## `--network <NETWORK>`
Optional.

Use predefined network with public provider.

Possible values: `mainnet`, `sepolia`, `devnet`.

Overrides network from `snfoundry.toml`.

