# Cairo Contract Verification via Block Explorers

## Context

Cairo smart contracts deployed to Starknet are only visible as a Cairo Bytecode and their's ABIs, which is difficult to decipher and understand. To increase readability of a smart contract, it's source code can also be made public, however in order to ensure trustlessness, the source code vs resulting Cairo Bytecode verification can be performed. Such verification and its results can be performed and made public by popular Blockchain Explorers.

## Goal

This proposal includes an extension to `sncast` utility enabling a contract owner to perform contract verification against a selected Blockchain Explorer API. We propose to create a first, reference implementation for the Voyager APIs.

## Proposed Solution

We propose to design a dedicated `verify` command for the `sncast` tool, and add a mechanism whereby this command can be implemented for various Blockchain Explorers.

### `sncast` utility command - `verify`

Command name: `verify`

The `verify` command will perform following actions:
- select the verifiation logic to use (based on `--verifier` parameter)
- pick the selected scarb workspace source code from local filesystem (based on `Scarb.toml` file)
- for the selected verifier it will upload the Scarb workspace contracts source code to the verifier's API
- call the verifier's API to trigger verification of the uploaded source code
- wait for results of the verification API call
- respond to the user with the results 

#### Parameters

#### `--path-to-scarb-toml, -s <PATH>`

Optional.
Path to `Scarb.toml` file.
If supplied, cast will not look for `Scarb.toml`` file in current (or parent) directory, but will use this path instead.

#### `--contract-name`

Required.
Name of the contract to be submitted for verification.

#### `--contract-address`

Required.
Address of the contract to be submitted for verification.

#### `--verifier <VERIFIER NAME>`

Optional.
Specifies the Blockchan Explorer to verify with.  
Default: `voyager`

Options are: 
 - voyager
 - starkscan

#### `--verifier-url <URL>`

Optional.
Specifies the Blockchain Explorer's Verification API URL ().  
Default: the default API URL for selected verifier (eg. https://goerli.voyager.online/) (testnet)
Options are: 
 - `https://voyager.online/` - for verification at mainnet
 - `https://goerli.voyager.online/` -  for verification on testnet

### Voyager API plugin

A sample request to the voyager API end point will look as follows: 
```rust

const url = `${voyager.testnet.url}/contract/`

const payload = serde_json::json!({
        "contract_address": "0x0",
        "contract_name": "balance",
        "source_code": {
            "Scarb.toml" : {
                """
                [package]
                ...
                [dependencies]
                starknet = "2.3.0-rc0"
                """
            },
            "src/lib.cairo" : {
                """
                mod balance;
                mod forty_two;
                """
            },
            "src/balance.cairo" : {
                """
                #[starknet::interface]
                ...
                #[starknet::contract]
                mod Balance {
                    ...
                }
                """
            },
            "src/forty_two.cairo" : {
                """
                #[starknet::contract]
                mod FortyTwo {
                    ...
                }
                """
            }
        }
    });

const resp = client
    .post(url)
    .json(&payload)
    .send()
    .await?;

println!(resp.verify_status)
```
