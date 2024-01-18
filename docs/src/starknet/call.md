# Calling Contracts

## Overview

Starknet Foundry `sncast` supports calling smart contracts on a given network with the `sncast call` command.

The basic inputs that you need for this command are:

- Contract address
- Function name
- Inputs to the function

For a detailed CLI description, see the [call command reference](../appendix/sncast/call.md).

## Examples

### General Example

```shell
$ sncast \
  --url http://127.0.0.1:5050 \
  call \
  --contract-address 0x4a739ab73aa3cac01f9da5d55f49fb67baee4919224454a2e3f85b16462a911 \
  --function "some_function" \
  --calldata 1 2 3

command: call
response: [0x1, 0x23, 0x4]
```

> 📝 **Note**
> Call does not require passing account-connected parameters (`account` and `accounts-file`) because it doesn't create a transaction.

### Passing `block-id` Argument

You can call a contract at the specific block by passing `--block-id` argument.

```shell
$ sncast call \
  --contract-address 0x4a739ab73aa3cac01f9da5d55f49fb67baee4919224454a2e3f85b16462a911 \
  --function "some_function" \
  --calldata 1 2 3 \
  --block-id 1234

command: call
response: [0x1, 0x23]
```
