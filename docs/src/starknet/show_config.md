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
  --url http://127.0.0.1:5050 \
  --account user1
  show-config

command: show-config
account: user1
chain_id: alpha-goerli
keystore: ../keystore
rpc_url: http://127.0.0.1:5050/rpc
```