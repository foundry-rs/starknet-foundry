# Printing Current Configuration

## Overview

Sometimes, before executing any other `sncast` command, one may want to make sure that the right
configuration settings are being used (eg proper network or account is used).

To see just that, a `show-config` subcommand can be used. You can just
replace any subcommand (and its parameters) with `show-config` and it will show currently used configuration.


## Examples

### General Example

```shell
$ sncast \
  --account user0 \
  show-config 
```

<details>
<summary>Output:</summary>

```shell
command: show-config
account: user0
chain_id: alpha-sepolia
rpc_url: http://127.0.0.1:5055
```
</details>
<br>