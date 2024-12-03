# `Fees` and `versions`


By default, fees for transactions are paid in STRK. But there is possibility to pay fees in ETH by specifying `--fee-token` 
and proper transaction version with `--version`. However, in the near future this feature will be deprecated in the starknet and `sncast` also.

> 💡 **Info**
> V3 (STRK) transactions have additional options, that give you more control over transaction fee. You can specify the maximum gas unit price and the maximum gas for the transaction. 
This is done using the `--max-gas` and `--max-gas-unit-price` flags.

## sncast account deploy

When deploying an account, you can specify the version of the transaction and the fee token to use. The table below shows which token is used for which version of the transaction:

| Version | Fee Token |
|---------|-----------|
| v1      | eth       |
| v3      | strk      |

When paying in STRK, you don't need to provide any additional flags. Just run:

```shell
$ sncast account deploy \
    --name some-name \
    --max-fee 9999999999999
```

In case of paying in ETH. You need to set `--fee-token` to `eth` and `--version` to `v1`:

```shell
$ sncast account deploy \
    --name some-name \
    --fee-token eth \
    --version v1 \
    --max-fee 9999999999999
```

> 📝 **Note**
> The unit used in `--max-fee` flag is the smallest unit of the given fee token. For ETH it is WEI, for STRK it is FRI.

## sncast deploy

Currently, there are two versions of the deployment transaction: v1 and v3. The table below shows which token is used for which version of the transaction:


| Version | Fee Token |
|---------|-----------|
| v1      | eth       |
| v3      | strk      |

## sncast declare

Currently, there are two versions of the declare transaction: v2 and v3. The table below shows which token is used for which version of the transaction:


| Version | Fee Token |
|---------|-----------|
| v2      | eth       |
| v3      | strk      |

## sncast invoke and sncast multicall run

Currently, there are two versions of invoke transaction: v1 and v3. The table below shows which token is used for which version of the transaction:


| Version | Fee Token |
|---------|-----------|
| v1      | eth       |
| v3      | strk      |