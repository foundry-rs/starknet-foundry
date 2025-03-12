use anyhow::{Result, ensure};
use clap::Args;
use conversions::serde::deserialize::CairoDeserialize;
use starknet::core::types::FeeEstimate;
use starknet_types_core::felt::{Felt, NonZeroFelt};

#[derive(Args, Debug, Clone)]
pub struct FeeArgs {
    /// Max fee for the transaction. If not provided, will be automatically estimated.
    #[clap(value_parser = parse_non_zero_felt, short, long)]
    pub max_fee: Option<NonZeroFelt>,

    /// Max L1 gas amount. If not provided, will be automatically estimated.
    #[clap(long)]
    pub l1_gas: Option<Felt>,

    /// Max L1 gas price in Fri. If not provided, will be automatically estimated.
    #[clap(long)]
    pub l1_gas_price: Option<Felt>,

    /// Max L2 gas amount. If not provided, will be automatically estimated.
    #[clap(long)]
    pub l2_gas: Option<Felt>,

    /// Max L2 gas price in Fri. If not provided, will be automatically estimated.
    #[clap(long)]
    pub l2_gas_price: Option<Felt>,

    /// Max L1 data gas amount. If not provided, will be automatically estimated.
    #[clap(long)]
    pub l1_data_gas: Option<Felt>,

    /// Max L1 data gas price in Fri. If not provided, will be automatically estimated.
    #[clap(long)]
    pub l1_data_gas_price: Option<Felt>,
}

impl From<ScriptFeeSettings> for FeeArgs {
    fn from(script_fee_settings: ScriptFeeSettings) -> Self {
        let ScriptFeeSettings {
            max_fee,
            l1_gas,
            l1_gas_price,
            l2_gas,
            l2_gas_price,
            l1_data_gas,
            l1_data_gas_price,
        } = script_fee_settings;
        Self {
            max_fee,
            l1_gas: l1_gas.map(Felt::from),
            l1_gas_price: l1_gas_price.map(Felt::from),
            l2_gas: l2_gas.map(Felt::from),
            l2_gas_price: l2_gas_price.map(Felt::from),
            l1_data_gas: l1_data_gas.map(Felt::from),
            l1_data_gas_price: l1_data_gas_price.map(Felt::from),
        }
    }
}

impl FeeArgs {
    pub fn try_into_fee_settings(&self, fee_estimate: &Option<FeeEstimate>) -> Result<FeeSettings> {
        if let Some(max_fee) = self.max_fee {
            self.ensure_max_fee_greater_than_gas_values()?;

            ensure!(
                fee_estimate.is_some(),
                "Fee estimate is required when passing max fee is provided"
            );

            ensure!(
                Felt::from(max_fee) >= fee_estimate.as_ref().unwrap().overall_fee,
                "Estimated fee ({}) is higher than provided max fee ({})",
                fee_estimate.as_ref().unwrap().overall_fee,
                Felt::from(max_fee)
            );

            let fee_settings = FeeSettings {
                l1_gas: Some(
                    u64::try_from(fee_estimate.as_ref().unwrap().l1_gas_consumed)
                        .expect("Failed to convert l1_gas"),
                ),
                l1_gas_price: Some(
                    u128::try_from(fee_estimate.as_ref().unwrap().l1_gas_price)
                        .expect("Failed to convert l1_gas_price"),
                ),
                l2_gas: Some(
                    u64::try_from(fee_estimate.as_ref().unwrap().l2_gas_consumed)
                        .expect("Failed to convert l2_gas"),
                ),
                l2_gas_price: Some(
                    u128::try_from(fee_estimate.as_ref().unwrap().l2_gas_price)
                        .expect("Failed to convert l2_gas_price"),
                ),
                l1_data_gas: Some(
                    u64::try_from(fee_estimate.as_ref().unwrap().l1_data_gas_consumed)
                        .expect("Failed to convert l1_data_gas"),
                ),
                l1_data_gas_price: Some(
                    u128::try_from(fee_estimate.as_ref().unwrap().l1_data_gas_price)
                        .expect("Failed to convert l1_data_gas_price"),
                ),
            };
            Ok(fee_settings)
        } else {
            let fee_settings = FeeSettings {
                l1_gas: self
                    .l1_gas
                    .map(|val| u64::try_from(val).expect("Failed to convert l1_gas")),
                l1_gas_price: self
                    .l1_gas_price
                    .map(|val| u128::try_from(val).expect("Failed to convert l1_gas_price")),
                l2_gas: self
                    .l2_gas
                    .map(|val| u64::try_from(val).expect("Failed to convert l2_gas")),
                l2_gas_price: self
                    .l2_gas_price
                    .map(|val| u128::try_from(val).expect("Failed to convert l2_gas_price")),
                l1_data_gas: self
                    .l1_data_gas
                    .map(|val| u64::try_from(val).expect("Failed to convert l1_data_gas")),
                l1_data_gas_price: self
                    .l1_data_gas_price
                    .map(|val| u128::try_from(val).expect("Failed to convert l1_data_gas_price")),
            };
            Ok(fee_settings)
        }
    }

    fn ensure_max_fee_greater_than_gas_values(&self) -> Result<()> {
        if let Some(max_fee) = self.max_fee {
            let max_fee_felt = Felt::from(max_fee);

            let gas_checks = [
                (self.l1_gas, "--l1-gas"),
                (self.l1_gas_price, "--l1-gas-price"),
                (self.l2_gas, "--l2-gas"),
                (self.l2_gas_price, "--l2-gas-price"),
                (self.l1_data_gas, "--l1-data-gas"),
                (self.l1_data_gas_price, "--l1-data-gas-price"),
            ];

            for (gas_value, flag) in gas_checks.iter() {
                ensure!(
                    max_fee_felt >= gas_value.unwrap_or_default(),
                    "--max-fee should be greater than or equal to {}",
                    *flag
                );
            }
        }
        Ok(())
    }
}

/// Struct used in `sncast script` for deserializing from cairo, `FeeSettings` can't be
/// used as it missing `max_fee`
#[derive(Debug, PartialEq, CairoDeserialize)]
pub struct ScriptFeeSettings {
    max_fee: Option<NonZeroFelt>,
    l1_gas: Option<u64>,
    l1_gas_price: Option<u128>,
    l2_gas: Option<u64>,
    l2_gas_price: Option<u128>,
    l1_data_gas: Option<u64>,
    l1_data_gas_price: Option<u128>,
}

#[derive(Debug, PartialEq)]
pub struct FeeSettings {
    pub l1_gas: Option<u64>,
    pub l1_gas_price: Option<u128>,
    pub l2_gas: Option<u64>,
    pub l2_gas_price: Option<u128>,
    pub l1_data_gas: Option<u64>,
    pub l1_data_gas_price: Option<u128>,
}

fn parse_non_zero_felt(s: &str) -> Result<NonZeroFelt, String> {
    let felt: Felt = s.parse().map_err(|_| "Failed to parse value")?;
    felt.try_into()
        .map_err(|_| "Value should be greater than 0".to_string())
}
