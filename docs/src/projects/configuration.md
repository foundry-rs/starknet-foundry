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
> `--profile` only selects the `snfoundry.toml` profile. 
> 
> The Scarb profile is set with `--scarb-profile` or the `scarb-profile` field under `[sncast.<profile>]` (defaults to value `release`). 
> See [common flags](../appendix/sncast/common.md)). 
> If that Scarb profile does not exist in `Scarb.toml`, `sncast` falls back with a warning.
> (Applies to subcommands using Scarb - namely `declare`, `verify`, `script`, and `utils class-hash`.)

> 💡 **Info**
> Not all parameters have to be present in the configuration - you can choose to include only some of them and supply
> the rest of them using CLI flags. You can also override parameters from the configuration using CLI flags.

```shell
$ sncast --profile myprofile \
    call \
    --contract-address 0x0589a8b8bf819b7820cb699ea1f6c409bc012c9b9160106ddc3dacd6a89653cf \
    --function get_balance \
    --block-id latest
```

<details>
<summary>Output:</summary>

```shell
Success: Call completed

Response:     0x34bc
Response Raw: [0x34bc]
```
</details>
<br>

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
    --contract-address 0x0589a8b8bf819b7820cb699ea1f6c409bc012c9b9160106ddc3dacd6a89653cf \
    --function get_balance \
    --block-id latest
```

<details>
<summary>Output:</summary>

```shell
Success: Call completed

Response:     0x34bc
Response Raw: [0x34bc]
```
</details>
<br>

### Global Configuration

Global configuration file is a [`snfoundry.toml`](https://foundry-rs.github.io/starknet-foundry/appendix/snfoundry-toml.html), 
which is a common storage for configurations to apply to multiple projects across various directories.
This file is stored in a predefined location and is used to store profiles that can be used from any location on your computer.

#### Interaction Between Local and Global Profiles

Each setting in the effective config comes from the highest-precedence layer that defines it.

Layer precedence (highest to lowest):

1. CLI flags
2. `[local.<profile>]`
3. `[local.default]`
4. `[global.<profile>]`
5. `[global.default]`
6. internal defaults

If a layer is missing, or it doesn't define a particular setting, the setting is looked up in the next layer.

The `[local.<name>]` and `[global.<name>]` layers are considered only when `--profile <name>` is set.
If `--profile <name>` is provided, at least one of `[local.<name>]` or `[global.<name>]` must be present.


#### Global Configuration File Location
The global configuration is stored in a specific location depending on the operating system:

- macOS/Linux : The global configuration file is located at `$HOME/.config/starknet-foundry/snfoundry.toml`

> 📝 **Note**
> If missing, global configuration file will be created automatically on running any `sncast` command for the first time.

### Config Interaction Example

```
root/
├── .config/
│   └── starknet-foundry/
│       └── snfoundry.toml -> A
└── /../../
        └── projects/
            ├── snfoundry.toml -> B
            └── cairo-projects/
                └── opus-magnum/
```

#### Glossary

**A:** Global configuration file containing the profiles `default` and `testnet`:
```toml
[sncast.default]
..

[sncast.testnet]
..
```
**B:** Local configuration file containing the profiles `default` and `mainnet`:
```toml
[sncast.default]
..

[sncast.mainnet]
..
```

#### No local config

In any directory in the file system, a user can run the `sncast` command using the `default` and `testnet` profiles, because they are defined in global config (file A).
If no local config file is present, only profiles defined in global config will be used.

- If the `testnet` profile is specified, the effective config is built by layering `global.testnet` -> `global.default` -> internal defaults.
- If no profile is specified, the effective config is built by layering `global.default` -> internal defaults.

#### Local config present

When running `sncast` from the `opus-magnum` directory, there is a configuration file in the parent directory (file B). 
This setup allows for the use of the following profiles: `default`, `testnet`, and `mainnet`:

- If the `mainnet` profile is specified, the effective config is built by layering `local.mainnet` -> `local.default` -> `global.default` -> internal defaults. 
  The `mainnet` profile does not need to exist in the global configuration as long as it is present in the local one.
- If the `testnet` profile is specified, the effective config is built by layering  `local.default` -> `global.testnet` -> `global.default` -> internal defaults.
  The `testnet` profile does not need to exist in the local configuration as long as it is present in the global one.
- If no profile is specified, the effective config is built by layering `local.default` -> `global.default` -> internal defaults.

## Environmental Variables

Programmers can use environmental variables in both `Scarb.toml::tool::snforge` and in `snfoundry.toml`. To use an environmental variable as a value, use its name either with or without curly braces, prefixed with `$` (e.g. `${MY_ENV}` or `$MY_ENV`).
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

Variable values are automatically resolved to numbers and booleans (strings `true`, `false`) where possible.
