# Declaring New Contracts

Starknet provides a distinction between contract class and instance. This is similar to the difference between writing the code of a `class MyClass {}` and creating a new instance of it `let myInstance = MyClass()` in object-oriented programming languages.

Declaring a contract is a necessary step to have your contract available on the network. Once a contract is declared, it then can be deployed and then interacted with.

For a detailed CLI description, see [declare command reference](../appendix/sncast/declare.md).

## Examples

### General Example

> 📝 **Note**
> Building a contract before running `declare` is not required. Starknet Foundry `sncast` builds a contract during declaration under the hood using [Scarb](https://docs.swmansion.com/scarb).

First make sure that you have created a `Scarb.toml` file for your contract (it should be present in project directory or one of its parent directories).

Then run:

<!-- TODO(#2736) -->
<!-- { "ignored": true } -->
```shell
$ sncast --account my_account \
    declare \
	--network sepolia \
    --contract-name HelloSncast
```

<details>
<summary>Output:</summary>

```shell
Success: Declaration completed

Contract Address: 0x0[..]
Transaction Hash: 0x0[..]

To see declaration details, visit:
class: https://starkscan.co/search/[..]
transaction: https://starkscan.co/search/[..]

To deploy a contract of this class, run:
sncast --account my_account deploy --class-hash 0x[..] --network sepolia
```
</details>
<br>

> 📝 **Note**
> Contract name is a part after the `mod` keyword in your contract file. It may differ from package name defined in `Scarb.toml` file.

> 📝 **Note**
> In the above example we supply `sncast` with `--account` and `--network` flags. If `snfoundry.toml` is present, and has
> the properties set, values provided using these flags will override values from `snfoundry.toml`. Learn more about `snfoundry.toml`
> configuration [here](../projects/configuration.md#sncast).

> 💡 **Info**
> Max fee will be automatically computed if `--max-fee <MAX_FEE>` is not passed.

# Declaring a Contract by Fetching It From a Different Starknet Instance

In some cases, you may need to declare a contract that was already compiled elsewhere and reuse the exact same class hash across multiple networks.
This is especially important for some contracts, e.g. Universal Deployer Contract (UDC), which must preserve the same class hash across mainnet, testnets, and appchains.

Compiling a contract locally with a different Cairo compiler version may result in a different class hash.

To avoid this, you can use the `declare-from` which allows you to declare a contract by providing its class hash and the source network where it is already declared.

## Example

Let's consider a basic contract, which is already declared on Sepolia network.

To declare it on another network, e.g. local devnet, run:

```shell
$ sncast --account my_account \
    declare-from \
    --class-hash 0x283a4f96ee7de15894d9205a93db7cec648562cfe90db14cb018c039e895e78 \
    --source-network sepolia \
    --url http://127.0.0.1:5055/rpc
```

<details>
<summary>Output:</summary>

```shell
Success: Declaration completed

Class Hash:       0x283a4f96ee7de15894d9205a93db7cec648562cfe90db14cb018c039e895e78
Transaction Hash: 0x[..]
```
</details>
<br>


