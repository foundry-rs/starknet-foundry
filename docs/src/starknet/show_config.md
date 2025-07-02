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
  --account my_account \
  show-config 
```

<details>
<summary>Output:</summary>

```shell
Chain ID:            alpha-sepolia
RPC URL:             http://127.0.0.1:5055/rpc
Account:             my_account
Accounts File Path:  [..]/accounts.json
Wait Timeout:        300s
Wait Retry Interval: 5s
Show Explorer Links: true
```
</details>
<br>