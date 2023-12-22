# Cairo Deployment Scripts

## Overview

> ⚠️⚠️⚠️ Highly experimental code, a subject to change  ⚠️⚠️⚠️

Starknet Foundry cast can be used to run deployment scripts written in Cairo, using `script` subcommand.
It aims to provide similar functionality to Foundry's `forge script`.

To start writing a deployment script in Cairo just add `cast_std` as a dependency to you scarb package and make sure to
have a `main` function in the module you want to run. `cast_std` docs can be found [here](../appendix/sncast-library.md).

Please note that **`sncast script` is in development**. While it is already possible to declare, deploy, invoke and call
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

### Minimal Example (Without Contract Deployment)

This example shows how to call an already deployed contract. Please find full example with contract deployment [here](#full-example-with-contract-deployment).

```cairo
use sncast_std::{invoke, call, InvokeResult, CallResult};

fn main() {
    let eth = 0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7;
    let addr = 0x0089496091c660345BaA480dF76c1A900e57cf34759A899eFd1EADb362b20DB5;
    let call_result = call(eth.try_into().unwrap(), 'allowance', array![addr, addr]);
    let call_result = *call_result.data[0];
    assert(call_result == 0, call_result);

    let call_result = call(eth.try_into().unwrap(), 'decimals', array![]);
    let call_result = *call_result.data[0];
    assert(call_result == 18, call_result);
}
```

The script should be included in a scarb package. The directory structure and config for this example looks like this:

```shell
$ tree
.
├── src
│   ├── my_script.cairo
│   └── lib.cairo
└── Scarb.toml
```

```toml
[package]
name = "my_script"
version = "0.1.0"

[dependencies]
starknet = ">=2.3.0"
sncast_std = { git = "https://github.com/foundry-rs/starknet-foundry.git", tag = "v0.12.0" }
```

To run the script, do:

```shell
$ sncast \
  --url http://127.0.0.1:5050 \
  script my_script

command: script
status: success
```

### Full Example (With Contract Deployment)

This example script declares, deploys and interacts with an example [map contract](https://github.com/foundry-rs/starknet-foundry/tree/master/crates/sncast/tests/data/contracts/map):

```cairo
use sncast_std::{
    declare, deploy, invoke, call, DeclareResult, DeployResult, InvokeResult, CallResult, get_nonce
};
use debug::PrintTrait;

fn main() {
    let max_fee = 99999999999999999;
    let salt = 0x3;

    let declare_result = declare('Map', Option::Some(max_fee), Option::None);

    let nonce = get_nonce('latest');
    let class_hash = declare_result.class_hash;
    let deploy_result = deploy(
        class_hash, ArrayTrait::new(), Option::Some(salt), true, Option::Some(max_fee), Option::Some(nonce)
    );

    'Deployed the contract to address'.print();
    deploy_result.contract_address.print();

    let invoke_nonce = get_nonce('pending');
    let invoke_result = invoke(
        deploy_result.contract_address, 'put', array![0x1, 0x2], Option::Some(max_fee), Option::Some(invoke_nonce)
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
```

```toml
[package]
name = "map_script"
version = "0.1.0"

[dependencies]
starknet = ">=2.3.0"
sncast_std = { git = "https://github.com/foundry-rs/starknet-foundry.git", tag = "v0.12.0" }
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

Please note that `map` contract was specified as the dependency. In our example, it resides in the filesystem. To generate the artifacts for it that will be accessible from the script you need to use the `build-external-contracts` property.

To run the script, do:

```shell
$ sncast \
  --url http://127.0.0.1:5050 \
  --account example_user \
  script map_script

[DEBUG]	Contract address               	(raw: 0x436f6e74726163742061646472657373
[DEBUG]	                               	(raw: 0x6f9492c9c2751ba5ccab5b7611068a6347d7b313c6f073d2edea864f062d730
[DEBUG]	Invoke tx hash                 	(raw: 0x496e766f6b652074782068617368
[DEBUG]	                               	(raw: 0x60175b3fca296fedc38f1a0ca30337ae666bb996a43beb2f6fe3a3fa90d3e6b

command: script
status: success
```
