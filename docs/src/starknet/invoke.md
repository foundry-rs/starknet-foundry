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

<!-- TODO(#2736) -->
<!-- { "ignored_output": true } -->
```shell
$ sncast \
  --account my_account \
  invoke \
  --network sepolia \
  --contract-address 0x522dc7cbe288037382a02569af5a4169531053d284193623948eac8dd051716 \
  --function "add" \
  --arguments 'pokemons::model::PokemonData {'\
'name: "Magmar",'\
'element: pokemons::model::Element::Fire'\
'}'
```

<details>
<summary>Output:</summary>

```shell
Success: Invoke completed

Transaction Hash: [..]

To see invocation details, visit:
transaction: https://sepolia.voyager.online/tx/[..]
```
</details>
<br>

> 💡 **Info**
> Transaction fee limit can be set either as a single upper bound by `--max-fee` or broken down
> into individual resource components using `--l1-gas`, `--l1-gas-price`, `--l2-gas`,
> `--l2-gas-price`, `--l1-data-gas`, and `--l1-data-gas-price`.
> `--max-fee` and the individual resource flags are mutually exclusive.
> Any individual resource flag that is not provided will be estimated automatically

### Invoking Function Without Arguments

Not every function accepts parameters. Here is how to call it.

```shell
$ sncast invoke \
  --contract-address 0x0589a8b8bf819b7820cb699ea1f6c409bc012c9b9160106ddc3dacd6a89653cf \
  --function "get_balance"
```

<details>
<summary>Output:</summary>

```shell
Success: Invoke completed

Transaction Hash: [..]

To see invocation details, visit:
transaction: https://sepolia.voyager.online/tx/[..]
```
</details>
