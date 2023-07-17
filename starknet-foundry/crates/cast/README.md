# Cast - Starknet Foundry CLI

Starknet Foundry `cast` is a command line tool for performing Starknet RPC calls. With it, you can easily interact with Starknet contracts!

Note, that at the moment, `cast` only supports contracts written in Cairo 1 and Cairo 2.

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

You can download latest version of `cast` [here](https://github.com/foundry-rs/starknet-foundry/releases).

## Documentation

For more details on Starknet Foundry `cast`, please visit [our docs](https://foundry-rs.github.io/starknet-foundry/starknet/index.html) 

## Example usages
All subcommand usages are shown for two scenarios - when all necessary arguments are supplied using CLI, and when `network`, `url` and `account` arguments are taken from `Scarb.toml`. To learn more about configuring profiles with parameters in `Scarb.toml` file, please refer to the [documentation](https://foundry-rs.github.io/starknet-foundry/projects/configuration.html#defining-profiles-in-scarbtoml).

### Declare a contract

```shell
$ cast --account myuser \
    --network testnet \
    --url http://127.0.0.1:5050/rpc \ 
    declare \
    --contract-name SimpleBalance

command: Declare
class_hash: 0x8448a68b5ea1affc45e3fd4b8b480ea36a51dc34e337a16d2567d32d0c6f8a
transaction_hash: 0x7ad0d6e449e33b6581a4bb8df866c0fce3919a5ee05a30840ba521dafee217f
```


With arguments taken from `Scarb.toml` file (default profile name):

```shell
$ cast declare \
    --contract-name SimpleBalance

command: Declare
class_hash: 0x8448a68b5ea1affc45e3fd4b8b480ea36a51dc34e337a16d2567d32d0c6f8a
transaction_hash: 0x7ad0d6e449e33b6581a4bb8df866c0fce3919a5ee05a30840ba521dafee217f
```

### Deploy a contract

```shell
$ cast deploy --class-hash 0x8448a68b5ea1affc45e3fd4b8b480ea36a51dc34e337a16d2567d32d0c6f8a

command: Deploy
contract_address: 0x301316d47a81b39c5e27cca4a7b8ca4773edbf1103218588d6da4d3ed53035a
transaction_hash: 0x64a62a000240e034d1862c2bbfa154aac6a8195b4b2e570f38bf4fd47a5ab1e
```


With arguments taken from `Scarb.toml` file (default profile name):

```shell
$ cast --account myuser \
    --network testnet \
    --url http://127.0.0.1:5050/rpc \ 
    deploy --class-hash 0x8448a68b5ea1affc45e3fd4b8b480ea36a51dc34e337a16d2567d32d0c6f8a

command: Deploy
contract_address: 0x301316d47a81b39c5e27cca4a7b8ca4773edbf1103218588d6da4d3ed53035a
transaction_hash: 0x64a62a000240e034d1862c2bbfa154aac6a8195b4b2e570f38bf4fd47a5ab1e
```

### Invoke a contract

```shell
$ cast --rpc_url http://127.0.0.1:5050 \
    --network testnet \
    --account example_user \
    invoke \
    --contract-address 0x4a739ab73aa3cac01f9da5d55f49fb67baee4919224454a2e3f85b16462a911 \
    --entry-point-name "some_function" \
    --calldata 1 2 3

command: Invoke
transaction_hash: 0x7ad0d6e449e33b6581a4bb8df866c0fce3919a5ee05a30840ba521dafee217f
```


With arguments taken from `Scarb.toml` file (default profile name):

```shell
$ cast invoke \
    --contract-address 0x4a739ab73aa3cac01f9da5d55f49fb67baee4919224454a2e3f85b16462a911 \
    --entry-point-name "some_function" \
    --calldata 1 2 3

command: Invoke
transaction_hash: 0x7ad0d6e449e33b6581a4bb8df866c0fce3919a5ee05a30840ba521dafee217f
```

### Call a contract

```shell
$ cast --rpc_url http://127.0.0.1:5050 \
    --network testnet \
    call \
    --contract-address 0x4a739ab73aa3cac01f9da5d55f49fb67baee4919224454a2e3f85b16462a911 \
    --entry-point-name "some_function" \
    --calldata 1 2 3

command: Call
response: [FieldElement { inner: 0x0000000000000000000000000000000000000000000000000000000000000000 }]
```


With arguments taken from `Scarb.toml` file (default profile name):

```shell
$ cast call \
    --contract-address 0x4a739ab73aa3cac01f9da5d55f49fb67baee4919224454a2e3f85b16462a911 \
    --function-name some_function \
    --calldata 1 2 3

command: Call
response: [FieldElement { inner: 0x0000000000000000000000000000000000000000000000000000000000000000 }]
```


## Development

Refer to [project readme](https://github.com/foundry-rs/starknet-foundry#development) to make sure you have all the pre-requisites, and to obtain an information on how to help to develop `cast`.
