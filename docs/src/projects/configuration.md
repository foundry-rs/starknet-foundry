# Project Configuration

## `snforge`

### Configuring `snforge` Settings in `Scarb.toml`

It is possible to configure `snforge` for all test runs through `Scarb.toml`.
Instead of passing arguments in the command line, set them directly in the file.

```toml
# ...
[tool.snforge]
exit_first = true
# ...
```

`snforge` automatically looks for `Scarb.toml` in the directory you are running the tests in or in any of its parents.

## `sncast`

### Defining Profiles in `snfoundry.toml`

To be able to work with the network, you need to supply `sncast` with a few parameters —
namely the rpc node url and an account name that should be used to interact with it.
This can be done
by either supplying `sncast` with those parameters directly [see more detailed CLI description,](../appendix/sncast.md)
or you can put them into `snfoundry.toml` file:

```toml
# ...
[sncast.myprofile]
account = "user"
accounts-file = "~/my_accounts.json"
url = "http://127.0.0.1:5050/rpc"
# ...
```

With `snfoundry.toml` configured this way, we can just pass `--profile myprofile` argument to make sure `sncast` uses parameters
defined in the profile.

> 📝 **Note**
> `snfoundry.toml` file has to be present in current or any of the parent directories.

> 📝 **Note**
> If there is a profile with the same name in Scarb.toml, scarb will use this profile. If not, scarb will default to using the dev profile.
> (This applies only to subcommands using scarb - namely `declare` and `script`).

> 💡 **Info**
> Not all parameters have to be present in the configuration - you can choose to include only some of them and supply
> the rest of them using CLI flags. You can also override parameters from the configuration using CLI flags.

```shell
$ sncast --profile myprofile \
    call \
    --contract-address 0x38b7b9507ccf73d79cb42c2cc4e58cf3af1248f342112879bfdf5aa4f606cc9 \
    --function get \
    --calldata 0x0 \
    --block-id latest

command: call
response: [0x0]
```

### Multiple Profiles

You can have multiple profiles defined in the `snfoundry.toml`.

### Default Profile

There is also an option to set up a default profile, which can be utilized without the need to specify a `--profile`. Here's an example:

```toml
# ...
[sncast.default]
account = "user123"
accounts-file = "~/my_accounts.json"
url = "http://127.0.0.1:5050/rpc"
# ...
```

With this, there's no need to include the `--profile` argument when using `sncast`.

```shell
$ sncast call \
    --contract-address 0x38b7b9507ccf73d79cb42c2cc4e58cf3af1248f342112879bfdf5aa4f606cc9 \
    --function get \
    --calldata 0x0 \
    --block-id latest

command: call
response: [0x1, 0x23, 0x4]
```

## Environmental variables

Programmers can use environmental variables in both `Scarb.toml::tool::snforge` and in `snfoundry.toml`. To use an environmental variable as a value, use its name prefixed with `$`. 
This might be useful, for example, to hide node urls in the public repositories. 
As an example:

```toml
# ...
[sncast.default]
account = "my_account"
accounts-file = "~/my_accounts.json"
url = "$NODE_URL"
# ...
```

Variable value are automatically resolved to numbers and booleans (strings `true`, `false`) if it is possible.
