# Calling Contracts

## Overview

Starknet Foundry cast supports calling smart contracts on a given network with the `cast call` command.

The basic inputs that you need for the commands are:

- Contract address
- Function name
- Inputs to the function

For detailed CLI description, see [call command reference](../reference/cast/index.html#call).

## Usage example

### With profiles

```shell
$ cast call \
  --profile testnet \
  --contract-address 0x4a739ab73aa3cac01f9da5d55f49fb67baee4919224454a2e3f85b16462a911 \
  --function-name some_function \
  --calldata 1 2 3
  
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
  --entry-point-name "some_function" \
  --calldata 1 2 3

command: Call
response: [FieldElement { inner: 0x0000000000000000000000000000000000000000000000000000000000000000 }]
```

> 📝 **Note**
> Call does not require passing account-connected parameters (`account` and `accounts-file`).
