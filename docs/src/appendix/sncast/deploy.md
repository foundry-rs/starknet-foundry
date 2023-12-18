# `deploy`
Deploy a contract to Starknet.

## `--class-hash, -g <CLASS_HASH>`
Required.

Class hash of contract to deploy.

## `--constructor-calldata, -c <CONSTRUCTOR_CALLDATA>`
Optional.

Calldata for the contract constructor.

## `--salt, -s <SALT>`
Optional.

Salt for the contract address.

## `--unique, -u`
Optional.

If passed, the salt will be additionally modified with an account address.

## `--max-fee, -m <MAX_FEE>`
Optional.

Max fee for the transaction. If not provided, max fee will be automatically estimated.

## `--nonce, -n <NONCE>`
Optional.

Nonce for transaction. If not provided, nonce will be set automatically.
