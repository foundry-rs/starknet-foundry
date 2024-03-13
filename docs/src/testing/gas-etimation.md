# Gas Estimation

Transaction gas cost is always important for the users. Developers need to know if their contracts are well-optimised
and gas-efficient.
`snforge` supports gas estimation for each test case. 

## Displaying Estimated Gas

### Single Test

When the test passes with no errors, estimated gas is displayed this way:
```shell
[PASS] tests::simple_test (gas: ~1)
```

### Fuzzed Tests

While using the fuzzing feature additional gas statistics will be displayed:
```shell
[PASS] tests::fuzzing_test (runs: 256, gas: {max: ~126, min: ~1, mean: ~65.00, std deviation: ~37.31})
```

> 📝 **Note**
> 
> Estimated gas will always be rounded up to the next integer.

For further details on how fees are calculated, please refer to fee mechanism 
[Starknet documentation](https://docs.starknet.io/documentation/architecture_and_concepts/Network_Architecture/fee-mechanism).


## Estimated Gas vs Starknet Transaction Fee

Transaction fees are a product of the `gas usage` and `gas price`. Although, fees are based on the `gas usage` it is
impossible to accurately predict the fee because cost of `single 32-byte word` varies depending on the block. 
However, estimated gas can give you good insight into the final transaction fee.
Remember that `gas_price` will vary between different blocks.
