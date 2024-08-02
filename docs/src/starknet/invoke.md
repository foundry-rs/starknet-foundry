# Invoking Contracts

## Overview

Starknet Foundry `sncast` supports invoking smart contracts on a given network with the `sncast invoke` command.

In most cases, you have to provide:

- Contract address
- Function name
- Function arguments

For detailed CLI description, see [invoke command reference](../appendix/sncast/invoke.md).

## Examples

### General Example

```shell
$ sncast \
  --url http://127.0.0.1:5050 \
  --account example_user \
  invoke \
  --fee-token strk \
  --contract-address 0x4a739ab73aa3cac01f9da5d55f49fb67baee4919224454a2e3f85b16462a911 \
  --function "some_function" \
  --calldata 1 2 0x1e
  
command: invoke
transaction_hash: 0x7ad0d6e449e33b6581a4bb8df866c0fce3919a5ee05a30840ba521dafee217f
```

> 💡 **Info**
> Max fee will be automatically computed if `--max-fee <MAX_FEE>` is not passed.

> 💡 **Info**
> You can also choose to pay in Ether by setting `--fee-token` to `eth`.

### Invoking Function Without Arguments

Not every function accepts parameters. Here is how to call it.

```shell
$ sncast invoke \
  --fee-token strk \
  --contract-address 0x4a739ab73aa3cac01f9da5d55f49fb67baee4919224454a2e3f85b16462a911 \
  --function "function_without_params"
  
command: invoke
transaction_hash: 0x7ad0d6e449e33b6581a4bb8df866c0fce3919a5ee05a30840ba521dafee217f
```
