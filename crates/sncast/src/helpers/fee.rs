use super::felt::felt_from_string;
use anyhow::{Result, ensure};
use clap::Args;
use configuration::Override;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;
use starknet_rust::core::types::FeeEstimate;
use starknet_types_core::felt::{Felt, NonZeroFelt};
use std::collections::HashMap;

#[derive(Args, Debug, Clone, Copy, Default)]
#[group(id = "fee_args", multiple = true)]
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

impl FeeArgs {
    #[must_use]
    pub fn resolve(&self, config: &FeeParams) -> Self {
        let mut resolved = *self;

        if self.max_fee.is_none() {
            resolved.l1_gas = self.l1_gas.or(config.l1_gas);
            resolved.l1_gas_price = self.l1_gas_price.or(config.l1_gas_price);
            resolved.l2_gas = self.l2_gas.or(config.l2_gas);
            resolved.l2_gas_price = self.l2_gas_price.or(config.l2_gas_price);
            resolved.l1_data_gas = self.l1_data_gas.or(config.l1_data_gas);
            resolved.l1_data_gas_price = self.l1_data_gas_price.or(config.l1_data_gas_price);
        }

        if !self.defines_tip() {
            resolved.tip = config.tip;
            resolved.estimate_tip = config.estimate_tip;
        }

        resolved
    }

    /// Whether these args define the tip group at all.
    fn defines_tip(&self) -> bool {
        self.tip.is_some() || self.estimate_tip
    }

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
            let fee_settings = FeeSettings::from(*self);
            Ok(fee_settings)
        }
    }
}

/// Fee settings that can be defined in `snfoundry.toml` under `fee-params`.
/// Every field is optional and CLI flags take precedence.
#[skip_serializing_none]
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub struct FeeParams {
    pub l1_gas: Option<u64>,
    pub l1_gas_price: Option<u128>,
    pub l2_gas: Option<u64>,
    pub l2_gas_price: Option<u128>,
    pub l1_data_gas: Option<u64>,
    pub l1_data_gas_price: Option<u128>,
    pub tip: Option<u64>,

    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub estimate_tip: bool,

    /// Additional data not captured by deserializer.
    #[doc(hidden)]
    #[serde(flatten, default, skip_serializing)]
    pub unknown_fields: HashMap<String, Value>,
}

impl FeeParams {
    /// Rejects mutually exclusive params, allows not fully specified params
    pub fn validate(&self) -> Result<()> {
        ensure!(
            !(self.tip.is_some() && self.estimate_tip),
            "`tip` cannot be used together with `estimate-tip`"
        );
        Ok(())
    }

    /// Whether this layer defines the tip group at all.
    fn defines_tip(&self) -> bool {
        self.tip.is_some() || self.estimate_tip
    }
}

impl Override for FeeParams {
    fn override_with(&self, other: FeeParams) -> FeeParams {
        let tip = if other.defines_tip() { &other } else { self };

        FeeParams {
            l1_gas: other.l1_gas.or(self.l1_gas),
            l1_gas_price: other.l1_gas_price.or(self.l1_gas_price),
            l2_gas: other.l2_gas.or(self.l2_gas),
            l2_gas_price: other.l2_gas_price.or(self.l2_gas_price),
            l1_data_gas: other.l1_data_gas.or(self.l1_data_gas),
            l1_data_gas_price: other.l1_data_gas_price.or(self.l1_data_gas_price),
            tip: tip.tip,
            estimate_tip: tip.estimate_tip,
            unknown_fields: HashMap::default(),
        }
    }
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
    pub fn with_resolved_tip(self, tip: Option<u64>, estimate_tip: bool) -> FeeSettings {
        let tip = if estimate_tip {
            None // If we leave it as None, the tip will be estimated before sending the transaction
        } else {
            Some(tip.unwrap_or(0)) // If a tip is not provided, set it to 0
        };

        FeeSettings { tip, ..self }
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
    let felt = felt_from_string(s).map_err(|e| e.to_string())?;
    felt.try_into()
        .map_err(|_| "Value should be greater than 0".to_string())
}

#[cfg(test)]
mod tests {
    use super::{FeeArgs, FeeParams, FeeSettings};
    use configuration::Override;
    use starknet_rust::core::types::FeeEstimate;
    use starknet_types_core::felt::{Felt, NonZeroFelt};
    use std::convert::TryFrom;

    fn non_zero_felt(value: u64) -> NonZeroFelt {
        NonZeroFelt::try_from(Felt::from(value)).unwrap()
    }

