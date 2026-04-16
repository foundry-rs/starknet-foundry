# Predpeloyed contracts

`snforge` provides a set of predeployed contracts for use in testing. To support this functionality, we maintain CASM of these contracts directly within our codebase. Because these contracts are subject to periodic updates, these files need to be updated. The list below details all predeployed contracts and the information required to keep them current.

## STRK and ETH 

Contracts are taken from the [`starkgate-contracts`](https://github.com/starknet-io/starkgate-contracts) repository.
Currently used version: [`v3.0.0`](https://github.com/starknet-io/starkgate-contracts/commit/07e11c39119a10d5742735be5b1d51894ebf5311).

Links to Cairo contracts:
- STRK: [`ERC20Lockable`](https://github.com/starknet-io/starkgate-contracts/blob/07e11c39119a10d5742735be5b1d51894ebf5311/packages/strk/src/erc20_lockable.cairo)
- ETH : [`ERC20Mintable`](https://github.com/starknet-io/starkgate-contracts/blob/07e11c39119a10d5742735be5b1d51894ebf5311/packages/sg_token/src/erc20_mintable.cairo)

### Artifacts generation

1. Clone `starkgate-contracts` repository and checkout `v3.0.0` tag.

    ```shell
    git clone https://github.com/starknet-io/starkgate-contracts
    cd starkgate-contracts
    git checkout v3.0.0
    ```

2. Enable CASM generation by adding the following lines to `Scarb.toml`.
This should be done in `sg_token` and `strk` packages.

    ```toml
    ...

    [[target.starknet-contract]]
    casm = true

    ...
    ```

3. Adjust compiler configuration in workspace `Scarb.toml`:

    ```toml
    ...

    [profile.release.cairo]
    unstable-add-statements-code-locations-debug-info = true
    unstable-add-statements-functions-debug-info = true
    panic-backtrace = true

    ...
    ```

4. Compile contracts with `scarb`

    ```shell
    scarb --release build
    ```

5. Visit `target/release` directory and copy relevant artifacts.
