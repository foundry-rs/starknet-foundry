# `declare`
Send a declare transaction of Cairo contract to Starknet.

## `--contract-name, -c <CONTRACT_NAME>`
Required.

Name of the contract. Contract name is a part after the mod keyword in your contract file.

## `--max-fee, -m <MAX_FEE>`
Optional.

Max fee for transaction. If not provided, max fee will be automatically estimated.

## `--nonce, -n <NONCE>`
Optional.

Nonce for transaction. If not provided, nonce will be set automatically.
