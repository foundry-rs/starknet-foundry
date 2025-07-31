# Block Explorers

When you use commands like `sncast deploy`, `sncast declare`, or `sncast account create`, `sncast` will display block explorer links in the output if the network supports it. These links allow you to quickly inspect the transaction, contract, or class on a public block explorer such as Starkscan or Voyager.

> 💡 **Tip**
> If you want to use a specific block explorer, see the [`block-explorer` configuration details](../appendix/snfoundry-toml.md#block-explorer).

## When Are Explorer Links Shown?

* **Public Networks:** Explorer links are enabled by default for public networks (e.g., Sepolia, Mainnet) if the tool can detect a supported explorer for the network.
* **Devnet:** Explorer links are not shown for localhost or devnet URLs (e.g., `http://localhost:5050/rpc` or `http://127.0.0.1:5050/rpc`). This prevents confusion, as local network transactions are not visible in local explorers.
* **Disabling Explorer Links:** You can turn off explorer links by setting `show-explorer-links = false` in your `snfoundry.toml` profile. See the [snfoundry.toml appendix](../appendix/snfoundry-toml.md#show-explorer-links) for details.
* **Environment Variable Override:** You can force explorer links to be shown by setting the environment variable `SNCAST_FORCE_SHOW_EXPLORER_LINKS=1` before running a command.

## Example Output

```shell
Success: Deployment completed

Contract Address: 0x0...
Transaction Hash: 0x0...

To see deployment details, visit:
contract: https://sepolia.starkscan.co/contract/0x0...
transaction: https://sepolia.starkscan.co/tx/0x0...
```
