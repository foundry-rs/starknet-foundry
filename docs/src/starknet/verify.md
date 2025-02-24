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

## Example

First, ensure that you have created a `Scarb.toml` file for your contract (it should be present in the project directory or one of its parent directories). Make sure the contract has already been deployed on the network.

Then run:

<!-- { "ignored_output": true, "replace_network": false } -->
```shell
$ sncast \
    verify \
    --class-hash 0x0227f52a4d2138816edf8231980d5f9e6e0c8a3deab45b601a1fcee3d4427b02 \
    --contract-name HelloSncast \
    --verifier walnut \
    --network sepolia
```

<details>
<summary>Output:</summary>

```shell

    You are about to submit the entire workspace code to the third-party verifier at walnut.

    Important: Make sure your project does not include sensitive information like private keys. The snfoundry.toml file will be uploaded. Keep the keystore outside the project to prevent it from being uploaded.

    Are you sure you want to proceed? (Y/n): Y

command: verify
message: Contract verification has started. You can check the verification status at the following link: https://app.walnut.dev/verification/status/77f1d905-fdb4-4280-b7d6-57cd029d1259.
```
</details>
<br>

> 📝 **Note**
> Contract name is a part after the `mod` keyword in your contract file. It may differ from package name defined in `Scarb.toml` file.