    #[test]
    fn test_fee_params_deserializes_kebab_case() {
        let toml_str = r"
            l1-gas = 1000
            l1-gas-price = 100000000000
            l2-gas = 5000000
            l2-gas-price = 100000000
            l1-data-gas = 1000
            l1-data-gas-price = 100000000
            estimate-tip = true
        ";

        let params: FeeParams = toml::from_str(toml_str).unwrap();
        assert_eq!(
            params,
            FeeParams {
                l1_gas: Some(1000),
                l1_gas_price: Some(100_000_000_000),
                l2_gas: Some(5_000_000),
                l2_gas_price: Some(100_000_000),
                l1_data_gas: Some(1000),
                l1_data_gas_price: Some(100_000_000),
                estimate_tip: true,
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_fee_params_collects_unknown_fields() {
        let toml_str = r"
            tip = 100
            l1_gas = 1000
            max-fee = 10000
        ";

        let params: FeeParams = toml::from_str(toml_str).unwrap();
        assert_eq!(params.tip, Some(100));
        assert_eq!(params.l1_gas, None);
        assert!(params.unknown_fields.contains_key("l1_gas"));
        // `max-fee` can only be passed as a CLI flag
        assert!(params.unknown_fields.contains_key("max-fee"));
    }

    #[test]
    fn test_fee_params_validation() {
        let conflicting = FeeParams {
            tip: Some(100),
            estimate_tip: true,
            ..Default::default()
        };
        assert!(conflicting.validate().is_err());

        let allowed = FeeParams {
            tip: Some(100),
            ..Default::default()
        };
        assert!(allowed.validate().is_ok());
    }

    #[test]
    fn test_fee_params_override_merges_bounds_per_field() {
        let global = FeeParams {
            l1_gas: Some(1000),
            l1_gas_price: Some(100),
            ..Default::default()
        };
        let local = FeeParams {
            l1_gas_price: Some(200),
            l2_gas: Some(2000),
            ..Default::default()
        };

        let merged = global.override_with(local);

        assert_eq!(merged.l1_gas, Some(1000));
        assert_eq!(merged.l1_gas_price, Some(200));
        assert_eq!(merged.l2_gas, Some(2000));
    }

    #[test]
    fn test_fee_params_override_keeps_groups_independent() {
        let global = FeeParams {
            l1_gas: Some(1000),
            tip: Some(1),
            ..Default::default()
        };
        let local = FeeParams {
            estimate_tip: true,
            ..Default::default()
        };

        let merged = global.override_with(local);

        assert_eq!(merged.l1_gas, Some(1000));
        assert_eq!(merged.tip, None);
        assert!(merged.estimate_tip);
    }

    #[test]
    fn test_fee_args_resolve_falls_back_to_config() {
        let config = FeeParams {
            l1_gas: Some(1000),
            l2_gas_price: Some(100),
            tip: Some(7),
            ..Default::default()
        };

        let resolved = FeeArgs::default().resolve(&config);

        assert_eq!(resolved.l1_gas, Some(1000));
        assert_eq!(resolved.l2_gas_price, Some(100));
        assert_eq!(resolved.tip, Some(7));
        assert!(!resolved.estimate_tip);
    }

    #[test]
    fn test_fee_args_resolve_cli_takes_precedence() {
        let config = FeeParams {
            l1_gas: Some(1000),
            tip: Some(7),
            ..Default::default()
        };
        let args = FeeArgs {
            l1_gas: Some(2000),
            ..Default::default()
        };

        let resolved = args.resolve(&config);

        assert_eq!(resolved.l1_gas, Some(2000));
        // The tip group is untouched by CLI bounds
        assert_eq!(resolved.tip, Some(7));
    }

    #[test]
    fn test_fee_args_resolve_merges_bounds_per_field() {
        let config = FeeParams {
            l1_gas: Some(1000),
            l1_gas_price: Some(100),
            ..Default::default()
        };
        let args = FeeArgs {
            l2_gas: Some(2000),
            l1_gas_price: Some(200),
            ..Default::default()
        };

        let resolved = args.resolve(&config);

        // Bounds only set in the config stay in force alongside the ones passed on the CLI
        assert_eq!(resolved.l1_gas, Some(1000));
        assert_eq!(resolved.l1_gas_price, Some(200));
        assert_eq!(resolved.l2_gas, Some(2000));
    }

    #[test]
    fn test_fee_args_resolve_cli_max_fee_drops_config_bounds() {
        let config = FeeParams {
            l1_gas: Some(1000),
            l1_gas_price: Some(100),
            ..Default::default()
        };
        let args = FeeArgs {
            max_fee: Some(non_zero_felt(999)),
            ..Default::default()
        };

        let resolved = args.resolve(&config);

        assert_eq!(resolved.max_fee, Some(non_zero_felt(999)));
        assert_eq!(resolved.l1_gas, None);
        assert_eq!(resolved.l1_gas_price, None);
    }

    #[test]
    fn test_fee_args_resolve_estimate_tip_flag_overrides_config_tip() {
        let config = FeeParams {
            tip: Some(7),
            ..Default::default()
        };
        let args = FeeArgs {
            estimate_tip: true,
            ..Default::default()
        };

        let resolved = args.resolve(&config);

        assert_eq!(resolved.tip, None);
        assert!(resolved.estimate_tip);
    }

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
