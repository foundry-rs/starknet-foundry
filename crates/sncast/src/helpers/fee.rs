use anyhow::{Result, bail, ensure};
use clap::Args;
use conversions::serde::deserialize::CairoDeserialize;
use starknet::core::types::BlockId;
use starknet::providers::Provider;
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
            l1_gas: l1_gas.map(|value| Felt::from(value)),
            l1_gas_price: l1_gas_price.map(|value| Felt::from(value)),
            l2_gas: l2_gas.map(|value| Felt::from(value)),
            l2_gas_price: l2_gas_price.map(|value| Felt::from(value)),
            l1_data_gas: l1_data_gas.map(|value| Felt::from(value)),
            l1_data_gas_price: l1_data_gas_price.map(|value| Felt::from(value)),
        }
    }
}

impl FeeArgs {
    fn ensure_max_fee_greater_or_equal_than_resource_bounds_values(&self) -> Result<()> {
        if let Some(max_fee) = self.max_fee {
            if let Some(l1_gas) = self.l1_gas {
                ensure!(
                    Felt::from(max_fee) >= l1_gas,
                    "--max-fee should be greater than or equal to --l1-gas"
                );
            }
            if let Some(l1_gas_price) = self.l1_gas_price {
                ensure!(
                    Felt::from(max_fee) >= l1_gas_price,
                    "--max-fee should be greater than or equal to --l1-gas-price"
                );
            }
            if let Some(l2_gas) = self.l2_gas {
                ensure!(
                    Felt::from(max_fee) >= l2_gas,
                    "--max-fee should be greater than or equal to --l2-gas"
                );
            }
            if let Some(l2_gas_price) = self.l2_gas_price {
                ensure!(
                    Felt::from(max_fee) >= l2_gas_price,
                    "--max-fee should be greater than or equal to --l2-gas-price"
                );
            }
            if let Some(l1_data_gas) = self.l1_data_gas {
                ensure!(
                    Felt::from(max_fee) >= l1_data_gas,
                    "--max-fee should be greater than or equal to --l1-data-gas"
                );
            }
            if let Some(l1_data_gas_price) = self.l1_data_gas_price {
                ensure!(
                    Felt::from(max_fee) >= l1_data_gas_price,
                    "--max-fee should be greater than or equal to --l1-data-gas-price"
                );
            }
            Ok(())
        } else {
            Ok(())
        }
    }

    pub async fn try_into_fee_settings<P: Provider>(
        &self,
        provider: P,
        block_id: BlockId,
    ) -> Result<FeeSettings> {
        match (
            self.max_fee,
            self.l1_gas,
            self.l1_gas_price,
            self.l2_gas,
            self.l2_gas_price,
            self.l1_data_gas,
            self.l1_data_gas_price,
        ) {
            (Some(_), Some(_), Some(_), Some(_), Some(_), Some(_), Some(_)) => {
                bail!(
                    "Passing all --max-fee, --l1-gas, --l1-gas-price, --l2-gas, --l2-gas-price, --l1-data-gas and --l1-data-gas-price is conflicting. Please pass only two of them or less"
                );
            }
            _ => {
                self.ensure_max_fee_greater_or_equal_than_resource_bounds_values()?;
                let fee_settings = FeeSettings {
                    // FIXME: Either remove or restore this logic
                    // (None, _, _, _, _, _, _) => Ok(FeeSettings {
                    l1_gas: self
                        .l1_gas
                        .map(|val| u64::try_from(val).expect("Failed to convert l1_gas")),
                    l1_gas_price: self
                        .l1_data_gas_price
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
                    l1_data_gas_price: self.l1_data_gas_price.map(|val| {
                        u128::try_from(val).expect("Failed to convert l1_data_gas_price")
                    }),
                };
                Ok(fee_settings)
            } // FIXME: Either remove or restore this logic
              // (Some(_), _, _, _, _, _, _) => {
              //     let block = provider.get_block_with_tx_hashes(block_id).await?;
              //     let mut fee_settings = FeeSettings {
              //         l1_gas: self.l1_gas.map_or(None, |value| {
              //             Some(u64::try_from(value).expect("Failed to convert l1_gas"))
              //         }),
              //         l1_gas_price: self.l1_gas_price.map_or(None, |value| {
              //             Some(u128::try_from(value).expect("Failed to convert l1_gas_price"))
              //         }),
              //         l2_gas: self.l2_gas.map_or(None, |value| {
              //             Some(u64::try_from(value).expect("Failed to convert l2_gas"))
              //         }),
              //         l2_gas_price: self.l2_gas_price.map_or(None, |value| {
              //             Some(u128::try_from(value).expect("Failed to convert l2_gas_price"))
              //         }),
              //         l1_data_gas: self.l1_data_gas.map_or(None, |value| {
              //             Some(u64::try_from(value).expect("Failed to convert l1_data_gas"))
              //         }),
              //         l1_data_gas_price: self.l1_data_gas_price.map_or(None, |value| {
              //             Some(u128::try_from(value).expect("Failed to convert l1_data_gas_price"))
              //         }),
              //     };

              //     fee_settings.l1_gas_price.get_or_insert(
              //         block
              //             .l1_gas_price()
              //             .price_in_fri
              //             .try_into_()
              //             .expect("Failed to convert fetched l1_gas_price"),
              //     );
              //     fee_settings.l2_gas_price.get_or_insert(
              //         block
              //             .l2_gas_price()
              //             .price_in_fri
              //             .try_into_()
              //             .expect("Failed to convert fetched l2_gas_price"),
              //     );
              //     fee_settings.l1_data_gas_price.get_or_insert(
              //         block
              //             .l1_data_gas_price()
              //             .price_in_fri
              //             .try_into_()
              //             .expect("Failed to convert fetched l1_data_gas_price"),
              //     );

              //     self.ensure_max_fee_greater_or_equal_than_resource_bounds_values()?;

              //     return Ok(fee_settings);
              // }
        }
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

// fn print_max_fee_conversion_info(
//     max_fee: impl Into<Felt>,
//     max_gas: impl Into<Felt>,
//     max_gas_unit_price: impl Into<Felt>,
// ) {
//     let max_fee: Felt = max_fee.into();
//     let max_gas: Felt = max_gas.into();
//     let max_gas_unit_price: Felt = max_gas_unit_price.into();
//     println!(
//         "Specifying '--max-fee' flag results in conversion to '--max-gas' and '--max-gas-unit-price' flags\nConverted {max_fee} max fee to {max_gas} max gas and {max_gas_unit_price} max gas unit price\n",
//     );
// }

fn parse_non_zero_felt(s: &str) -> Result<NonZeroFelt, String> {
    let felt: Felt = s.parse().map_err(|_| "Failed to parse value")?;
    felt.try_into()
        .map_err(|_| "Value should be greater than 0".to_string())
}
