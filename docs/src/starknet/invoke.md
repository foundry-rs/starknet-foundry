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
  --account example_user \
  invoke \
  --url http://127.0.0.1:5050 \
  --contract-address 0x522dc7cbe288037382a02569af5a4169531053d284193623948eac8dd051716 \
  --function "add" \
  --fee-token eth \
  --arguments 'pokemons::model::PokemonData {'\ 
  'name: "Magmar",'\
  'element: pokemons::model::Element::Fire'\ 
  '}'
```

<details>
<summary>Output:</summary>

```shell
command: invoke
transaction_hash: 0x504f830428d0fcf462b4b814e2f67e12dfbcf3dc7847c1e36ba39d3eb7ac313

To see invocation details, visit:
transaction: https://sepolia.starkscan.co/tx/0x504f830428d0fcf462b4b814e2f67e12dfbcf3dc7847c1e36ba39d3eb7ac313
```
</details>
<br>

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
```

<details>
<summary>Output:</summary>

```shell
command: invoke
transaction_hash: 0x7ad0d6e449e33b6581a4bb8df866c0fce3919a5ee05a30840ba521dafee217f

To see invocation details, visit:
transaction: https://starkscan.co/tx/0x7ad0d6e449...
```
</details>
