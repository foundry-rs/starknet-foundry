# `verify`
Verify Cairo contract on a chosen verification provider.

## `--class-hash, -g <CLASS_HASH>`

Optional. Required if `--contract-address` is not provided.

The class hash of the contract that is to be verified.

## `--contract-address, -d <CONTRACT_ADDRESS>`

Optional. Required if `--class-hash` is not provided.

The address of the contract that is to be verified.

## `--contract-name <CONTRACT_NAME>`
Required.

The name of the contract. The contract name is the part after the `mod` keyword in your contract file.

## `--verifier, -v <VERIFIER>`
Required.

The verification provider to use for the verification. Possible values are:
* `walnut`
* `voyager`

## `--network, -n <NETWORK>`
Optional.

The network on which block explorer will perform the verification. Possible values are:
* `mainnet`
* `sepolia`

## `--url <RPC_URL>`
Optional.

Starknet RPC node url address. Will use public provider if not set.

## `--package <NAME>`
Optional.

Name of the package that should be used.

If supplied, a contract from this package will be used. Required if more than one package exists in a workspace.

## `--confirm-verification`
Optional.

If passed, assume "yes" as answer to confirmation prompt and run non-interactively.

## `--test-files`
Optional.

Include test files under `src/` for verification. Only applies to `voyager`.
