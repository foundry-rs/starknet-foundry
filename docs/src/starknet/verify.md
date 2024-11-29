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

<!-- { "contract_name": "HelloStarknet", "ignored_output": true } -->
```shell
$ sncast \
    verify \
    --contract-address 0x01e4ebe3278ab4633a9d0d3f5c4290001f29bc3179a70e570b6817dd7f8264fa \
    --contract-name HelloStarknet \
    --verifier walnut \
    --network sepolia
```

<details>
<summary>Output:</summary>

```shell
You are about to submit the entire workspace's code to the third-party chosen verifier at walnut, and the code will be publicly available through walnut's APIs. Are you sure? (Y/n) Y

command: verify
message: Contract verification has started. You can check the verification status at the following link: https://api.walnut.dev/v1/verification/77f1d905-fdb4-4280-b7d6-57cd029d1259/status.
```
</details>
<br>

> 📝 **Note**
> Contract name is a part after the `mod` keyword in your contract file. It may differ from package name defined in `Scarb.toml` file.
