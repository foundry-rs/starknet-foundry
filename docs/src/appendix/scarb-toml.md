# Forge Configuration Reference

- `[[tool.snforge]]` - General configuration of [`snforge`](./forge.md):
    - [`exit_first`](./forge/test.md#-x---exit-first)
    - [`fuzzer_runs`](./forge/test.md#-x---exit-first)
    - [`fuzzer_seed`](./forge/test.md#-s---fuzzer-seed-fuzzerseed)
- `[[tool.snforge.fork]]` - Configuration of [forks](../testing/fork-testing.md#configure-fork-in-the-scarbtoml):
    - `name` - Name used to reference the fork inside tests
    - `url` - Url to the rpc of the network to be forked
    - `block_id` - Identifies block until which the network state will be synced:
        - `number` - A block number, example: `block_id.number = "12345"`
        - `hash` - A block hash, example: `block_id.hash = "0xffff"`
        - `tag` - A block tag: example `block_id.tag = "Latest"`
