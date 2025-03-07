use anyhow::{Result, ensure};
use clap::Args;
use conversions::{TryIntoConv, serde::deserialize::CairoDeserialize};
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
    pub l1_gas_unit_price: Option<Felt>,

    /// Max L2 gas amount. If not provided, will be automatically estimated.
    #[clap(long)]
    pub l2_gas: Option<Felt>,

    /// Max L2 gas price in Fri. If not provided, will be automatically estimated.
    #[clap(long)]
    pub l2_gas_unit_price: Option<Felt>,

    /// Max L1 data gas amount. If not provided, will be automatically estimated.
    #[clap(long)]
    pub l1_data_gas: Option<Felt>,

    /// Max L1 data gas price in Fri. If not provided, will be automatically estimated.
    #[clap(long)]
    pub l1_data_gas_unit_price: Option<Felt>,
}

impl From<ScriptFeeSettings> for FeeArgs {
    fn from(script_fee_settings: ScriptFeeSettings) -> Self {
        let ScriptFeeSettings {
            max_fee,
            l1_gas,
            l1_gas_unit_price,
            l2_gas,
            l2_gas_unit_price,
            l1_data_gas,
            l1_data_gas_unit_price,
        } = script_fee_settings;
        Self {
            max_fee,
            l1_gas: l1_gas.map(|value| Felt::from(value)),
            l1_gas_unit_price: l1_gas_unit_price.map(|value| Felt::from(value)),
            l2_gas: l2_gas.map(|value| Felt::from(value)),
            l2_gas_unit_price: l2_gas_unit_price.map(|value| Felt::from(value)),
            l1_data_gas: l1_data_gas.map(|value| Felt::from(value)),
            l1_data_gas_unit_price: l1_data_gas_unit_price.map(|value| Felt::from(value)),
        }
    }
}

impl FeeArgs {
    pub async fn try_into_fee_settings<P: Provider>(
        &self,
        provider: P,
        block_id: BlockId,
    ) -> Result<FeeSettings> {
        let mut fee_settings = FeeSettings {
            l1_gas: self.l1_gas.map_or(None, |value| {
                Some(u64::try_from(value).expect("Failed to convert l1_gas"))
            }),
            l1_gas_unit_price: self.l1_gas_unit_price.map_or(None, |value| {
                Some(u128::try_from(value).expect("Failed to convert l1_gas_unit_price"))
            }),
            l2_gas: self.l2_gas.map_or(None, |value| {
                Some(u64::try_from(value).expect("Failed to convert l2_gas"))
            }),
            l2_gas_unit_price: self.l2_gas_unit_price.map_or(None, |value| {
                Some(u128::try_from(value).expect("Failed to convert l2_gas_unit_price"))
            }),
            l1_data_gas: self.l1_data_gas.map_or(None, |value| {
                Some(u64::try_from(value).expect("Failed to convert l1_data_gas"))
            }),
            l1_data_gas_unit_price: self.l1_data_gas_unit_price.map_or(None, |value| {
                Some(u128::try_from(value).expect("Failed to convert l1_data_gas_unit_price"))
            }),
        };

        if fee_settings.l1_gas_unit_price.is_none()
            || fee_settings.l2_gas_unit_price.is_none()
            || fee_settings.l1_data_gas_unit_price.is_none()
        {
            let block = provider.get_block_with_tx_hashes(block_id).await?;

            fee_settings.l1_gas_unit_price.get_or_insert(
                block
                    .l1_gas_price()
                    .price_in_fri
                    .try_into_()
                    .expect("Failed to convert l1_gas_price from Felt to u128"),
            );
            fee_settings.l2_gas_unit_price.get_or_insert(
                block
                    .l2_gas_price()
                    .price_in_fri
                    .try_into_()
                    .expect("Failed to convert l2_gas_price from Felt to u128"),
            );
            fee_settings.l1_data_gas_unit_price.get_or_insert(
                block
                    .l1_data_gas_price()
                    .price_in_fri
                    .try_into_()
                    .expect("Failed to convert l1_data_gas_price from Felt to u128"),
            );
        }

        let calculated_total_fee = fee_settings.l1_gas.map_or(0, |l1_gas| {
            (l1_gas as u128)
                .checked_mul(fee_settings.l1_gas_unit_price.unwrap_or(0))
                .expect("Failed to calculate total fee")
        }) + fee_settings.l2_gas.map_or(0, |l2_gas| {
            (l2_gas as u128)
                .checked_mul(fee_settings.l2_gas_unit_price.unwrap_or(0))
                .expect("Failed to calculate total fee")
        }) + fee_settings.l1_data_gas.map_or(0, |l1_data_gas| {
            (l1_data_gas as u128)
                .checked_mul(fee_settings.l1_data_gas_unit_price.unwrap_or(0))
                .expect("Failed to calculate total fee")
        });

        if let Some(max_fee) = self.max_fee {
            ensure!(
                calculated_total_fee
                    <= u128::try_from(Felt::from(max_fee))
                        .expect("Failed to convert max_fee from NonZeroFelt to u128"),
                "Calculated total fee exceeds the provided max fee. Calculated total fee: {}, Max fee: {}",
                calculated_total_fee,
                Felt::from(max_fee)
            );
        }

        Ok(fee_settings)
    }
}

/// Struct used in `sncast script` for deserializing from cairo, `FeeSettings` can't be
/// used as it missing `max_fee`
#[derive(Debug, PartialEq, CairoDeserialize)]
pub struct ScriptFeeSettings {
    max_fee: Option<NonZeroFelt>,
    l1_gas: Option<u64>,
    l1_gas_unit_price: Option<u128>,
    l2_gas: Option<u64>,
    l2_gas_unit_price: Option<u128>,
    l1_data_gas: Option<u64>,
    l1_data_gas_unit_price: Option<u128>,
}

#[derive(Debug, PartialEq)]
pub struct FeeSettings {
    pub l1_gas: Option<u64>,
    pub l1_gas_unit_price: Option<u128>,
    pub l2_gas: Option<u64>,
    pub l2_gas_unit_price: Option<u128>,
    pub l1_data_gas: Option<u64>,
    pub l1_data_gas_unit_price: Option<u128>,
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
