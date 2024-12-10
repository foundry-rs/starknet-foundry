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
	--url http://127.0.0.1:5055 \
    --contract-name HelloSncast
```

<details>
<summary>Output:</summary>

```shell
command: declare
class_hash: [..]
transaction_hash: [..]

To see declaration details, visit:
class: https://starkscan.co/search/[..]
transaction: https://starkscan.co/search/[..]
```
</details>
<br>

> 📝 **Note**
> Contract name is a part after the `mod` keyword in your contract file. It may differ from package name defined in `Scarb.toml` file.

> 📝 **Note**
> In the above example we supply `sncast` with `--account` and `--url` flags. If `snfoundry.toml` is present, and has
> the properties set, values provided using these flags will override values from `snfoundry.toml`. Learn more about `snfoundry.toml`
> configuration [here](../projects/configuration.md#sncast).

> 💡 **Info**
> Max fee will be automatically computed if `--max-fee <MAX_FEE>` is not passed.


