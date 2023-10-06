# Printing Current Configuration

## Overview

Many times before executing any other cast command you want to make sure that you are passing the right
configuration settings and not mixing up commands that needs to be sent to development environment to mainnet :)

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