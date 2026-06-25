# `class-hash`
Calculate the class hash of a contract.

Either `--contract-name` or `--sierra-file` must be provided.

## `--contract-name <CONTRACT_NAME>`
Optional.

The name of the contract. The contract name is the part after the `mod` keyword in your contract file.
This argument also accepts a module tree path, for example `hello_sncast::contracts::HelloSncast` or `contracts::HelloSncast`.
When provided, the contract is built before its class hash is calculated.

## `--sierra-file <SIERRA_FILE>`
Optional.

Path to an already compiled Sierra contract class JSON file.

## `--package <NAME>`
Optional.

Specifies the scarb package to be used. Cannot be used together with `--sierra-file`.

## General Example

```shell
$ sncast utils \
  class-hash \
  --contract-name HelloSncast
```

<details>
<summary>Output:</summary>

```shell
Class Hash: 0x0[..]
```
</details>
<br>
<<<<<<< HEAD

## Example with a Sierra file

```shell
$ sncast utils \
  class-hash \
  --sierra-file target/dev/hello_sncast_HelloSncast.contract_class.json
```

<details>
<summary>Output:</summary>

```shell
Class Hash: 0x06134dac6883ac052bea8db10bc9ee8af6bad5eeab001cd61b8eaba6a03be103
```
</details>
<br>
=======
>>>>>>> origin/master
