# Gas Estimation

Transaction gas cost is always important for the users. Developers need to know if their contracts are well-optimised
and gas-efficient.
Forge supports gas estimation for each test case. All computations are based on the official
[Starknet docs](https://docs.starknet.io/documentation/architecture_and_concepts/Network_Architecture/fee-mechanism/#how_much_fee_is_charged_high_level_overview).

## Displaying Estimated Gas

When the test passes with no errors, estimated gas is displayed this way:
```shell
[PASS] tests::simple_test, gas: ~1
```

> 📝 **Note**
> 
> Estimated gas will always be rounded up to the next integer.

## Calculating Gas

### From Used VM Resources

[Starknet documentation](https://docs.starknet.io/documentation/architecture_and_concepts/Network_Architecture/fee-mechanism/#general_case)
mentions that the gas cost is connected to the most costly component. What does it mean?

Let's assume we have a function which uses 100 `Cairo steps`, 12 `range check builtins` and one `keccak builtin`.
[Table](https://docs.starknet.io/documentation/architecture_and_concepts/Network_Architecture/fee-mechanism/#general_case:~:text=and%20builtins%20used.-,The%20weights%20are%3A,-Step) (from the `Starknet documentation`) has gas cost defined
for all builtins.
In our case:
- `Cairo step` - 0.01
- `range check builtin` - 0.16
- `keccak builtin` - 20.48

Multiplication of those values gives us a gas cost for each component:
- `Cairo steps` - 100 * 0.01 = 1
- `range check builtins` - 12 * 0.16 = 1.92
- `keccak builtin` - 1 * 20.48 = 20.48

We should remember that only the most expensive factor will be taken into account, so our overall gas cost is `20.48`.

### From Used Onchain Data

[Starknet documentation](https://docs.starknet.io/documentation/architecture_and_concepts/Network_Architecture/fee-mechanism/#_on_chain_data)
splits onchain data into three parts:
- `storage updates`
- `L2 -> L1 messages`
- `deployed contracts`

Let's calculate gas based on those operations:
- contract deployment
- one storage write
- one L2 -> L1 message (with `[1, 2, 3]` as a payload)

We assume the cost of a single 32-byte word is 612 gas units. Said that we can estimate the cost of previous operations:
- contract deployment - 3 * 612 = 1836
- storage write - 2 * 612 = 1224
- one L2 -> L1 message - (3 + 3) * 612 = 3672 (read more about L2 -> L1 message cost
  [here](https://docs.starknet.io/documentation/architecture_and_concepts/Network_Architecture/fee-mechanism/#l2l1_messages))

This time we sum all calculated values because all of them will be kept onchain. Overall onchain data gas cost is `6732`.

## Estimated Gas vs Starknet Transaction Fee

Transaction fees are a product of the `gas usage` and `gas price`. Although, fees are based on the `gas usage` it is
impossible to accurately predict the fee because cost of `single 32-byte word` varies depending on the block. 
However, estimated gas can give you good insight into the final transaction fee.
Remember that `gas_price` will vary between different blocks.
