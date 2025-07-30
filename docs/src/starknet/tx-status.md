# Inspecting Transactions

## Overview

Starknet Foundry `sncast` supports the inspection of transaction statuses on a given network with the `sncast tx-status` command.

For a detailed CLI description, refer to the [tx-status command reference](../appendix/sncast/tx-status.md).

## Usage Examples

### Inspecting Transaction Status

You can track the details about the execution and finality status of a transaction in the given network by using the transaction hash as shown below:

```shell
$ sncast \
 tx-status \
 0x07d2067cd7675f88493a9d773b456c8d941457ecc2f6201d2fe6b0607daadfd1 \
 --network sepolia
```

<details>
<summary>Output:</summary>

```shell
Success: Transaction status retrieved

Finality Status:  Accepted on L1
Execution Status: Succeeded
```

</details>

## Block Explorer Links

When you use commands like `sncast deploy`, `sncast declare`, or `sncast account create`, `sncast` will display block explorer links in the output if the network supports it. These links allow you to quickly inspect the transaction, contract, or class on a public block explorer such as Starkscan or Voyager.

### When Are Explorer Links Shown?

- **Public Networks:** Explorer links are enabled by default for public networks (e.g., Sepolia, Mainnet) if the tool can detect a supported explorer for the network.
- **Devnet:** Explorer links are not shown for localhost or devnet URLs (e.g., `http://localhost:5050/rpc` or `http://127.0.0.1:5050/rpc`). This prevents confusion, as local networks typically do not have a public explorer.
- **Disabling Explorer Links:** You can turn off explorer links by setting `show-explorer-links = false` in your `snfoundry.toml` profile. See the [snfoundry.toml appendix](../appendix/snfoundry-toml.md#show-explorer-links) for details.
- **Environment Variable Override:** You can force explorer links to be shown by setting the environment variable `FORCE_SHOW_EXPLORER_LINKS=1` before running a command. This is useful for testing or custom explorers.

### Example Output

```shell
Success: Deployment completed

Contract Address: 0x0...
Transaction Hash: 0x0...

To see deployment details, visit:
contract: https://sepolia.starkscan.co/contract/0x0...
transaction: https://sepolia.starkscan.co/tx/0x0...
```
