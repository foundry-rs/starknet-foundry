# Performing Multicall

## Overview

Starknet Foundry cast supports executing multiple deployments or calls with the `sncast multicall run` command.

> 📝 **Note**
> `sncast multicall run` executes only one transaction containing all the prepared calls. Which means the fee is paid once.

You need to provide a **path** to a `.toml` file with declarations of desired operations that you want to execute.

You can also compose such config `.toml` file with the `sncast multicall new` command.

For a detailed CLI description, see the [multicall command reference](../appendix/cast/multicall/multicall.md).

## Usage examples

### `run` example

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

After running `sncast multicall run --path file.toml`, a declared contract will be first deployed, and then its function `put` will be invoked.

> 📝 **Note**
> The example above demonstrates the use of the `id` property in a deploy call, which is then referenced as the `contract address` in an invoke call 🔥

```shell
$ sncast multicall run --path /Users/john/Desktop/multicall_example.toml

command: multicall
transaction_hash: 0x38fb8a0432f71bf2dae746a1b4f159a75a862e253002b48599c9611fa271dcb
```

### `new` example

```shell
$ sncast multicall new

[[call]]
call_type = ""
class_hash = ""
inputs = []
id = ""
unique = false

[[call]]
call_type = ""
contract_address = ""
function = ""
inputs = []
```
