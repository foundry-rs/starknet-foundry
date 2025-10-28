# Gas and VM Resources Estimation

`snforge` supports gas and VM resources estimation for each individual test case.

It does not calculate the final transaction fee, for details on how fees are calculated, 
please refer to fee mechanism in [Starknet documentation](https://docs.starknet.io/learn/protocol/fees#overview).

## Gas Estimation

### Single Test

When the test passes with no errors, estimated gas is displayed this way:
```shell
[PASS] tests::simple_test (l1_gas: ~1, l1_data_gas: ~1, l2_gas: ~1)
```

This gas calculation is based on the collected Sierra gas or VM resources (that you can [display additionally on demand](#usage)),
storage updates, events and l1 <> l2 messages.

#### Gas Report

For individual tests, more detailed L2 gas usage can be displayed by passing the [`--gas-report`](../appendix/snforge/test.md#--gas-report) flag.
This will generate a table that shows gas statistics for each contract and function.

```shell
$ snforge test --gas-report
```

<details>
<summary>Output:</summary>

```shell
Collected 1 test(s) from hello_starknet package
Running 1 test(s) from tests/
[PASS] hello_starknet_integrationtest::test_contract::test_increase_balance (l1_gas: ~0, l1_data_gas: ~192, l2_gas: ~998280)
╭------------------------+-------+-------+-------+---------+---------╮
| HelloStarknet Contract |       |       |       |         |         |
+====================================================================+
| Function Name          | Min   | Max   | Avg   | Std Dev | # Calls |
|------------------------+-------+-------+-------+---------+---------|
| get_balance            | 13340 | 13340 | 13340 | 0       | 4       |
|------------------------+-------+-------+-------+---------+---------|
| increase_balance       | 25540 | 61240 | 37440 | 16829   | 3       |
╰------------------------+-------+-------+-------+---------+---------╯
```

</details>
<br>

Note that gas report data calculation ignores state changes, the cost of declared classes, Starknet OS overhead, L1 handler payload length and calldata payload length.

### Fuzzed Tests

While using the fuzzing feature additional gas statistics will be displayed:
```shell
[PASS] tests::fuzzing_test (runs: 256, l1_gas: {max: ~126, min: ~1, mean: ~65, std deviation: ~37}, l1_data_gas: {max: ~96, min: ~96, mean: ~96, std deviation: ~0}, l2_gas: {max: ~1888390, min: ~1852630, mean: ~1881888, std deviation: ~9322}})
```

> 📝 **Note**
>  
> Starknet-Foundry uses blob-based gas calculation formula in order to calculate gas usage. 
> For details on the exact formula, [see the docs](https://docs.starknet.io/learn/protocol/fees#overall-fee).

## Resources Estimation 

It is possible to enable more detailed breakdown of resources, on which the gas calculations are based on.
Depending on `--tracked-resource`, vm resources or sierra gas will be displayed (by default, Sierra gas is used).
To learn more about the tracked resource flag, see [--tracked-resource](../appendix/snforge/test.md#--tracked-resource).

### Usage
In order to run tests with this feature, run the `test` command with the `--detailed-resources` flag:

```shell
$ snforge test --detailed-resources
```

<details>
<summary>Output:</summary>

```shell
Collected 2 test(s) from hello_starknet package
Running 2 test(s) from tests/
[PASS] hello_starknet_integrationtest::test_contract::test_cannot_increase_balance_with_zero_value (l1_gas: ~0, l1_data_gas: ~96, l2_gas: ~406680)
        sierra gas: 406680
        syscalls: (CallContract: 2, StorageRead: 1, Deploy: 1)

[PASS] hello_starknet_integrationtest::test_contract::test_increase_balance (l1_gas: ~0, l1_data_gas: ~192, l2_gas: ~511980)
        sierra gas: 511980
        syscalls: (CallContract: 3, StorageRead: 3, Deploy: 1, StorageWrite: 1)

Running 0 test(s) from src/
Tests: 2 passed, 0 failed, 0 ignored, 0 filtered out
```
</details>
<br>

## Analyzing the results
Normally in transaction receipt (or block explorer transaction details), you would see some additional OS resources
that starknet-foundry does not include for a test (since it's not a normal transaction per-se):

#### Not included in the gas/resource estimations
- Fee transfer costs
- Transaction type related resources - in real Starknet additional cost depending on the transaction type (e.g., `Invoke`/`Declare`/`DeployAccount`) is added
- Declaration gas costs (CASM/Sierra bytecode or ABIs)
- Call validation gas costs (if you did not call `__validate__` endpoint explicitly)

#### Included in the gas/resource estimations
- Cost of syscalls (additional steps or builtins needed for syscalls execution)