# Verifying Contracts

## Overview

Starknet Foundry `sncast` supports verifying Cairo contract classes with the `sncast verify` command by submitting the source code to a selected verification provider. Verification provides transparency, making the code accessible to users and aiding debugging tools.

The verification provider guarantees that the submitted source code aligns with the deployed contract class on the network by compiling the source code into Sierra bytecode and comparing it with the network-deployed Sierra bytecode.

For detailed CLI description, see [verify command reference](../appendix/sncast/verify.md).

> ⚠️ **Warning**
> Please be aware that submitting the source code means it will be publicly exposed through the provider's APIs.

## Verification Providers

### Walnut

Walnut is a tool for step-by-step debugging of Starknet transactions. You can learn more about Walnut here [walnut.dev](https://walnut.dev). Note that Walnut requires you to specify the Starknet version in your `Scarb.toml` config file.

### Voyager

[Voyager](https://voyager.online) is a Starknet block explorer that provides contract verification services. It allows you to verify your contracts and make their source code publicly available on the explorer. Voyager supports both mainnet and testnet (Sepolia) networks.

## Examples

First, ensure that you have created a `Scarb.toml` file for your contract (it should be present in the project directory or one of its parent directories). Make sure the contract has already been deployed on the network.

### Using Walnut

<!-- { "ignored_output": true, "replace_network": false } -->
```shell
$ sncast \
    verify \
    --class-hash 0x031966c9fe618bcee61d267750b9d46e3d71469e571e331f35f0ca26efe306dc \
    --contract-name SimpleBalance \
    --verifier walnut \
    --network sepolia
```

<details>
<summary>Output:</summary>

```shell

    You are about to submit the entire workspace code to the third-party verifier at walnut.

    Important: Make sure your project does not include sensitive information like private keys.

    Are you sure you want to proceed? (Y/n): Y

Success: Verification completed

Contract successfully verified
```

</details>

### Using Voyager

<!-- { "ignored_output": true, "replace_network": false } -->
```shell
$ sncast \
    verify \
    --class-hash 0x031966c9fe618bcee61d267750b9d46e3d71469e571e331f35f0ca26efe306dc \
    --contract-name SimpleBalance \
    --verifier voyager \
    --network sepolia
```

<details>
<summary>Output:</summary>

```shell

    You are about to submit the entire workspace code to the third-party verifier at voyager.

    Important: Make sure your project's Scarb.toml does not include sensitive information like private keys.

    Are you sure you want to proceed? (Y/n): Y

Success: Verification completed

SimpleBalance submitted for verification, you can query the status at: https://sepolia-api.voyager.online/beta/class-verify/job/[..]
```

</details>
<br>

> 📝 **Note**
> Contract name is a part after the `mod` keyword in your contract file. It may differ from package name defined in `Scarb.toml` file.
