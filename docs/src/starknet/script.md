# Cairo Deployment Scripts

## Overview

> ⚠️⚠️⚠️ Highly experimental code, a subject to change ⚠️⚠️⚠️

Starknet Foundry cast can be used to run deployment scripts written in Cairo, using `script run` subcommand.
It aims to provide similar functionality to Foundry's `forge script`.

To start writing a deployment script in Cairo just add `sncast_std` as a dependency to you scarb package and make sure to
have a `main` function in the module you want to run. `sncast_std` docs can be found [here](../appendix/sncast-library.md).

Please note that **`sncast script` is in development**. While it is already possible to declare, deploy, invoke and call
contracts from within Cairo, its interface, internals and feature set can change rapidly each version.

<!-- TODO(#2490): Update fee usage here -->
> ⚠️⚠️ By default, the nonce for each transaction is being taken from the pending block ⚠️⚠️
>
> Some RPC nodes can be configured with higher poll intervals, which means they may return "older" nonces
> in pending blocks, or even not be able to obtain pending blocks at all. This might be the case if you get
> an error like "Invalid transaction nonce" when running a script, and you may need to manually set both nonce
> and max_fee for transactions.
>
> Example:
>
> ```rust
>      let declare_result = declare(
>        "Map",
>        FeeSettings {
>           max_fee: Option::None,
>           max_gas: Option::Some(999999),
>           max_gas_unit_price: Option::Some(100000000000)
>        },
>        Option::Some(nonce)
>    )
>        .expect('declare failed');
> ```

Some of the planned features that will be included in future versions are:

- dispatchers support
- logging
- account creation/deployment
- multicall support
- dry running the scripts

and more!

## State file

By default, when you run a script a state file containing information about previous runs will be created. This file
can later be used to skip making changes to the network if they were done previously.

To determine if an operation (a function like declare, deploy or invoke) has to be sent to the network, the script will
first check if such operation with given arguments already exists in state file. If it does, and previously ended with
a success, its execution will be skipped. Otherwise, sncast will attempt to execute this function, and will write its status
to the state file afterwards.

To prevent sncast from using the state file, you can set [the --no-state-file flag](../appendix/sncast/script/run.md#--no-state-file).

A state file is typically named in a following manner:

```
{script name}_{network name}_state.json
```

## Suggested directory structures

As sncast scripts are just regular scarb packages, there are multiple ways to incorporate scripts into your existing scarb workspace.
Most common directory structures include:

### 1. `scripts` directory with all the scripts in the same workspace with cairo contracts (default for `sncast script init`)

```shell
$ tree
```

<details>
<summary>Output:</summary>

```shell
.
├── scripts
│   └── my_script
│       ├── Scarb.toml
│       └── src
│           ├── my_script.cairo
│           └── lib.cairo
├── src
│   ├── my_contract.cairo
│   └── lib.cairo
└── Scarb.toml
```
</details>
<br>

> 📝 **Note**
> You should add `scripts` to `members` field in your top-level Scarb.toml to be able to run the script from
> anywhere in the workspace - otherwise you will have to run the script from within its directory. To learn more consult
> [Scarb documentation](https://docs.swmansion.com/scarb/docs/reference/workspaces.html#members).

You can also have multiple scripts as separate packages, or multiple modules inside one package, like so:

#### 1a. multiple scripts in one package

```shell
$ tree
```

<details>
<summary>Output:</summary>

```shell
.
├── scripts
│   └── my_script
│       ├── Scarb.toml
│       └── src
│           ├── my_script1.cairo
│           ├── my_script2.cairo
│           └── lib.cairo
├── src
│   ├── my_contract.cairo
│   └── lib.cairo
└── Scarb.toml
```
</details>
<br>

#### 1b. multiple scripts as separate packages

```shell
$ tree
```

<details>
<summary>Output:</summary>

```shell
.
├── scripts
│   ├── Scarb.toml
│   ├── first_script
│   │   ├── Scarb.toml
│   │   └── src
│   │       ├── first_script.cairo
│   │       └── lib.cairo
│   └── second_script
│       ├── Scarb.toml
│       └── src
│           ├── second_script.cairo
│           └── lib.cairo
├── src
│   ├── my_contract.cairo
│   └── lib.cairo
└── Scarb.toml
```
</details>
<br>

#### 1c. single script with flat directory structure

```shell
$ tree
```

<details>
<summary>Output:</summary>

```shell
.
├── Scarb.toml
├── scripts
│   ├── Scarb.toml
│   └── src
│       ├── my_script.cairo
│       └── lib.cairo
└── src
    └── lib.cairo
```
</details>
<br>

### 2. scripts disjointed from the workspace with cairo contracts

```shell
$ tree
```

<details>
<summary>Output:</summary>

```shell
.
├── Scarb.toml
└── src
    ├── lib.cairo
    └── my_script.cairo
```
</details>
<br>

In order to use this directory structure you must set any contracts you're using as dependencies in script's Scarb.toml,
and override `build-external-contracts` property to build those contracts. To learn more consult [Scarb documentation](https://docs.swmansion.com/scarb/docs/extensions/starknet/contract-target.html#compiling-external-contracts).

This setup can be seen in action in [Full Example below](#full-example-with-contract-deployment).

## Examples

### Initialize a script

To get started, a deployment script with all required elements can be initialized using the following command:

```shell
$ sncast script init my_script
```

For more details, see [init command](../appendix/sncast/script/init.md).

> 📝 **Note**
> To include a newly created script in an existing workspace, it must be manually added to the members list in the `Scarb.toml` file, under the defined workspace.
> For more detailed information about workspaces, please refer to the [Scarb documentation](https://docs.swmansion.com/scarb/docs/reference/workspaces.html).

### Minimal Example (Without Contract Deployment)

This example shows how to call an already deployed contract. Please find full example with contract deployment [here](#full-example-with-contract-deployment).

```rust
{{#include ../../listings/basic_example/src/basic_example.cairo}}
```

The script should be included in a Scarb package. The directory structure and config for this example looks like this:

```shell
$ tree
```

<details>
<summary>Output:</summary>

```shell
.
├── src
│   ├── my_script.cairo
│   └── lib.cairo
└── Scarb.toml
```
</details>
<br>

```toml
[package]
name = "my_script"
version = "0.1.0"

[dependencies]
starknet = ">=2.8.0"
sncast_std = "0.33.0"
```

To run the script, do:

<!-- TODO(#2736) -->
<!-- { "ignored_output": true } -->
```shell
$ sncast \
  script run my_script
  --network sepolia
```

<details>
<summary>Output:</summary>

```shell
CallResult { data: [0, 96231036770510887841935600920878740513, 16] }
command: script run
status: success
```
</details>

### Full Example (With Contract Deployment)

This example script declares, deploys and interacts with an example `MapContract`:

```rust
{{#include ../../listings/map3/src/lib.cairo}}
```

We prepare a script:

```rust
{{#include ../../listings/full_example/src/full_example.cairo}}
```

The script should be included in a Scarb package. The directory structure and config for this example looks like this:

```shell
$ tree
```

<details>
<summary>Output:</summary>

```shell
.
├── contracts
│    ├── Scarb.toml
│    └── src
│        └── lib.cairo
└── scripts
    ├── Scarb.toml
    └── src
        ├── lib.cairo
        └── map_script.cairo
```
</details>
<br>

```toml
[package]
name = "map_script"
version = "0.1.0"

[dependencies]
starknet = ">=2.8.0"
sncast_std = "0.33.0"
map = { path = "../contracts" }

[lib]
sierra = true
casm = true

[[target.starknet-contract]]
build-external-contracts = [
    "map::MapContract"
]
```

Please note that `map` contract was specified as the dependency. In our example, it resides in the filesystem. To generate the artifacts for it that will be accessible from the script you need to use the `build-external-contracts` property.

To run the script, do:

<!-- { "ignored_output": true } -->
```shell
$ sncast \
  --account example_user \
  script run map_script \
  --network sepolia
```

<details>
<summary>Output:</summary>

```shell
Class hash of the declared contract: 685896493695476540388232336434993540241192267040651919145140488413686992233
...
Deployed the contract to address: 2993684914933159551622723238457226804366654523161908704282792530334498925876
...
Invoke tx hash is: 2455538849277152825594824366964313930331085452149746033747086127466991639149
Call result: [2]

command: script run
status: success
```
</details>
<br>

As [an idempotency](#state-file) feature is turned on by default, executing the same script once again ends with a success
and only `call` functions are being executed (as they do not change the network state):

<!-- { "ignored_output": true } -->
```shell
$ sncast \
  --account example_user \
  script run map_script \
  --network sepolia
```

<details>
<summary>Output:</summary>

```shell
Class hash of the declared contract: 1922774777685257258886771026518018305931014651657879651971507142160195873652
Deployed the contract to address: 3478557462226312644848472512920965457566154264259286784215363579593349825684
Invoke tx hash is: 1373185562410761200747829131886166680837022579434823960660735040169785115611
Call result: [2]
command: script run
status: success
```
</details>
<br>


whereas, when we run the same script once again with `--no-state-file` flag set, it fails (as the `Map` contract is already deployed):

<!-- { "ignored_output": true } -->
```shell
$ sncast \
  --account example_user \
  script run map_script \
  --network sepolia \
  --no-state-file
```

<details>
<summary>Output:</summary>

```shell
command: script run
message:
    0x6d6170206465706c6f79206661696c6564 ('map deploy failed')

status: script panicked
```
</details>
<br>

## Error handling

Each of `declare`, `deploy`, `invoke`, `call` functions return `Result<T, ScriptCommandError>`, where `T` is a corresponding response struct.
This allows for various script errors to be handled programmatically.
Script errors implement `Debug` trait, allowing the error to be printed to stdout.

### Minimal example with `assert!` and `println!`

```rust
{{#include ../../listings/error_handling/src/error_handling.cairo}}
```

More on deployment scripts errors [here](../appendix/sncast-library/errors.md).
