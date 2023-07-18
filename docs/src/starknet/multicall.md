# Calling Contracts

## Overview

Starknet Foundry cast supports executing multiple deployments or calls with the `cast multicall` command.

You need to provide a **path** to a `.toml` file with declarations of desired operations that you want to execute.

For a detailed CLI description, see the [multicall command reference](../appendix/cast/multicall.md).

## Usage example

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
contract_address = "0x38b7b9507ccf73d79cb42c2cc4e58cf3af1248f342112879bfdf5aa4f606cc9"
function = "put"
inputs = ["0x123", "234"]
```

After running `multicall --path file.toml`, a declared contract will be first deployed, and then its function `put` will be invoked.

```shell
$ cast multicall --path /Users/john/Desktop/multicall_example.toml

command: Deploy
contract_address: 0x67354442e6cfecdbc0b06a6f55f330e73597f72d461bec7191093b9be78a2b6
transaction_hash: 0x38fb8a0432f71bf2dae746a1b4f159a75a862e253002b48599c9611fa271dcb
command: Invoke
transaction_hash: 0x14cb324d948cd826e708de6e71808b6e483e86945752dd70ae99fe6b16f2905
```
