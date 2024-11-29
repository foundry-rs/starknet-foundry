# Performing Multicall

## Overview

Starknet Foundry `sncast` supports executing multiple deployments or calls with the `sncast multicall run` command.

> 📝 **Note**
> `sncast multicall run` executes only one transaction containing all the prepared calls. Which means the fee is paid once.

You need to provide a **path** to a `.toml` file with declarations of desired operations that you want to execute.

You can also compose such config `.toml` file with the `sncast multicall new` command.

For a detailed CLI description, see the [multicall command reference](../appendix/sncast/multicall/multicall.md).

## Example

### `multicall run` Example

Example file:

```toml
[[call]]
call_type = "deploy"
class_hash = "0x076e94149fc55e7ad9c5fe3b9af570970ae2cf51205f8452f39753e9497fe849"
inputs = []
id = "map_contract"
unique = false

[[call]]
call_type = "invoke"
contract_address = "map_contract"
function = "put"
inputs = ["0x123", 234]  # Numbers can be used directly without quotes
```

After running `sncast multicall run --path file.toml --fee-token strk`, a declared contract will be first deployed, and then its function `put` will be invoked.

> 📝 **Note**
> The example above demonstrates the use of the `id` property in a deploy call, which is then referenced as the `contract address` in an invoke call.
Additionally, the `id` can be referenced in the inputs of deploy and invoke calls 🔥

> 💡 **Info**
> Inputs can be either strings (like `"0x123"`) or numbers (like `234`).

> 📝 **Note**
> For numbers larger than 2^63 - 1 (that can't fit into `i64`), use string format (e.g., `"9223372036854775808"`) due to TOML parser limitations.

<!-- TODO: Adjust snippet and check remove ignoring output -->
<!-- { "ignored_output": true } -->
```shell
$ sncast multicall run --path multicall_example.toml --fee-token strk
```

<details>
<summary>Output:</summary>

```shell
command: multicall
transaction_hash: [..]

To see invocation details, visit:
transaction: [..]
```
</details>
<br>

> 💡 **Info**
> Max fee will be automatically computed if `--max-fee <MAX_FEE>` is not passed.

> 💡 **Info**
> You can also choose to pay in Ether by setting `--fee-token` to `eth`.


### `multicall new` Example

You can also generate multicall template with `multicall new` command, specifying output path.
```shell
$ sncast multicall new ./template.toml
```

<details>
<summary>Output:</summary>

```shell
command: multicall new
content: [[call]]
call_type = "deploy"
class_hash = ""
inputs = []
id = ""
unique = false

[[call]]
call_type = "invoke"
contract_address = ""
function = ""
inputs = []

path: ./template.toml
```
</details>
<br>

> ⚠️ **Warning**
> Trying to pass any existing file as an output for `multicall new` will result in error, as the command doesn't overwrite by default.

### `multicall new` With `overwrite` Argument

If there is a file with the same name as provided, it can be overwritten.

```shell
$ sncast multicall new ./template.toml --overwrite
```

<details>
<summary>Output:</summary>

```shell
command: multicall new
content: [[call]]
call_type = "deploy"
class_hash = ""
inputs = []
id = ""
unique = false

[[call]]
call_type = "invoke"
contract_address = ""
function = ""
inputs = []

path: ./template.toml
```
</details>
