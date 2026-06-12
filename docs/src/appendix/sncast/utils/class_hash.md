# `class-hash`
Calculate the class hash of a contract.

## `--contract-name <CONTRACT_NAME>`
Required.

The name of the contract. The contract name is the part after the `mod` keyword in your contract file.
This argument also accepts an absolute module tree path, for example `hello_sncast::HelloSncast`.

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
