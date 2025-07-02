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
  call \
  --network sepolia \
  --contract-address 0x522dc7cbe288037382a02569af5a4169531053d284193623948eac8dd051716 \
  --function "balance_of" \
  --arguments '0x0554d15a839f0241ba465bb176d231730c01cf89cdcb95fe896c51d4a6f4bb8f'
```

<details>
<summary>Output:</summary>

```shell
Success: Call completed

Response:     0_u256
Response Raw: [0x0, 0x0]
```
</details>
<br>

> 📝 **Note**
> Call does not require passing account-connected parameters (`account` and `accounts-file`) because it doesn't create a transaction.

### Passing `block-id` Argument

You can call a contract at the specific block by passing `--block-id` argument.

```shell
$ sncast call \
  --network sepolia \
  --contract-address 0x522dc7cbe288037382a02569af5a4169531053d284193623948eac8dd051716 \
  --function "balance_of" \
  --arguments '0x0554d15a839f0241ba465bb176d231730c01cf89cdcb95fe896c51d4a6f4bb8f' \
  --block-id 77864
```

<details>
<summary>Output:</summary>

```shell
Success: Call completed

Response:     0_u256
Response Raw: [0x0, 0x0]
```
</details>
