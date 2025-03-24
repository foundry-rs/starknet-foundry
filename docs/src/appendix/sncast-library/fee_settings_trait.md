# `FeeSettingsTrait`

```rust
#[generate_trait]
pub impl FeeSettingsImpl of FeeSettingsTrait {
    /// Sets transaction resource bounds with specified gas values.
    fn resource_bounds(
        l1_gas: u64,
        l1_gas_price: u128,
        l2_gas: u64,
        l2_gas_price: u128,
        l1_data_gas: u64,
        l1_data_gas_price: u128
    ) -> FeeSettings;

    /// Ensures that total resource bounds of transaction execution won't exceed the given value.
    fn max_fee(max_fee: felt252) -> FeeSettings;

    /// Performs an automatic estimation of the resrouce bounds.
    fn estimate() -> FeeSettings;
}
```