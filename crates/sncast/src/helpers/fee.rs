use anyhow::{Result, ensure};
use clap::Args;
use conversions::serde::deserialize::CairoDeserialize;
use starknet::core::types::FeeEstimate;
use starknet_types_core::felt::{Felt, NonZeroFelt};

#[derive(Args, Debug, Clone)]
pub struct FeeArgs {
    /// Max fee for the transaction. If not provided, will be automatically estimated.
    #[arg(value_parser = parse_non_zero_felt, short, long, conflicts_with_all = ["l1_gas", "l1_gas_price", "l2_gas", "l2_gas_price", "l1_data_gas", "l1_data_gas_price"])]
    pub max_fee: Option<NonZeroFelt>,

    /// Max L1 gas amount. If not provided, will be automatically estimated.
    #[arg(long)]
    pub l1_gas: Option<u64>,

    /// Max L1 gas price in Fri. If not provided, will be automatically estimated.
    #[arg(long)]
    pub l1_gas_price: Option<u128>,

    /// Max L2 gas amount. If not provided, will be automatically estimated.
    #[arg(long)]
    pub l2_gas: Option<u64>,

    /// Max L2 gas price in Fri. If not provided, will be automatically estimated.
    #[arg(long)]
    pub l2_gas_price: Option<u128>,

    /// Max L1 data gas amount. If not provided, will be automatically estimated.
    #[arg(long)]
    pub l1_data_gas: Option<u64>,

    /// Max L1 data gas price in Fri. If not provided, will be automatically estimated.
    #[arg(long)]
    pub l1_data_gas_price: Option<u128>,

    /// Tip for the transaction. Defaults to 0 unless `--estimate-tip` is used.
    #[arg(long, conflicts_with = "estimate_tip")]
    pub tip: Option<u64>,

    /// If passed, an estimated tip will be added to pay for the transaction.
    #[arg(long)]
    pub estimate_tip: bool,
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
            l1_gas,
            l1_gas_price,
            l2_gas,
            l2_gas_price,
            l1_data_gas,
            l1_data_gas_price,
            tip: Some(0),
            estimate_tip: false,
        }
    }
}

impl FeeArgs {
    pub fn try_into_fee_settings(&self, fee_estimate: Option<&FeeEstimate>) -> Result<FeeSettings> {
        // If some resource bounds values are lacking, starknet-rs will estimate them automatically
        // but in case someone passes --max-fee flag, we need to make estimation on our own
        // to check if the fee estimate isn't higher than provided max fee
        if let Some(max_fee) = self.max_fee {
            let fee_estimate =
                fee_estimate.expect("Fee estimate must be passed when max_fee is provided");

            ensure!(
                Felt::from(max_fee) >= Felt::from(fee_estimate.overall_fee),
                "Estimated fee ({}) is higher than provided max fee ({})",
                fee_estimate.overall_fee,
                Felt::from(max_fee)
            );

            let fee_settings = FeeSettings::try_from(fee_estimate.clone())
                .expect("Failed to convert FeeEstimate to FeeSettings")
                .with_resolved_tip(self.tip, self.estimate_tip);

            Ok(fee_settings)
        } else {
            let fee_settings = FeeSettings::from(self.clone());
            Ok(fee_settings)
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
    pub tip: Option<u64>,
}

impl FeeSettings {
    #[must_use]
    pub fn with_resolved_tip(&self, tip: Option<u64>, estimate_tip: bool) -> FeeSettings {
        let tip = if estimate_tip {
            None // If we leave it as None, the tip will be estimated before sending the transaction
        } else {
            Some(tip.unwrap_or(0)) // If a tip is not provided, set it to 0
        };

        FeeSettings { tip, ..*self }
    }
}

impl TryFrom<FeeEstimate> for FeeSettings {
    type Error = anyhow::Error;
    fn try_from(fee_estimate: FeeEstimate) -> Result<FeeSettings, anyhow::Error> {
        Ok(FeeSettings {
            l1_gas: Some(fee_estimate.l1_gas_consumed),
            l1_gas_price: Some(fee_estimate.l1_gas_price),
            l2_gas: Some(fee_estimate.l2_gas_consumed),
            l2_gas_price: Some(fee_estimate.l2_gas_price),
            l1_data_gas: Some(fee_estimate.l1_data_gas_consumed),
            l1_data_gas_price: Some(fee_estimate.l1_data_gas_price),
            tip: None,
        })
    }
}

impl From<FeeArgs> for FeeSettings {
    fn from(fee_args: FeeArgs) -> FeeSettings {
        FeeSettings {
            l1_gas: fee_args.l1_gas,
            l1_gas_price: fee_args.l1_gas_price,
            l2_gas: fee_args.l2_gas,
            l2_gas_price: fee_args.l2_gas_price,
            l1_data_gas: fee_args.l1_data_gas,
            l1_data_gas_price: fee_args.l1_data_gas_price,
            tip: None,
        }
        .with_resolved_tip(fee_args.tip, fee_args.estimate_tip)
    }
}

fn parse_non_zero_felt(s: &str) -> Result<NonZeroFelt, String> {
    let felt: Felt = s.parse().map_err(|_| "Failed to parse value")?;
    felt.try_into()
        .map_err(|_| "Value should be greater than 0".to_string())
}

#[cfg(test)]
mod tests {
    use super::FeeSettings;
    use starknet::core::types::FeeEstimate;
    use std::convert::TryFrom;

    #[tokio::test]
    async fn test_from_fee_estimate() {
        let mock_fee_estimate = FeeEstimate {
            l1_gas_consumed: 1,
            l1_gas_price: 2,
            l2_gas_consumed: 3,
            l2_gas_price: 4,
            l1_data_gas_consumed: 5,
            l1_data_gas_price: 6,
            overall_fee: 44,
        };
        let settings = FeeSettings::try_from(mock_fee_estimate).unwrap();

        assert_eq!(
            settings,
            FeeSettings {
                l1_gas: Some(1),
                l1_gas_price: Some(2),
                l2_gas: Some(3),
                l2_gas_price: Some(4),
                l1_data_gas: Some(5),
                l1_data_gas_price: Some(6),
                tip: None,
            }
        );
    }
}
