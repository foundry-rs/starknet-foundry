# Blake Hash Support

Starting from Starknet version 0.14.1, the network switched from Poseidon hash to Blake hash for compiled class hashes.

Starknet Foundry version 0.53.0-rc.1 and later includes support for Blake hash, ensuring compatibility with the new Starknet version.

## Common Issue: Mismatch Compiled Class Hash

If you encounter a `Mismatch compiled class hash` error, it likely means you're using an older version of Starknet Foundry that doesn't support Blake hash.

**Solution**: Upgrade to Starknet Foundry version 0.53.0-rc.1 or later:

```shell
asdf install starknet-foundry latest
```

```shell
asdf set --home starknet-foundry latest
```

This will update your installation to the latest version with Blake hash support.

> 📝 **Note**
>
> For more detailed installation instructions, see the [Installation](../getting-started/installation.md) guide.
