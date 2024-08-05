# Joining the Declaration and Deployment Together

`sncast` allows declaring and deploying contracts through dedicated commands. However, sometimes contracts are meant to be only instantiated once, thus the standard way of declaration and deployment is unnecessarily time-consuming.

The `declare-deploy` command allows performing both actions at once.

For detailed description, see [declare-deploy command reference](../appendix/sncast/declare-deploy.md).

## Examples

Command operates under all the circumstances `declare` and `deploy` would do when invoked separately.

### General Example

Make sure you have a `Scarb.toml` in your project directory. Suppose we would like to declare and instantly deploy an example contract named `SimpleBalance`, defined in the default project.

Running:

```shell
$ sncast --account myuser \
    --url http://127.0.0.1:5050/rpc \ 
    declare-deploy \
    --fee-token strk \
    --contract-name SimpleBalance
```

results in declaration and deployment.

> 📝 **Note**
> This command relies on auto-estimation and does not allow specifying max fees explicitly.

> 📝 **Note**
> Only a `fee-token` can be specified. Transaction versions for both declaration and deployment are inferred from token type.