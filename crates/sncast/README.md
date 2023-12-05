# Cast - Starknet Foundry CLI

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

All subcommand usages are shown for two scenarios - when all necessary arguments are supplied using CLI, and when `url`, `accounts-file` and `account` arguments are taken from `Scarb.toml`. To learn more about configuring profiles with parameters in `Scarb.toml` file, please refer to the [documentation](https://foundry-rs.github.io/starknet-foundry/projects/configuration.html#defining-profiles-in-scarbtoml).

### Declare a contract

```shell
$ sncast --account myuser \
    --url http://127.0.0.1:5050/rpc \
    declare \
    --contract-name SimpleBalance

command: Declare
class_hash: 0x8448a68b5ea1affc45e3fd4b8b480ea36a51dc34e337a16d2567d32d0c6f8a
transaction_hash: 0x7ad0d6e449e33b6581a4bb8df866c0fce3919a5ee05a30840ba521dafee217f
```


With arguments taken from `Scarb.toml` file (default profile name):

```shell
$ sncast declare \
    --contract-name SimpleBalance

command: Declare
class_hash: 0x8448a68b5ea1affc45e3fd4b8b480ea36a51dc34e337a16d2567d32d0c6f8a
transaction_hash: 0x7ad0d6e449e33b6581a4bb8df866c0fce3919a5ee05a30840ba521dafee217f
```

### Deploy a contract

```shell
$ sncast --account myuser \
    --url http://127.0.0.1:5050/rpc \
    deploy --class-hash 0x8448a68b5ea1affc45e3fd4b8b480ea36a51dc34e337a16d2567d32d0c6f8a

command: Deploy
contract_address: 0x301316d47a81b39c5e27cca4a7b8ca4773edbf1103218588d6da4d3ed53035a
transaction_hash: 0x64a62a000240e034d1862c2bbfa154aac6a8195b4b2e570f38bf4fd47a5ab1e
```


With arguments taken from `Scarb.toml` file (default profile name):

```shell
$ sncast deploy --class-hash 0x8448a68b5ea1affc45e3fd4b8b480ea36a51dc34e337a16d2567d32d0c6f8a

command: Deploy
contract_address: 0x301316d47a81b39c5e27cca4a7b8ca4773edbf1103218588d6da4d3ed53035a
transaction_hash: 0x64a62a000240e034d1862c2bbfa154aac6a8195b4b2e570f38bf4fd47a5ab1e
```


### Invoke a contract

```shell
$ sncast --url http://127.0.0.1:5050 \
    --account example_user \
    invoke \
    --contract-address 0x4a739ab73aa3cac01f9da5d55f49fb67baee4919224454a2e3f85b16462a911 \
    --function "some_function" \
    --calldata 1 2 3

command: Invoke
transaction_hash: 0x7ad0d6e449e33b6581a4bb8df866c0fce3919a5ee05a30840ba521dafee217f
```


With arguments taken from `Scarb.toml` file (default profile name):

```shell
$ sncast invoke \
    --contract-address 0x4a739ab73aa3cac01f9da5d55f49fb67baee4919224454a2e3f85b16462a911 \
    --function "some_function" \
    --calldata 1 2 3

command: Invoke
transaction_hash: 0x7ad0d6e449e33b6581a4bb8df866c0fce3919a5ee05a30840ba521dafee217f
```

### Call a contract

```shell
$ sncast --url http://127.0.0.1:5050 \
    call \
    --contract-address 0x4a739ab73aa3cac01f9da5d55f49fb67baee4919224454a2e3f85b16462a911 \
    --function "some_function" \
    --calldata 1 2 3

command: call
response: [0x0]
```


With arguments taken from `Scarb.toml` file (default profile name):

```shell
$ sncast call \
    --contract-address 0x4a739ab73aa3cac01f9da5d55f49fb67baee4919224454a2e3f85b16462a911 \
    --function some_function \
    --calldata 1 2 3

command: call
response: [0x0]
```


## Development

Refer to [documentation](https://foundry-rs.github.io/starknet-foundry/development/environment-setup.html) to make sure you have all the pre-requisites, and to obtain an information on how to help to develop `sncast`.

Please make sure you're using scarb installed via asdf - otherwise some tests may fail.
To verify, run:

```shell
$ which scarb
$HOME/.asdf/shims/scarb
```

If you previously installed scarb using official installer, you may need to remove this installation or modify your PATH to make sure asdf installed one is always used.
