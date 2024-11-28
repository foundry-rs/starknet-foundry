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

<!-- { "contract_name": "HelloStarknet" } -->
```shell
$ sncast --account user0 \
    declare \
    --contract-name HelloStarknet \
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

```shell
$ sncast declare \
    --contract-name HellloStarknet \
    --fee-token strk
```

<details>
<summary>Output:</summary>

```shell
command: Declare
class_hash: 0x0555d84fd95ab9fa84a826382ca91127336d4b3c640d8571c32c4e7717e38799
transaction_hash: [..]
```
</details>
<br>


### Deploy a contract

<!-- { "contract_name": "HelloStarknet" } -->
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
contract_address: 2545627361586725870760320986020069125077312730415714481413957911720773997114
transaction_hash: [..]
```
</details>
<br>

With arguments taken from `snfoundry.toml` file (default profile name):

<!-- { "contract_name": "HelloStarknet" } -->
```shell
$ sncast deploy \
--class-hash 0x0555d84fd95ab9fa84a826382ca91127336d4b3c640d8571c32c4e7717e38799 \
--fee-token strk

```

<details>
<summary>Output:</summary>

```shell
command: Deploy
contract_address: 2545627361586725870760320986020069125077312730415714481413957911720773997114
transaction_hash: [..]
```
</details>
<br>


### Invoke a contract

<!-- { "contract_name": "HelloStarknet" } -->
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
command: Invoke
transaction_hash: 0x7ad0d6e449e33b6581a4bb8df866c0fce3919a5ee05a30840ba521dafee217f
```
</details>
<br>


With arguments taken from `snfoundry.toml` file (default profile name):

<!-- { "contract_name": "HelloStarknet" } -->
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
command: Invoke
transaction_hash: 0x7ad0d6e449e33b6581a4bb8df866c0fce3919a5ee05a30840ba521dafee217f
```
</details>
<br>

### Call a contract

<!-- { "contract_name": "HelloStarknet" } -->
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
response: [0x0]
```
</details>
<br>


With arguments taken from `snfoundry.toml` file (default profile name):

<!-- { "contract_name": "HelloStarknet" } -->
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
response: [0x0]
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
