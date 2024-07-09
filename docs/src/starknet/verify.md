# Verifying Contracts

## Overview

Starknet Foundry `sncast` supports verifying Cairo contract classes with the `sncast verify` command by submitting the source code to a selected verification provider.

For detailed CLI description, see [verify command reference](../appendix/sncast/verify.md).

> ⚠️ **Warning**
> Please be aware that submitting the source code means it will be publicly exposed through the provider's APIs.

## Verification Providers

### Walnut

Walnut is a tool for step-by-step debugging of StarkNet transactions. You can request early access to Walnut on [walnut.dev](https://walnut.dev). Note that Walnut requires you to specify the StarkNet version in your `Scarb.toml` config file.

## Example

First, ensure that you have created a `Scarb.toml` file for your contract (it should be present in the project directory or one of its parent directories). Make sure the contract has already been deployed on the network. Then run:

```shell
$ sncast --url http://127.0.0.1:5050/rpc \
    verify \
    --contract-address 0x1234567890abcdef \
    --contract-name SimpleBalance \
    --verifier walnut \
    --network mainnet
```

> 📝 **Note**
> Contract name is a part after the `mod` keyword in your contract file. It may differ from package name defined in `Scarb.toml` file.
