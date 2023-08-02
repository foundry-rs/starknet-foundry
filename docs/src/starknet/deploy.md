# Deploying New Contracts

## Overview

Starknet Foundry cast supports deploying smart contracts to a given network with the `sncast deploy` command.

It works by invoking a [Universal Deployer Contract](https://docs.openzeppelin.com/contracts-cairo/0.6.1/udc), which deploys the contract with the given class hash and constructor arguments.

For detailed CLI description, see [deploy command reference](../appendix/cast/deploy.md).

## Usage examples

### General example

After [declaring your contract](./declare.md), you can deploy it the following way:

```shell
$ sncast deploy --class-hash 0x8448a68b5ea1affc45e3fd4b8b480ea36a51dc34e337a16d2567d32d0c6f8a

command: Deploy
contract_address: 0x301316d47a81b39c5e27cca4a7b8ca4773edbf1103218588d6da4d3ed53035a
transaction_hash: 0x64a62a000240e034d1862c2bbfa154aac6a8195b4b2e570f38bf4fd47a5ab1e
```

### Deploying contract with constructor

For such a constructor in the declared contract

```rust    
#[constructor]
fn constructor(ref self: ContractState, first: felt252, second: u256) {
    ...
}
```

you have to pass constructor calldata to deploy it.

```shell
$ sncast deploy \
    --class-hash 0x8448a68b5ea1affc45e3fd4b8b480ea36a51dc34e337a16d2567d32d0c6f8a
    --constructor-calldata 0x1 0x1 0x0
    
command: Deploy
contract_address: 0x301316d47a81b39c5e27cca4a7b8ca4773edbf1103218588d6da4d3ed53035a
transaction_hash: 0x64a62a000240e034d1862c2bbfa154aac6a8195b4b2e570f38bf4fd47a5ab1e
```

> 📝 **Note**
> Although the constructor has only two params you have to pass more because u256 is serialized to two felts.
> It is important to know how types are serialized because all values passed as constructor calldata are
> interpreted as a field elements (felt252).
