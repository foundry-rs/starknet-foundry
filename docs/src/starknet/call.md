# Calling Contracts

## Overview

Starknet Foundry cast supports calling smart contracts on a given network with the `cast call` command.

The basic inputs that you need for this command are:

- Contract address
- Function name
- Inputs to the function

For a detailed CLI description, see the [call command reference](../appendix/cast/call.md).

## Usage example

### With [profiles](../projects/configuration.md#Cast)

```shell
$ cast call \
  --profile testnet \
  --contract-address 0x4a739ab73aa3cac01f9da5d55f49fb67baee4919224454a2e3f85b16462a911 \
  --function some_function \
  --calldata 1 2 0x1e
  
command: Call
response: [FieldElement { inner: 0x0000000000000000000000000000000000000000000000000000000000000000 }]
```

### Without profiles

```shell
$ cast \
  --rpc_url http://127.0.0.1:5050 \
  --network testnet \
  call \
  --contract-address 0x4a739ab73aa3cac01f9da5d55f49fb67baee4919224454a2e3f85b16462a911 \
  --function "some_function" \
  --calldata 1 2 3

command: Call
response: [FieldElement { inner: 0x0000000000000000000000000000000000000000000000000000000000000000 }]
```

> 📝 **Note**
> Call does not require passing account-connected parameters (`account` and `accounts-file`) because it doesn't create a transaction.
