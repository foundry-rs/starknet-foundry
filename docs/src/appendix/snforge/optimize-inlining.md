# `snforge optimize-inlining`

Find the optimal `inlining-strategy` value to minimize gas cost or contract size.

See [Inlining Optimizer](../inlining-optimizer.md) for a full guide.

## `<TEST_FILTER>`

Fully qualified name of the test to run at each threshold. Must be used together with `--exact`.

## `--contracts <CONTRACTS>`

Comma-delimited list of contract names or Cairo module paths
(e.g. `MyContract,my_package::MyOther`) to include in contract size checks.

Required.

## `--min-threshold <MIN_THRESHOLD>`

Minimum `inlining-strategy` value to test. Default: `0`.

## `--max-threshold <MAX_THRESHOLD>`

Maximum `inlining-strategy` value to test. Default: `250`.

## `--step <STEP>`

Step size between tested thresholds. Must be greater than zero. Default: `25`.

## `--max-contract-size <MAX_CONTRACT_SIZE>`

Maximum allowed contract file size in bytes. Thresholds that produce contracts exceeding this value are
skipped. Default: `4089446`.

## `--max-contract-felts <MAX_CONTRACT_FELTS>`

Maximum allowed number of felts in a compiled contract. Thresholds that exceed this limit are
skipped. Default: `81920`.

## `--gas`

After the search completes, update `Scarb.toml` with the threshold that minimizes runtime gas cost.

Conflicts with `--size`.

## `--size`

After the search completes, update `Scarb.toml` with the threshold that minimizes contract
bytecode L2 gas (a proxy for deployment cost).

Conflicts with `--gas`.

## `-e`, `--exact`

Required. Run only the test whose name exactly matches `TEST_FILTER`.

## `--max-threads <MAX_THREADS>`

Maximum number of threads used for test execution. Defaults to the number of available CPU cores.

## `-P`, `--profile <PROFILE>`

Specify the Scarb profile to use by name.

## `--release`

Use Scarb release profile.

## `--dev`

Use Scarb dev profile.

## `-F`, `--features <FEATURES>`

Comma separated list of features to activate.

## `--all-features`

Activate all available features.

## `--no-default-features`

Do not activate the `default` feature.

## `-h`, `--help`

Print help.
