# Gas and VM Resources Estimation

`snforge` supports gas and other VM resources estimation for each individual test case. 

It does not calculate the final transaction fee, for details on how fees are calculated, 
please refer to fee mechanism in [Starknet documentation](https://docs.starknet.io/documentation/architecture_and_concepts/Network_Architecture/fee-mechanism).

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
> For details on the exact formula, [see the docs](https://docs.starknet.io/documentation/architecture_and_concepts/Network_Architecture/fee-mechanism/#overall_fee_blob). 

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
...
[PASS] package_name::tests::resources (gas: ~2213)
        steps: 881
        memory holes: 36
        builtins: ("range_check_builtin": 32)
        syscalls: (StorageWrite: 1, StorageRead: 1, CallContract: 1)
...
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