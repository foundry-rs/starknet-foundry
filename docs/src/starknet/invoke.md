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

<!-- TODO: Add pokemon contract or modify example so the output will be ckeched -->
<!-- { "ignored_output": true } -->
```shell
$ sncast \
  --account user0 \
  invoke \
  --url http://127.0.0.1:5055 \
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
transaction_hash: [..]

To see invocation details, visit:
transaction: https://sepolia.starkscan.co/tx/[..]
```
</details>
<br>

> 💡 **Info**
> Max fee will be automatically computed if `--max-fee <MAX_FEE>` is not passed.

> 💡 **Info**
> You can also choose to pay in Ether by setting `--fee-token` to `eth`.

### Invoking Function Without Arguments

Not every function accepts parameters. Here is how to call it.

<!-- { "contract_name": "HelloSncast" } -->
```shell
$ sncast invoke \
  --fee-token strk \
  --contract-address 0x0555d84fd95ab9fa84a826382ca91127336d4b3c640d8571c32c4e7717e38799 \
  --function "get_balance"
```

<details>
<summary>Output:</summary>

```shell
command: invoke
transaction_hash: [..]

To see invocation details, visit:
transaction: https://sepolia.starkscan.co/tx/[..]
```
</details>
