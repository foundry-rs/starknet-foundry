# Cairo Deployment Scripts

## Overview

> ⚠️⚠️⚠️ Highly experimental code, a subject to change  ⚠️⚠️⚠️

Starknet Foundry cast can be used to run deployment scripts written in Cairo, using `script` subcommand. 
It aims to provide similar functionality to Foundry's `forge script`. 

To start writing a deployment script in Cairo just add `cast_std` as a dependency to you scarb package and make sure to
have a `main` function in the module you want to run.

Please note that **`sncast script` is in develoment**. While it is already possible to declare, deploy, invoke and call 
contracts from within Cairo, its interface, internals and feature set can change rapidly each version. 

Some of the planned features that will be included in future versions are:

- scripts idempotency
- dispatchers support
- better error handling
- logging
- account creation/deployment
- multicall support
- dry running the scripts
- init subcommand

and more!

## Examples

This example script declares, deploys and interacts with an example [map contract](https://github.com/foundry-rs/starknet-foundry/tree/master/crates/cast/tests/data/contracts/map):

```cairo
use sncast_std::{
    declare, deploy, invoke, call, DeclareResult, DeployResult, InvokeResult, CallResult
};
use debug::PrintTrait;

fn main() {
    let max_fee = 99999999999999999;
    let salt = 0x3;

    let declare_result = declare('Map', Option::Some(max_fee));

    let class_hash = declare_result.class_hash;
    let deploy_result = deploy(
        class_hash, ArrayTrait::new(), Option::Some(salt), true, Option::Some(max_fee)
    );
    
    'Deployed the contract to address'.print();
    deploy_result.contract_address.print();

    let invoke_result = invoke(
        deploy_result.contract_address, 'put', array![0x1, 0x2], Option::Some(max_fee)
    );
    
    'Invoke tx hash is'.print();
    invoke_result.transaction_hash.print();

    let call_result = call(deploy_result.contract_address, 'get', array![0x1]);
    assert(call_result.data == array![0x2], *call_result.data.at(0));
}
```

The script should be included in a scarb package. The directory structure and config for this example looks like this:

```shell
$ tree
.
├── contracts
│    ├── Scarb.toml
│    └── src
│        └── lib.cairo
└── scripts
    ├── Scarb.toml
    └── src
        ├── lib.cairo
        └── map_script.cairo

5 directories, 5 files
```

```toml
[package]
name = "map_script"
version = "0.1.0"

[dependencies]
starknet = ">=2.3.0"
sncast_std = { git = "https://github.com/foundry-rs/starknet-foundry.git", tag = "v0.11.0" }
map = { path = "../contracts" }

[lib]
sierra = true
casm = true

[[target.starknet-contract]]
sierra = true
casm = true
build-external-contracts = [
    "map::Map"
]
```

Please note that `map` contract was specified as the dependency. In our example, it resides in the filesystem. It has to
be compiled in `build-external-contracts`.

To run the script, do:

```shell
$ sncast \
  --rpc-url http://127.0.0.1:5050 \
  --account example_user \
  script ./scripts
  
(todo: dodaj output)
```

