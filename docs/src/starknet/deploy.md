# Deploying New Contracts

## Overview

Starknet Foundry `sncast` supports deploying smart contracts to a given network with the `sncast deploy` command.

It works by invoking a [Universal Deployer Contract](https://docs.openzeppelin.com/contracts-cairo/0.19.0/udc), which deploys the contract with the given class hash and constructor arguments.

For detailed CLI description, see [deploy command reference](../appendix/sncast/deploy.md).

## Usage Examples

### General Example

After [declaring your contract](./declare.md), you can deploy it the following way:

<!-- { "contract_name": "HelloStarknet" } -->
```shell
$ sncast \
    --account user0 \
    deploy \
    --url http://127.0.0.1:5055/rpc \
	--fee-token strk \
    --class-hash 0x0555d84fd95ab9fa84a826382ca91127336d4b3c640d8571c32c4e7717e38799
```

<details>
<summary>Output:</summary>

```shell
command: deploy
contract_address: [..]
transaction_hash: [..]

To see deployment details, visit:
contract: https://sepolia.starkscan.co/contract/[..]
transaction: https://sepolia.starkscan.co/tx/[..]
```
</details>
<br>

> 💡 **Info**
> Max fee will be automatically computed if `--max-fee <MAX_FEE>` is not passed.

> 💡 **Info**
> You can also choose to pay in Ether by setting `--fee-token` to `eth`.

### Deploying Contract With Constructor

For such a constructor in the declared contract

```rust    
#[constructor]
fn constructor(ref self: ContractState, first: felt252, second: u256) {
    ...
}
```

you have to pass constructor calldata to deploy it.

<!-- { "contract_name": "ConstructorContract" } -->
```shell
$ sncast deploy \
    --fee-token strk \
    --class-hash 0x8448a68b5ea1affc45e3fd4b8b480ea36a51dc34e337a16d2567d32d0c6f8a \
    --constructor-calldata 0x1 0x2 0x3
```

<details>
<summary>Output:</summary>

```shell
command: deploy
contract_address: [..]
transaction_hash: [..]

To see deployment details, visit:
contract: https://sepolia.starkscan.co/contract/[..]
transaction: https://sepolia.starkscan.co/tx/[..]
```
</details>
<br>

> 📝 **Note**
> Although the constructor has only two params you have to pass more because u256 is serialized to two felts.
> It is important to know how types are serialized because all values passed as constructor calldata are
> interpreted as a field elements (felt252).

### Passing `salt` Argument

Salt is a parameter which modifies contract's address, if not passed it will be automatically generated.

<!-- { "contract_name": "HelloStarknet" } -->
```shell
$ sncast deploy \
    --fee-token strk \
    --class-hash 0x0555d84fd95ab9fa84a826382ca91127336d4b3c640d8571c32c4e7717e38799 \
    --salt 0x123
```

<details>
<summary>Output:</summary>

```shell
command: deploy
contract_address: [..]
transaction_hash: [..]

To see deployment details, visit:
contract: https://sepolia.starkscan.co/contract/[..]
transaction: https://sepolia.starkscan.co/tx/[..]
```
</details>
<br>

### Passing `unique` Argument

Unique is a parameter which modifies contract's salt with the deployer address.
It can be passed even if the `salt` argument was not provided.

<!-- { "contract_name": "HelloStarknet" } -->
```shell
$ sncast deploy \
    --fee-token strk \
    --class-hash 0x0555d84fd95ab9fa84a826382ca91127336d4b3c640d8571c32c4e7717e38799 \
    --unique
```

<details>
<summary>Output:</summary>
    
```shell
command: deploy
contract_address: [..]
transaction_hash: [..]

Details:
contract: [..]
transaction: [..]
```
</details>
