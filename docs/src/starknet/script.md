# Cairo Deployment Scripts

## Overview

> ⚠️⚠️⚠️ Highly experimental code, a subject to change  ⚠️⚠️⚠️

Starknet Foundry cast can be used to run deployment scripts written in Cairo, using `script run` subcommand.
It aims to provide similar functionality to Foundry's `forge script`.

To start writing a deployment script in Cairo just add `sncast_std` as a dependency to you scarb package and make sure to
have a `main` function in the module you want to run. `sncast_std` docs can be found [here](../appendix/sncast-library.md).

Please note that **`sncast script` is in development**. While it is already possible to declare, deploy, invoke and call
contracts from within Cairo, its interface, internals and feature set can change rapidly each version.

> ⚠️⚠️ By default, the nonce for each transaction is being taken from the pending block ⚠️⚠️
>
> Some RPC nodes can be configured with higher poll itervals, which means they may return "older" nonces
> in pending blocks, or even not be able to obtain pending blocks at all. When running a script you get an error like
> "Invalid transaction nonce", this might be the case and you may need to manually set both nonce and max_fee for
> transactions.
>
> Example:
>
>```cairo
>  let declare_result = declare("Map", Option::Some(max_fee), Option::Some(nonce)).expect('declare failed');
>```

Some of the planned features that will be included in future versions are:

- scripts idempotency
- dispatchers support
- logging
- account creation/deployment
- multicall support
- dry running the scripts

and more!

## Examples

### Initialize a script

To get started, a deployment script with all required elements can be initialized using the following command:

```shell
$ sncast script init my_script
```

For more details, see [init command](../appendix/sncast/script/init.md).

> 📝 **Note**
> To include a newly created script in an existing workspace, it must be manually added to the members list in the `Scarb.toml` file, under the defined workspace.
> For more detailed information about workspaces, please refer to the [Scarb documentation](https://docs.swmansion.com/scarb/docs/reference/workspaces.html).

### Minimal Example (Without Contract Deployment)

This example shows how to call an already deployed contract. Please find full example with contract deployment [here](#full-example-with-contract-deployment).

```cairo
use sncast_std::{invoke, call, CallResult};

fn main() {
    let eth = 0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7;
    let addr = 0x0089496091c660345BaA480dF76c1A900e57cf34759A899eFd1EADb362b20DB5;
    let call_result = call(eth.try_into().unwrap(), selector!("allowance"), array![addr, addr]).expect('call failed');
    let call_result = *call_result.data[0];
    assert(call_result == 0, call_result);

    let call_result = call(eth.try_into().unwrap(), selector!("decimals"), array![]).expect('call failed');
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
  script run my_script

command: script run
status: success
```

### Full Example (With Contract Deployment)

This example script declares, deploys and interacts with an example [map contract](https://github.com/foundry-rs/starknet-foundry/tree/master/crates/sncast/tests/data/contracts/map):

```cairo
use sncast_std::{
    declare, deploy, invoke, call, DeclareResult, DeployResult, InvokeResult, CallResult, get_nonce, DisplayContractAddress, DisplayClassHash
};

fn main() {
    let max_fee = 99999999999999999;
    let salt = 0x3;

    let declare_result = declare("Map", Option::Some(max_fee), Option::None).expect('declare failed');

    let nonce = get_nonce('latest');
    let class_hash = declare_result.class_hash;

    println!("Class hash of the declared contract: {}", declare_result.class_hash);

    let deploy_result = deploy(
        class_hash, ArrayTrait::new(), Option::Some(salt), true, Option::Some(max_fee), Option::Some(nonce)
    ).expect('deploy failed');

    println!("Deployed the contract to address: {}", deploy_result.contract_address);

    let invoke_nonce = get_nonce('pending');
    let invoke_result = invoke(
        deploy_result.contract_address, selector!("put"), array![0x1, 0x2], Option::Some(max_fee), Option::Some(invoke_nonce)
    ).expect('invoke failed');

    println!("Invoke tx hash is: {}", invoke_result.transaction_hash);

    let call_result = call(deploy_result.contract_address, selector!("get"), array![0x1]).expect('call failed');

    println!("Call result: {}", call_result);
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
  script run map_script

Class hash of the declared contract: 685896493695476540388232336434993540241192267040651919145140488413686992233
...
Deployed the contract to address: 2993684914933159551622723238457226804366654523161908704282792530334498925876
...
Invoke tx hash is: 2455538849277152825594824366964313930331085452149746033747086127466991639149
Call result: [2]

command: script run
status: success
```

## Error handling

Each of `declare`, `deploy`, `invoke`, `call` functions return `Result<T, ScriptCommandError>`, where `T` is a corresponding response struct. 
This allows for various script errors to be handled programmatically. 
Script errors implement `Debug` trait, allowing the error to be printed to stdout.

### Minimal example with assert and print

```rust
use sncast_std::{
    get_nonce, declare, DeclareResult, ScriptCommandError, ProviderError, StarknetError
};

fn main() {
    let max_fee = 9999999999999999999999999999999999;

    let declare_nonce = get_nonce('latest');
    let declare_result = declare("Map", Option::Some(max_fee), Option::Some(declare_nonce))
        .unwrap_err();
    println!("{:?}", declare_result);

    assert(
        ScriptCommandError::ProviderError(
            ProviderError::StarknetError(StarknetError::InsufficientAccountBalance)
        ) == declare_result,
        'ohno'
    )
}
```

stdout:
```shell
...
ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::InsufficientAccountBalance(())))
command: script
status: success
```

Some errors may contain an error message in the form of `ByteArray`

### Minimal example with an error msg:

```rust
use sncast_std::{call, CallResult, ScriptCommandError, ProviderError, StarknetError, ErrorData};

fn main() {
    let eth = 0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7.try_into().expect('bad address');
    let call_err: ScriptCommandError = call(
        eth, selector!("gimme_money"), array![]
    )
        .unwrap_err();

    println!("{:?}", call_err);
}
```
stdout:
```shell
...
ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::ContractError(ErrorData { msg: "Entry point EntryPointSelector(StarkFelt( ... )) not found in contract." })))
command: script
status: success
```

More on deployment scripts errors [here](../appendix/sncast-library/errors.md).
