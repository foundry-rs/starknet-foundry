# Deploying New Contracts

## Overview

Starknet Foundry `sncast` supports deploying smart contracts to a given network with the `sncast deploy` command.

It works by invoking a [Universal Deployer Contract](https://docs.openzeppelin.com/contracts-cairo/2.x/udc), which
deploys the contract with the given class hash and constructor arguments.

For contract to be deployed on starknet, it must be declared first.
It can be done with the [declare command](./declare.md) or by using the [`--contract-name`](#deploying-by-contract-name)
flag in the `deploy` command.

For detailed CLI description, see [deploy command reference](../appendix/sncast/deploy.md).

## Usage Examples

### General Example

After [declaring your contract](./declare.md), you can deploy it the following way:

```shell
$ sncast \
    --account my_account \
    deploy \
    --network sepolia \
    --class-hash 0x0227f52a4d2138816edf8231980d5f9e6e0c8a3deab45b601a1fcee3d4427b02
```

<details>
<summary>Output:</summary>

```shell
Success: Deployment completed

Contract Address: 0x0[..]
Transaction Hash: 0x0[..]

To see deployment details, visit:
contract: https://sepolia.voyager.online/contract/[..]
transaction: https://sepolia.voyager.online/tx/[..]
```
</details>
<br>

> 💡 **Info**
> Max fee will be automatically computed if `--max-fee <MAX_FEE>` is not passed.


### Deploying Contract With Constructor

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
    --class-hash 0x02e93ad9922ac92f3eed232be8ca2601fe19f843b7af8233a2e722c9975bc4ea \
    --constructor-calldata 0x1 0x2 0x3
```

<details>
<summary>Output:</summary>

```shell
Success: Deployment completed

Contract Address: 0x0[..]
Transaction Hash: 0x0[..]

To see deployment details, visit:
contract: https://sepolia.voyager.online/contract/[..]
transaction: https://sepolia.voyager.online/tx/[..]
```
</details>
<br>

> 📝 **Note**
> Although the constructor has only two params you have to pass more because u256 is serialized to two felts.
> It is important to know how types are serialized because all values passed as constructor calldata are
> interpreted as a field elements (felt252).

### Deploying by Contract Name

Instead of providing the `--class-hash` of an already declared contract, you can pass the name of the
contract from a Scarb project by providing the `--contract-name` flag.
Under the hood, if the passed contract was never declared to starknet, it will run the [declare](../starknet/declare.md)
command first and then execute the contract deployment.

> 📝 **Note**
> When passing `--contract-name` flag, `sncast` must wait for the declare transaction to be completed first.
> The contract might wait for a few seconds before executing the deployment.

> 📝 **Note**
> If fee arguments are provided to the method, same fee arguments will be used for **each** transaction separately.
> That is, the total fee paid for the operation will be **2 times the fee limits provided**.
>
> For better control over fee, use `declare` and `deploy` with  `--class-hash` separately.

<!-- TODO(#2736) -->
<!-- { "ignored": true } -->
```shell
$ sncast deploy \
    --contract-name HelloSncast
```

<details>
<summary>Output:</summary>

```shell
Success: Deployment completed

Contract Address:         0x0[..]
Class Hash:               0x0[..]
Declare Transaction Hash: 0x0[..]
Deploy Transaction Hash:  0x0[..]

To see deployment details, visit:
contract: [..]
class: [..]
deploy transaction: [..]
declare transaction: [..]
```
</details>

### Passing `salt` Argument

Salt is a parameter which modifies contract's address, if not passed it will be automatically generated.

```shell
$ sncast deploy \
    --class-hash 0x0227f52a4d2138816edf8231980d5f9e6e0c8a3deab45b601a1fcee3d4427b02 \
    --salt 0x123
```

<details>
<summary>Output:</summary>

```shell
Success: Deployment completed

Contract Address: 0x0[..]
Transaction Hash: 0x0[..]

To see deployment details, visit:
contract: https://sepolia.voyager.online/contract/[..]
transaction: https://sepolia.voyager.online/tx/[..]
```
</details>
<br>

### Passing `unique` Argument

Unique is a parameter which modifies contract's salt with the deployer address.
It can be passed even if the `salt` argument was not provided.

```shell
$ sncast deploy \
    --class-hash 0x0227f52a4d2138816edf8231980d5f9e6e0c8a3deab45b601a1fcee3d4427b02 \
    --unique
```

<details>
<summary>Output:</summary>
    
```shell
Success: Deployment completed

Contract Address: 0x0[..]
Transaction Hash: 0x0[..]

Details:
contract: https://sepolia.voyager.online/contract/[..]
transaction: https://sepolia.voyager.online/tx/[..]
```
</details>
