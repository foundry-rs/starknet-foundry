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
inputs = ["0x123", "234"]
```

After running `sncast multicall run --path file.toml --fee-token strk`, a declared contract will be first deployed, and then its function `put` will be invoked.

> 📝 **Note**
> The example above demonstrates the use of the `id` property in a deploy call, which is then referenced as the `contract address` in an invoke call.
Additionally, the `id` can be referenced in the inputs of deploy and invoke calls 🔥

```shell
$ sncast multicall run --path /Users/john/Desktop/multicall_example.toml --fee-token strk

command: multicall
transaction_hash: 0x38fb8a0432f71bf2dae746a1b4f159a75a862e253002b48599c9611fa271dcb

To see invocation details, visit:
transaction: https://starkscan.co/tx/0x38fb8a0432...
```

> 💡 **Info**
> Max fee will be automatically computed if `--max-fee <MAX_FEE>` is not passed.

> 💡 **Info**
> You can also choose to pay in Ether by setting `--fee-token` to `eth`.


### `multicall new` Example

You can also generate multicall template with `multicall new` command, specifying output path.
```shell
$ sncast multicall new ./template.toml

Multicall template successfully saved in ./template.toml
```

Resulting in output:
```toml
[[call]]
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
```

> ⚠️ **Warning**
> Trying to pass any existing file as an output for `multicall new` will result in error, as the command doesn't overwrite by default.

### `multicall new` With `overwrite` Argument

If there is a file with the same name as provided, it can be overwritten.

```shell
$ sncast multicall new ./template.toml --overwrite

Multicall template successfully saved in ./new_multicall_template.toml
```
