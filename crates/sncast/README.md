# sncast - Starknet Foundry CLI

Starknet Foundry `sncast` is a command line tool for performing Starknet RPC calls. With it, you can easily interact with Starknet contracts!

Note, that `sncast` only officially supports contracts written in Cairo 2.

## Table of contents

<!-- TOC -->
  * [Installation](#installation)
  * [Documentation](#documentation)
  * [Example usages](#example-usages)
    * [Declaring contracts](#declare-a-contract)
    * [Deploying contracts](#deploy-a-contract)
    * [Invoking contracts](#invoke-a-contract)
    * [Calling contracts](#call-a-contract)
  * [Development](#development)
<!-- TOC -->

## Installation

You can download latest version of `sncast` [here](https://github.com/foundry-rs/starknet-foundry/releases).

## Documentation

For more details on Starknet Foundry `sncast`, please visit [our docs](https://foundry-rs.github.io/starknet-foundry/starknet/index.html) 

## Example usages

All subcommand usages are shown for two scenarios - when all necessary arguments are supplied using CLI, and when `url`, `accounts-file` and `account` arguments are taken from `snfoundry.toml`. To learn more about configuring profiles with parameters in `snfoundry.toml` file, please refer to the [documentation](https://foundry-rs.github.io/starknet-foundry/projects/configuration.html#defining-profiles-in-snfoundrytoml).

### Declare a contract

<!-- TODO(#2736) -->
<!-- { "ignored": true } -->
```shell
$ sncast --account user0 \
    declare \
    --contract-name HelloSncast \
    --fee-token strk
```

<details>
<summary>Output:</summary>

```shell
command: Declare
class_hash: [..]
transaction_hash: [..]
```
</details>
<br>

With arguments taken from `snfoundry.toml` file (default profile name):

<!-- TODO(#2736) -->
<!-- { "ignored": true } -->
```shell
$ sncast declare \
    --contract-name HelloSncast \
    --fee-token strk
```

<details>
<summary>Output:</summary>

```shell
command: Declare
class_hash: [..]
transaction_hash: [..]
```
</details>
<br>


### Deploy a contract

<!-- TODO(#2736) -->
<!-- { "contract_name": "HelloSncast", "ignored": true } -->
```shell
$ sncast --account user0 \
    deploy --class-hash 0x0555d84fd95ab9fa84a826382ca91127336d4b3c640d8571c32c4e7717e38799 \
    --url http://127.0.0.1:5055 \
    --fee-token strk
```

<details>
<summary>Output:</summary>

```shell
command: Deploy
contract_address: [..]
transaction_hash: [..]
```
</details>
<br>

With arguments taken from `snfoundry.toml` file (default profile name):

<!-- TODO(#2736) -->
<!-- { "contract_name": "HelloSncast", "ignored": true } -->
```shell
$ sncast deploy \
--class-hash 0x0555d84fd95ab9fa84a826382ca91127336d4b3c640d8571c32c4e7717e38799 \
--fee-token strk

```

<details>
<summary>Output:</summary>

```shell
command: Deploy
contract_address: [..]
transaction_hash: [..]
```
</details>
<br>


### Invoke a contract

<!-- { "contract_name": "HelloSncast" } -->
```shell
$ sncast \
    --account user0 \
    invoke \
    --contract-address 0x0555d84fd95ab9fa84a826382ca91127336d4b3c640d8571c32c4e7717e38799 \
    --function "sum_numbers" \
    --arguments '1, 2, 3' \
    --url http://127.0.0.1:5055/rpc \
    --fee-token strk
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


With arguments taken from `snfoundry.toml` file (default profile name):

<!-- { "contract_name": "HelloSncast" } -->
```shell
$ sncast invoke \
    --contract-address 0x0555d84fd95ab9fa84a826382ca91127336d4b3c640d8571c32c4e7717e38799 \
    --function "sum_numbers" \
    --arguments '1, 2, 3' \
    --url http://127.0.0.1:5055/rpc \
    --fee-token strk
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

### Call a contract

<!-- { "contract_name": "HelloSncast" } -->
```shell
$ sncast \
    call \
    --contract-address 0x0555d84fd95ab9fa84a826382ca91127336d4b3c640d8571c32c4e7717e38799 \
    --function "sum_numbers" \
    --arguments '1, 2, 3' \
    --url http://127.0.0.1:5055/rpc
```

<details>
<summary>Output:</summary>

```shell
command: call
response: [0x6]
```
</details>
<br>


With arguments taken from `snfoundry.toml` file (default profile name):

<!-- { "contract_name": "HelloSncast" } -->
```shell
$ sncast call \
    --contract-address 0x0555d84fd95ab9fa84a826382ca91127336d4b3c640d8571c32c4e7717e38799 \
    --function "sum_numbers" \
    --arguments '1, 2, 3' \
    --url http://127.0.0.1:5055/rpc
```

<details>
<summary>Output:</summary>

```shell
command: call
response: [0x6]
```
</details>
<br>


## Development

Refer to [documentation](https://foundry-rs.github.io/starknet-foundry/development/environment-setup.html) to make sure you have all the pre-requisites, and to obtain an information on how to help to develop `sncast`.

Please make sure you're using scarb installed via asdf - otherwise some tests may fail.
To verify, run:

```shell
$ which scarb
```

<details>
<summary>Output:</summary>

```shell
$HOME/.asdf/shims/scarb
```
</details>
<br>

If you previously installed scarb using official installer, you may need to remove this installation or modify your PATH to make sure asdf installed one is always used.
