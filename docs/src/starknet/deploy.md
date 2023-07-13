# Deploying New Contracts

## Overview

Starknet Foundry cast supports deploying smart contracts to a given network with the `cast deploy` command.

It works by invoking a [Universal Deployer Contract](https://docs.openzeppelin.com/contracts-cairo/0.6.1/udc), which deploys the contract with the given class hash and constructor arguments.

For detailed CLI description, see [deploy command reference](../reference/cast/index.html#deploy).

## Usage example

After [declaring your contract](./declare.md), you can deploy it the following way:

```shell
$ cast deploy --class-hash 0x8448a68b5ea1affc45e3fd4b8b480ea36a51dc34e337a16d2567d32d0c6f8a
command: Deploy
contract_address: 0x301316d47a81b39c5e27cca4a7b8ca4773edbf1103218588d6da4d3ed53035a
transaction_hash: 0x64a62a000240e034d1862c2bbfa154aac6a8195b4b2e570f38bf4fd47a5ab1e
```

> 💡 **Info**
> Deploying a contract is a transaction, so it must be signed and requires paying a fee, similarly to how invoke transaction does. See [signing](./invoke.md#signing) for more details.
