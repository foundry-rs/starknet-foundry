# `Fees` and `versions`

Historically, fees for transactions on Starknet had to be paid exclusively with ETH. However, with the rollout of v3
transactions, users now have the additional option to pay these fees using STRK.

> 💡 **Info**
> V3 transactions have additional options, that give you more control over transaction fee. You can specify the maximum gas unit price and the maximum gas for the transaction. 
This is done using the `--max-gas` and `--max-gas-unit-price` flags.

Cast allows you to specify either the version of the transaction you want to send or the fee token you want to pay the fees in. This is done using
the `--version` and `--fee-token` flags.

> 💡 **Info**
> Don't worry if you're not sure which version to use, it will be inferred automatically based on the fee token you
> provide. The same goes for the fee token, if you provide a version, the fee token will be inferred.

## sncast account deploy

When deploying an account, you can specify the version of the transaction and the fee token to use. The table below shows which token is used for which version of the transaction:

| Version | Fee Token |
|---------|-----------|
| v1      | eth       |
| v3      | strk      |

When paying in STRK, you need to either set `--fee-token` to `strk`:

```shell
$ sncast account deploy \
    --name some-name \
    --fee-token strk \
    --max-fee 9999999999999
```
or set `--version` to `v3`:

```shell
$ sncast account deploy \
    --name some-name \
    --version v3 \
    --max-fee 9999999999999
```

In case of paying in ETH, same rules apply. You need to set either `--fee-token` to `eth`:

```shell
$ sncast account deploy \
    --name some-name \
    --fee-token eth \
    --max-fee 9999999999999
```

or set `--version` to `v1`:

```shell
$ sncast account deploy \
    --name some-name \
    --version v1 \
    --max-fee 9999999999999
```

> 📝 **Note**
> The unit used in `--max-fee` flag is the smallest unit of the given fee token. For ETH it is Wei, for STRK it is Fri.

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