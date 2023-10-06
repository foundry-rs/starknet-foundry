# Printing Current Configuration

## Overview

Sometimes, before executing any other cast command, one may want to make sure that the right
configuration settings are being used (eg proper network or account is used).

For those cases you can call `show-config` with all the same configuration related arguments, you can also just
replace subcommand and anything after it with `show-config` and it should show all the configuration that will be
used.

Anything configuration that you cannot see there means its not set properly or is not being detected.

## Examples

### General example

```shell
$ sncast \
  --rpc_url http://127.0.0.1:5050 \
  --acount user1
  show-config

command: show-config
account: user1
account_file_path: ../account-file
chain_id: alpha-goerli
keystore: ../keystore
rpc_url: http://127.0.0.1:5050/rpc
```