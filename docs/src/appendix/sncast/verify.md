# `verify`
Verify Cairo contract on a chosen verification provider.

## `--contract-address, -a <CONTRACT_ADDRESS>`
Required.
Conflicts with: `--class-hash`

The address of the contract that is to be verified.

## `--class-hash, -c <CLASS_HASH>`
Required.
Conflicts with: `--contract-address`

The class hash of the contract that is to be verified.

## `--class-name <CLASS NAME>`
Required.

The name of the contract class. The contract name is the part after the `mod` keyword in your contract file.

## `--verifier, -v <VERIFIER>`
Optional.

The verification provider to use for the verification. Possible values are:
* `walnut`
* `voyager`

## `--network, -n <NETWORK>`
Required.

The network on which block explorer will perform the verification. Possible values are:
* `mainnet`
* `sepolia`

## `--package <NAME>`
Optional.

Name of the package that should be used.

If supplied, a contract from this package will be used. Required if more than one package exists in a workspace.

## `--confirm-verification`
Optional.

If passed, assume "yes" as answer to confirmation prompt and run non-interactively.


## `--custom-base-api-url`
Optional.

If supplied, will be used as the base url for the selected verifier.
