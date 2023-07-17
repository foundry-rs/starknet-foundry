# Declaring new contracts

Starknet provides a distinction between contract class and instance. This is similar to the difference between writing the code of a `class MyClass {}` and creating a new instance of it `let myInstance = MyClass()` in object-oriented programming languages.

Declaring a contract is a necessary step to have your contract available on the network. Once a contract is declared, it then can be deployed and then interacted with.

For a detailed CLI description, see [declare command reference](../appendix/cast/declare.md).

## Usage example

> 📝 **Note**
> Building a contract before running `declare` is not required. Starknet Foundry `cast` builds a contract during declaration under the hood using [Scarb](https://docs.swmansion.com/scarb).

First make sure that you have created a [`Scarb.toml`](../projects/template.md) file for your contract (it should be present in project directory or one of its parent directories).

Then run:

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

> 📝 **Note**
> Contract name is a part after the `mod` keyword in your contract file. It may differ from package name defined in `Scarb.toml` file.

> 📝 **Note**
> In the above example we supply cast with `--account`, `--network` and `--url` flags. If `Scarb.toml` is present, and have this properties set, values provided using these flags will override values from `Scarb.toml`. Learn more about `Scarb.toml` configuration [here](../projects/configuration.md#cast).
