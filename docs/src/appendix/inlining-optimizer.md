# Inlining Optimizer

The inlining optimizer helps you find the optimal value for the Scarb compiler's `inlining-strategy`
setting. This setting controls how aggressively the compiler inlines function calls,
which affects both runtime gas cost and contract bytecode size.
The optimizer tries to optimize the parameter for a subset of contracts that you choose from your project, against a benchmark you define as an snforge test case.

## Overview
During Cairo contracts compilation, the compiler applies an optimization called inlining. 
It works by replacing some functions calls, with "inlined" source code of the called function. 
This may make the resulting compilation artifacts larger, as we copy some of your code, but it can make the performance of the compiled code better. 
The amount of inlining that should be applied to your code is a balance between those two. 
Each of the projects you write may have a different "perfect" amount of inlining it needs. 
Also, the right amount of inlining may depend on your use case - sometimes, if your contract will be called rarely, it may make sense to use less inlining and save gas during the contract declaration. 
There is really no way for the compiler to find this balance during the compilation. 
Instead, the compiler assumes some default "weight" of the function, and inlines all functions below this "weight". 
This one-suit-all solution is practical, but leaves some leeway for you to optimize the compiler configuration to best suit your projects needs. 
The inlinig optimizer built into snforge is a tool, that should help you look for the right amount of inlining that your project needs. 

The inlining optimization during the compilation can be configured with [`inlining-strategy` key in your Scarb.toml file](https://docs.swmansion.com/scarb/docs/reference/manifest.html#inlining-strategy).
The `inlining-strategy` value determines the inlining threshold: the compiler inlines a function when its cost estimate is below this value. 
A higher threshold means more aggressive inlining, which can lower runtime gas but also increase contract bytecode size.
Note, that defining how compiler measures function "weight" is outside the scope for this document. 
Also, the exact definition may change between compiler releases! 
From practical standpoint, it should be enough to treat this as a blackbox hyperparameter of your compilation.

The `snforge optimize-inlining` helps you better understand the effects of changing the `inlining-strategy` key, by compiling your project for a range of inlining keys and benchmark them against a predefined benchmark. 

## Usage

The optimizer requires a single representative test to measure gas at each threshold. 
Pass it with `--exact` and a fully qualified test name, the same way as `snforge test --exact`:
It also requires you to specify a subset of contracts from your project that should be benchmarked. 
You can specify by passing their names / paths to  `--contracts` argument. 
Note, that only contracts specified with this argument will be changed before executing your benchmarks - all other contracts are compiled with default inlinig strategy.

```shell
$ snforge optimize-inlining --contracts MyContract --exact my_package::tests::my_test
```

> 📝 **Note**
>
> The test should exercise the hot paths you care about!

The optimizer works on a copy of the project in a temporary directory so that intermediate
`Scarb.toml` edits do not affect your working tree.

## Output

After testing all thresholds the optimizer prints a results table:

```
┌──────────────┬─────────────────┬──────────────────┬──────────────────────────┬────────┐
│  Threshold   │    Total Gas    │  Contract Size   │ Contract Bytecode L2 Gas │ Status │
├──────────────┼─────────────────┼──────────────────┼──────────────────────────┼────────┤
│          0   │        123456   │        204800    │                  512000  │   ✓    │
│         25   │        118000   │        215040    │                  537600  │   ✓    │
│         50   │        115200   │        225280    │                  563200  │   ✓    │
│        ...                                                                           │
└──────────────┴─────────────────┴──────────────────┴──────────────────────────┴────────┘

Lowest runtime gas cost:     threshold=50, gas=115200, contract bytecode L2 gas=563200
Lowest contract size cost:   threshold=0,  gas=123456, contract bytecode L2 gas=512000
```

A PNG graph is also saved to the project's `target` directory, showing how gas and contract
bytecode L2 gas vary with the threshold.

## Applying the Result

Add `--gas` or `--size` to automatically write the optimal threshold back to `Scarb.toml`:

```shell
# Minimize runtime gas cost
$ snforge optimize-inlining --contracts MyContract --exact my_package::tests::my_test --gas

# Minimize contract bytecode L2 gas (deployment cost)
$ snforge optimize-inlining --contracts MyContract --exact my_package::tests::my_test --size
```

This updates the active Scarb profile with:

```toml
[profile.dev.cairo]
inlining-strategy = 50
```

> 📝 **Note**
>
> `--gas` and `--size` are mutually exclusive. Without either flag, `Scarb.toml` is not modified
> and the results are only printed for your review.

## Contract Size Limits

By default the optimizer skips thresholds that produce contracts exceeding Starknet's on-chain
limits. You can adjust these limits with `--max-contract-size` (bytes) and `--max-contract-program-len`:

```shell
$ snforge optimize-inlining \
    --contracts MyContract \
    --exact my_package::tests::my_test \
    --max-contract-size 3000000 \
    --max-contract-program-len 60000
```

Thresholds that violate either limit are marked with `✗` in the results table and are excluded from
the optimal threshold selection.

## Search Range and Step

By default the optimizer tests thresholds from `0` to `250` in steps of `25`. Adjust this with
`--min-threshold`, `--max-threshold`, and `--step`:

```shell
$ snforge optimize-inlining \
    --contracts MyContract \
    --exact my_package::tests::my_test \
    --min-threshold 20 \
    --max-threshold 100 \
    --step 10
```

A smaller step gives a finer-grained result but requires more compilations and test runs.

## Passing Test Arguments

`snforge optimize-inlining` accepts the same flags as `snforge test` for controlling fuzzer runs,
profiles, features, and so on. For example, to run under the `release` profile:

```shell
$ snforge optimize-inlining --contracts MyContract --exact my_package::tests::my_test --release
```
