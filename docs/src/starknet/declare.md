# Declaring new contracts

Starknet provides a distinction between contract class and instance. This is similar to difference between writing the code of a `class MyClass {}` and creating a new instance of it `let myInstance = MyClass()` in object-oriented programming languages.

To deploy a new contract, for instance, you need to:

    1. Declare a contract on the network
    2. Deploy an instance of that declared contract

For detailed API description, see [declare command reference](../appendix/cast/index.html#declare).

## Usage example

> 📝 **Note**
> Building a contract before running `declare` is not required. Starknet Foundry cast builds a contract during declaration under the hood using [scarb](https://docs.swmansion.com/scarb).

First make sure your contract was initialized with scarb (Scarb.toml is present in project directory or one of its parent directories). Example `Scarb.toml` file can look like this:

```toml
[package]
name = "myawesomecontract"
version = "0.1.0"

[dependencies]
starknet = ">=1.1.1"

[[target.starknet-contract]]
casm = true

[lib]
sierra = false

[tool.protostar]
network = "testnet"
rpc_url = "http://127.0.0.1:5050/rpc"
account = "myuser"

```

Then run:

```shell
$ cast declare --contract-name SimpleBalance
command: Declare
class_hash: 0x8448a68b5ea1affc45e3fd4b8b480ea36a51dc34e337a16d2567d32d0c6f8a
transaction_hash: 0x7ad0d6e449e33b6581a4bb8df866c0fce3919a5ee05a30840ba521dafee217f
```

> 📝 **Note**
> Contract name may differ from package name defined in Scarb.toml file.

> 💡 **Info**
> The declare transaction must be signed and requires paying a fee, similarly to how invoke transaction does. See [signing](./invoke.md#signing) for more details.
