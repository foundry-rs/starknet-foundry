# Cairo Contract Verification via Block Explorers

## Context

Cairo smart contracts deployed to Starknet are only visible as a Cairo Bytecode and their's ABIs, which is difficult to decipher and understand. To increase readability of a smart contract, it's source code can also be made public, however in order to ensure trustlessness, the source code vs resulting Cairo Bytecode verification can be performed. Such verification and its results can be performed and made public by popular Blockchain Explorers.

## Goal

This proposal includes an extension to `sncast` utility enabling a contract owner to perform contract verification against a selected Blockchain Explorer API. We propose to create a first, reference implementation for the Voyager APIs.

## Proposed Solution

We propose to design a dedicated `verify` command for the `sncast` tool, and add a mechanism whereby this command can be implemented for various Blockchain Explorers. We define a "generic" contract verification interface and propose to implement the interface by "adapters" specific to respective explorers. 

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

#### `--contract-name`

Required.
Name of the contract to be submitted for verification.

#### `--contract-address`

Required.
Address of the contract to be submitted for verification.

#### `--class-hash`

Required.
Address of the contract to be submitted for verification.

#### `--verifier <VERIFIER NAME>`

Required.
Specifies the Blockchain Explorer to verify with.  

Options as of writing of this document: 
 - voyager
 - starkscan

#### `--network <NETWORK_NAME>`

Required.
Specifies the network on which Blockchain Explorers will do the verification

Options are:
 - mainnet - for verification at mainnet
 - goerli -  for verification on testnet

### Contract Verification interface

To implement contract verification for a specific explorer, it is required to implement a generic interface (request/response), as described below. The data structures proposed can be eventually extended to cater for detailed requirements of subsequent explorer adapters.

#### Request

- `ContractAddress` - `ContractAddress`
- `ClassHash` - `String`
- `ClassName` - `String`
- `SourceCode` - collection of file records (`.cairo` files  + `Scarb.toml`):
  - `FilePath` - `String`
  - `FileContent` - `String`

Note: `ContractAddress` and `ClassHash` are mutually exclusive. One of them must be provided.

#### Response

- `VerificationStatus` - `Enum` (`OK`, `Error`)
- `Errors` - (Optional) collection of error records:
  - `ErrorMessage` - `String`
  - `ErrorDetail` - `String`

### Voyager API adapter plugin

A sample request in the Voyager API adapter implementation will look as follows: 
```rust

const url = `${voyager.testnet.url}/contract/`

const payload = serde_json::json!({
        "contract_address": "0x0", // this is optional if class_hash is provided
        "class_hash": "0x0", // this is optional if contract_address is provided
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

println!(resp.verification_status)
```
