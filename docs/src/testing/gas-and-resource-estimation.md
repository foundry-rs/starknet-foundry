# Gas and VM Resources Estimation

`snforge` supports gas and other VM resources estimation for each individual test case. 

It does not calculate the final transaction fee, for details on how fees are calculated, 
please refer to fee mechanism in [Starknet documentation](https://docs.starknet.io/architecture-and-concepts/network-architecture/fee-mechanism).

## Gas Estimation

### Single Test

When the test passes with no errors, estimated gas is displayed this way:
```shell
[PASS] tests::simple_test (gas: ~1)
```

This gas calculation is based on the estimated VM resources (that you can [display additionally on demand](#usage)), 
deployed contracts, storage updates, events and l1 <> l2 messages. 

### Fuzzed Tests

While using the fuzzing feature additional gas statistics will be displayed:
```shell
[PASS] tests::fuzzing_test (runs: 256, gas: {max: ~126, min: ~1, mean: ~65.00, std deviation: ~37.31})
```

> 📝 **Note**
>  
> Starknet-Foundry uses blob-based gas calculation formula in order to calculate gas usage. 
> For details on the exact formula, [see the docs](https://docs.starknet.io/architecture-and-concepts/network-architecture/fee-mechanism/#overall_fee_blob). 

## VM Resources estimation 

It is possible to enable more detailed breakdown of resources, on which the gas calculations are based on.

### Usage
In order to run tests with this feature, run the `test` command with the appropriate flag:

```shell
$ snforge test --detailed-resources
```

<details>
<summary>Output:</summary>

```shell
Collected 2 test(s) from hello_starknet package
Running 2 test(s) from tests/
[PASS] hello_starknet_integrationtest::test_contract::test_cannot_increase_balance_with_zero_value (gas: ~105)
        steps: 3405
        memory holes: 22
        builtins: (range_check: 77, pedersen: 7)
        syscalls: (CallContract: 2, StorageRead: 1, Deploy: 1)
        
[PASS] hello_starknet_integrationtest::test_contract::test_increase_balance (gas: ~172)
        steps: 4535
        memory holes: 15
        builtins: (range_check: 95, pedersen: 7)
        syscalls: (CallContract: 3, StorageRead: 3, Deploy: 1, StorageWrite: 1)
        
Running 0 test(s) from src/
Tests: 2 passed, 0 failed, 0 skipped, 0 ignored, 0 filtered out
```
</details>
<br>

This displays the resources used by the VM during the test execution.

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