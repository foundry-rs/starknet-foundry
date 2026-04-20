# Dry Run

`sncast` supports dry-running transaction commands before submitting them to the network. Use `--dry-run` to validate transaction logic and estimate fees without broadcasting the transaction.

Dry run is available for transaction commands such as:

- [`account deploy`](../appendix/sncast/account/deploy.md)
- [`declare`](../appendix/sncast/declare.md)
- [`declare-from`](../appendix/sncast/declare_from.md)
- [`deploy`](../appendix/sncast/deploy.md)
- [`invoke`](../appendix/sncast/invoke.md)
- [`multicall execute`](../appendix/sncast/multicall/execute.md)
- [`multicall run`](../appendix/sncast/multicall/run.md)


## Example

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
'}' \
  --dry-run
```

<details>
<summary>Output:</summary>

```shell
Success: Dry run completed

Overall Fee: [..] Fri (~[..] STRK)
```
</details>
<br>

To include a detailed fee breakdown, add `--detailed`:

```shell
$ sncast invoke \
  --network sepolia \
  --contract-address 0x522dc7cbe288037382a02569af5a4169531053d284193623948eac8dd051716 \
  --function "add" \
  --arguments 'pokemons::model::PokemonData {'\
'name: "Magmar",'\
'element: pokemons::model::Element::Fire'\
'}' \
  --dry-run \
  --detailed
```
