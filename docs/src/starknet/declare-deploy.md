# Deploying an Undeclared Contract

`sncast` allows declaring and deploying contracts through dedicated commands.
The `declare-deploy` command simplifies this pipeline by performing these two actions at the same time. It is used to declare a contract and deploy it immediately. If the contract has been already declared, the command will behave exactly as `deploy`.

For detailed description, see [declare-deploy command reference](../appendix/sncast/declare-deploy.md).

## Examples

Command operates under all the circumstances [`declare`](./declare.md) and [`deploy`](./deploy.md) would do when invoked separately.

### General Example

Make sure you have a `Scarb.toml` in your project directory. Suppose we would like to declare and instantly deploy an example contract named `HelloSncast`, defined in the default project.

Running:

<!-- TODO(#2736) -->
<!-- { "ignored": true } -->
```shell
$ sncast --account myuser \
    --url http://127.0.0.1:5050/rpc \ 
    declare-deploy \
    --fee-token strk \
    --contract-name HelloSncast
```

results in declaration and deployment.

>  ⚠️ **Warning**
> This command relies on auto-estimation and does not allow specifying max fees explicitly.
> ⚠️ **Warning**
> Only a `fee-token` can be specified. Transaction versions for both declaration and deployment are inferred from token type.