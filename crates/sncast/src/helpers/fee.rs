use anyhow::{Context, Result, bail};
use clap::Args;
use conversions::{FromConv, TryFromConv, serde::deserialize::CairoDeserialize};
use starknet::core::types::BlockId;
use starknet::providers::Provider;
use starknet_types_core::felt::{Felt, NonZeroFelt};
use std::num::{NonZeroU64, NonZeroU128};

#[derive(Args, Debug, Clone)]
pub struct FeeArgs {
    /// Max fee for the transaction. If not provided, will be automatically estimated.
    #[clap(value_parser = parse_non_zero_felt, short, long)]
    pub max_fee: Option<NonZeroFelt>,

    /// Max gas amount. If not provided, will be automatically estimated.
    #[clap(value_parser = parse_non_zero_felt, long)]
    pub max_gas: Option<NonZeroFelt>,

    /// Max gas price in Fri. If not provided, will be automatically estimated.
    #[clap(value_parser = parse_non_zero_felt, long)]
    pub max_gas_unit_price: Option<NonZeroFelt>,
}

impl From<ScriptFeeSettings> for FeeArgs {
    fn from(script_fee_settings: ScriptFeeSettings) -> Self {
        let ScriptFeeSettings {
            max_fee,
            max_gas,
            max_gas_unit_price,
        } = script_fee_settings;
        Self {
            max_fee,
            max_gas: max_gas.map(NonZeroFelt::from_),
            max_gas_unit_price: max_gas_unit_price.map(NonZeroFelt::from_),
        }
    }
}

impl FeeArgs {
    #[allow(clippy::too_many_lines)]
    pub async fn try_into_fee_settings<P: Provider>(
        &self,
        provider: P,
        block_id: BlockId,
    ) -> Result<FeeSettings> {
        let settings = match (self.max_fee, self.max_gas, self.max_gas_unit_price) {
            (Some(_), Some(_), Some(_)) => {
                bail!(
                    "Passing all --max-fee, --max-gas and --max-gas-unit-price is conflicting. Please pass only two of them or less"
                )
            }
            (None, _, _) => FeeSettings {
                max_gas: self
                    .max_gas
                    .map(NonZeroU64::try_from_)
                    .transpose()
                    .map_err(anyhow::Error::msg)?,
                max_gas_unit_price: self
                    .max_gas_unit_price
                    .map(NonZeroU128::try_from_)
                    .transpose()
                    .map_err(anyhow::Error::msg)?,
            },
            (Some(max_fee), None, Some(max_gas_unit_price)) => {
                if max_fee < max_gas_unit_price {
                    bail!("--max-fee should be greater than or equal to --max-gas-unit-price");
                }

                let max_gas = NonZeroFelt::try_from(Felt::from(max_fee).floor_div(&max_gas_unit_price))
                        .unwrap_or_else(|_| unreachable!("Calculated max gas must be >= 1 because max_fee >= max_gas_unit_price ensures a positive result"));
                print_max_fee_conversion_info(max_fee, max_gas, max_gas_unit_price);
                FeeSettings {
                    max_gas: Some(NonZeroU64::try_from_(max_gas).map_err(anyhow::Error::msg)?),
                    max_gas_unit_price: Some(
                        NonZeroU128::try_from_(max_gas_unit_price).map_err(anyhow::Error::msg)?,
                    ),
                }
            }
            (Some(max_fee), Some(max_gas), None) => {
                if max_fee < max_gas {
                    bail!("--max-fee should be greater than or equal to --max-gas amount");
                }

                let max_gas_unit_price = NonZeroFelt::try_from(Felt::from(max_fee).floor_div(&max_gas))
                        .unwrap_or_else(|_| unreachable!("Calculated max gas unit price must be >= 1 because max_fee >= max_gas ensures a positive result"));
                print_max_fee_conversion_info(max_fee, max_gas, max_gas_unit_price);
                FeeSettings {
                    max_gas: Some(NonZeroU64::try_from_(max_gas).map_err(anyhow::Error::msg)?),
                    max_gas_unit_price: Some(
                        NonZeroU128::try_from_(max_gas_unit_price).map_err(anyhow::Error::msg)?,
                    ),
                }
            }
            (Some(max_fee), None, None) => {
                let max_gas_unit_price = NonZeroFelt::try_from(
                    provider
                        .get_block_with_tx_hashes(block_id)
                        .await?
                        .l1_gas_price()
                        .price_in_fri,
                )?;
                // TODO(#2852)
                let max_gas = NonZeroFelt::try_from(Felt::from(max_fee)
                            .floor_div(&max_gas_unit_price)).context("Calculated max-gas from provided --max-fee and the current network gas price is 0. Please increase --max-fee to obtain a positive gas amount")?;
                print_max_fee_conversion_info(max_fee, max_gas, max_gas_unit_price);
                FeeSettings {
                    max_gas: Some(NonZeroU64::try_from_(max_gas).map_err(anyhow::Error::msg)?),
                    max_gas_unit_price: Some(
                        NonZeroU128::try_from_(max_gas_unit_price).map_err(anyhow::Error::msg)?,
                    ),
                }
            }
        };

        Ok(settings)
    }
}

/// Struct used in `sncast script` for deserializing from cairo, `FeeSettings` can't be
/// used as it missing `max_fee`
#[derive(Debug, PartialEq, CairoDeserialize)]
pub struct ScriptFeeSettings {
    max_fee: Option<NonZeroFelt>,
    max_gas: Option<NonZeroU64>,
    max_gas_unit_price: Option<NonZeroU128>,
}

#[derive(Debug, PartialEq)]
pub struct FeeSettings {
    pub max_gas: Option<NonZeroU64>,
    pub max_gas_unit_price: Option<NonZeroU128>,
}

fn print_max_fee_conversion_info(
    max_fee: impl Into<Felt>,
    max_gas: impl Into<Felt>,
    max_gas_unit_price: impl Into<Felt>,
) {
    let max_fee: Felt = max_fee.into();
    let max_gas: Felt = max_gas.into();
    let max_gas_unit_price: Felt = max_gas_unit_price.into();
    println!(
        "Specifying '--max-fee' flag results in conversion to '--max-gas' and '--max-gas-unit-price' flags\nConverted {max_fee} max fee to {max_gas} max gas and {max_gas_unit_price} max gas unit price\n",
    );
}

fn parse_non_zero_felt(s: &str) -> Result<NonZeroFelt, String> {
    let felt: Felt = s.parse().map_err(|_| "Failed to parse value")?;
    felt.try_into()
        .map_err(|_| "Value should be greater than 0".to_string())
}
