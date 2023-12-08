# Gas estimation

Transaction gas cost is always important for the users. Developers need to know if their contracts are well-optimised
and gas-efficient.
Forge supports gas estimation for each test case. All computations are based on the official
[Starknet docs](https://docs.starknet.io/documentation/architecture_and_concepts/Network_Architecture/fee-mechanism/#how_much_fee_is_charged_high_level_overview).

> ⚠️ In the next releases, gas estimation will be improved to include [onchain data cost](https://docs.starknet.io/documentation/architecture_and_concepts/Network_Architecture/fee-mechanism/#_on_chain_data)  ⚠️

## Displaying estimated gas

When the test passes with no errors, estimated gas is displayed this way:
```shell
[PASS] tests::simple_test, gas: ~0.1
```

## Calculating gas from VM resources

[General case](https://docs.starknet.io/documentation/architecture_and_concepts/Network_Architecture/fee-mechanism/#general_case)
of the docs is focused on gas used by running Cairo code. It is mentioned that the gas cost is connected to
the most costly component. What does it mean?

Let's assume we have a function which uses 100 `Cairo steps`, 12 `range check builtins` and one `keccak builtin`.
Table (from the `General case`) has gas cost defined for all builtins. In our case:
- `Cairo step` - 0.01
- `range check builtin` - 0.16
- `keccak builtin` - 20.48

Multiplication of those values gives us a gas cost for each component:
- `Cairo steps` - 100 * 0.01 = 1
- `range check builtins` - 12 * 0.16 = 1.92
- `keccak builtin` - 1 * 20.48 = 20.48

We should remember that only the most expensive factor will be taken into account, so our overall gas cost is `20.48`.

## Estimated gas vs Starknet transaction fee

Transaction fees are a product of the `gas usage` and `gas price`. Although, fees are based on the `gas usage` it is
impossible to accurately predict the fee because cost of some parts varies depending on the block. However, estimated gas
can give you good insight into the final transaction fee.