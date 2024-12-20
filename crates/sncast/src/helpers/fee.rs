use anyhow::{bail, ensure, Context, Error, Result};
use clap::{Args, ValueEnum};
use conversions::{serde::deserialize::CairoDeserialize, FromConv, TryFromConv};
use shared::print::print_as_warning;
use starknet::core::types::BlockId;
use starknet::providers::Provider;
use starknet_types_core::felt::{Felt, NonZeroFelt};
use std::{
    num::{NonZeroU128, NonZeroU64},
    str::FromStr,
};

#[derive(Args, Debug, Clone)]
pub struct FeeArgs {
    /// Token that transaction fee will be paid in
    #[clap(long, value_parser = parse_fee_token)]
    pub fee_token: Option<FeeToken>,

    /// Max fee for the transaction. If not provided, will be automatically estimated.
    #[clap(value_parser = parse_non_zero_felt, short, long)]
    pub max_fee: Option<NonZeroFelt>,

    /// Max gas amount. If not provided, will be automatically estimated. (Only for STRK fee payment)
    #[clap(value_parser = parse_non_zero_felt, long)]
    pub max_gas: Option<NonZeroFelt>,

    /// Max gas price in Fri. If not provided, will be automatically estimated. (Only for STRK fee payment)
    #[clap(value_parser = parse_non_zero_felt, long)]
    pub max_gas_unit_price: Option<NonZeroFelt>,
}

impl From<ScriptFeeSettings> for FeeArgs {
    fn from(script_fee_settings: ScriptFeeSettings) -> Self {
        match script_fee_settings {
            ScriptFeeSettings::Eth { max_fee } => Self {
                fee_token: Some(FeeToken::Eth),
                max_fee,
                max_gas: None,
                max_gas_unit_price: None,
            },
            ScriptFeeSettings::Strk {
                max_fee,
                max_gas,
                max_gas_unit_price,
            } => Self {
                fee_token: Some(FeeToken::Strk),
                max_fee,
                max_gas: max_gas.and_then(|val| NonZeroFelt::try_from(Felt::from_(val)).ok()),
                max_gas_unit_price: max_gas_unit_price
                    .and_then(|val| NonZeroFelt::try_from(Felt::from_(val)).ok()),
            },
        }
    }
}

impl FeeArgs {
    #[must_use]
    pub fn fee_token(self, fee_token: FeeToken) -> Self {
        Self {
            fee_token: Some(fee_token),
            ..self
        }
    }

    pub async fn try_into_fee_settings<P: Provider>(
        &self,
        provider: P,
        block_id: BlockId,
    ) -> Result<FeeSettings> {
        match self.fee_token.clone().unwrap_or_else(|| unreachable!()) {
            FeeToken::Eth => {
                ensure!(
                    self.max_gas.is_none(),
                    "--max-gas is not supported for ETH fee payment"
                );
                ensure!(
                    self.max_gas_unit_price.is_none(),
                    "--max-gas-unit-price is not supported for ETH fee payment"
                );
                Ok(FeeSettings::Eth {
                    max_fee: self.max_fee,
                })
            }
            FeeToken::Strk => {
                let settings = match (self.max_fee, self.max_gas, self.max_gas_unit_price) {
                    (Some(_), Some(_), Some(_)) => {
                        bail!("Passing all --max-fee, --max-gas and --max-gas-unit-price is conflicting. Please pass only two of them or less")
                    }
                    (Some(max_fee), Some(max_gas), None) if max_fee < max_gas => {
                        bail!("--max-fee should be greater than or equal to --max-gas amount")
                    }
                    (Some(max_fee), None, Some(max_gas_unit_price))
                        if max_fee < max_gas_unit_price =>
                    {
                        bail!("--max-fee should be greater than or equal to --max-gas-unit-price")
                    }
                    (None, _, _) => FeeSettings::Strk {
                        max_gas: self
                            .max_gas
                            .and_then(|val| NonZeroU64::try_from_(Felt::from(val)).ok()),
                        max_gas_unit_price: self
                            .max_gas_unit_price
                            .and_then(|val| NonZeroU128::try_from_(Felt::from(val)).ok()),
                    },
                    (Some(max_fee), None, Some(max_gas_unit_price)) => {
                        let max_gas = NonZeroFelt::try_from(Felt::from(max_fee).floor_div(&max_gas_unit_price)).context("Calculated max gas from provided --max-fee and --max-gas-unit-price is zero. Please increase --max-fee to obtain a positive gas amount")?;
                        print_max_fee_conversion_info(
                            max_fee.into(),
                            max_gas.into(),
                            max_gas_unit_price.into(),
                        );
                        FeeSettings::Strk {
                            max_gas: NonZeroU64::try_from_(Felt::from(max_gas)).ok(),
                            max_gas_unit_price: NonZeroU128::try_from_(Felt::from(
                                max_gas_unit_price,
                            ))
                            .ok(),
                        }
                    }
                    (Some(max_fee), Some(max_gas), None) => {
                        let max_gas_unit_price = NonZeroFelt::try_from(Felt::from(max_fee).floor_div(&max_gas)).context("Calculated max gas unit price from provided --max-fee and --max-gas is zero. Please increase --max-fee or decrease --max-gas to ensure a positive gas unit price")?;
                        print_max_fee_conversion_info(
                            max_fee.into(),
                            max_gas.into(),
                            max_gas_unit_price.into(),
                        );
                        FeeSettings::Strk {
                            max_gas: NonZeroU64::try_from_(Felt::from(max_gas)).ok(),
                            max_gas_unit_price: NonZeroU128::try_from_(Felt::from(
                                max_gas_unit_price,
                            ))
                            .ok(),
                        }
                    }
                    (Some(max_fee), None, None) => {
                        let max_gas_unit_price = provider
                            .get_block_with_tx_hashes(block_id)
                            .await?
                            .l1_gas_price()
                            .price_in_fri;
                        let max_gas = NonZeroFelt::try_from(Felt::from(max_fee)
                            .floor_div(&NonZeroFelt::try_from(max_gas_unit_price)?)).context("Calculated max-gas from provided --max-fee and the current network gas price is zero. Please increase --max-fee to obtain a positive gas amount")?;
                        print_max_fee_conversion_info(
                            max_fee.into(),
                            max_gas.into(),
                            max_gas_unit_price,
                        );
                        FeeSettings::Strk {
                            max_gas: NonZeroU64::try_from_(Felt::from(max_gas)).ok(),
                            max_gas_unit_price: NonZeroU128::try_from_(max_gas_unit_price).ok(),
                        }
                    }
                };

                Ok(settings)
            }
        }
    }
}

#[derive(ValueEnum, Default, Debug, Clone, PartialEq)]
pub enum FeeToken {
    Eth,
    #[default]
    Strk,
}

/// Struct used in `sncast script` for deserializing from cairo, `FeeSettings` can't be
/// used as it missing `max_fee` for `Strk`
#[derive(Debug, PartialEq, CairoDeserialize)]
pub enum ScriptFeeSettings {
    Eth {
        max_fee: Option<NonZeroFelt>,
    },
    Strk {
        max_fee: Option<NonZeroFelt>,
        max_gas: Option<NonZeroU64>,
        max_gas_unit_price: Option<NonZeroU128>,
    },
}

#[derive(Debug, PartialEq)]
pub enum FeeSettings {
    Eth {
        max_fee: Option<NonZeroFelt>,
    },
    Strk {
        max_gas: Option<NonZeroU64>,
        max_gas_unit_price: Option<NonZeroU128>,
    },
}

impl From<ScriptFeeSettings> for FeeSettings {
    fn from(value: ScriptFeeSettings) -> Self {
        match value {
            ScriptFeeSettings::Eth { max_fee } => FeeSettings::Eth { max_fee },
            ScriptFeeSettings::Strk {
                max_gas,
                max_gas_unit_price,
                ..
            } => FeeSettings::Strk {
                max_gas,
                max_gas_unit_price,
            },
        }
    }
}

pub trait PayableTransaction {
    fn error_message(&self, token: &str, version: &str) -> String;
    fn validate_and_get_token(&self) -> Result<FeeToken>;
    fn token_from_version(&self) -> Option<FeeToken>;
}

#[macro_export]
macro_rules! impl_payable_transaction {
    ($type:ty, $err_func:ident, $($version:pat => $token:expr),+) => {
        impl PayableTransaction for $type {
            fn error_message(&self, token: &str, version: &str) -> String {
                $err_func(token, version)
            }

            fn validate_and_get_token(&self) -> Result<FeeToken> {
                match (
                    &self.token_from_version(),
                    &self.fee_args.fee_token,
                ) {
                    (Some(token_from_version), Some(token)) if token_from_version != token => {
                        Err(anyhow!(self.error_message(
                            &format!("{:?}", token).to_lowercase(),
                            &format!("{:?}", self.version.clone().unwrap()).to_lowercase()
                        )))
                    },
                    (None, Some(token)) => {
                        Ok(token.clone())
                    },
                    (Some(token_from_version), None) => {
                        Ok(token_from_version.clone())
                    },
                    (None, None) => {
                        Ok(FeeToken::default())
                    },
                    _ =>  Ok(self.fee_args.fee_token.clone().unwrap_or_else(|| self.token_from_version().unwrap_or_else(|| unreachable!())))
                }
            }

            fn token_from_version(&self) -> Option<FeeToken> {
                self.version.clone().map(|version| match version {
                    $($version => $token),+
                })
            }
        }
    };
}

impl FromStr for FeeToken {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "eth" => Ok(FeeToken::Eth),
            "strk" => Ok(FeeToken::Strk),
            _ => Err(String::from("Possible values: eth, strk")),
        }
    }
}

fn parse_fee_token(s: &str) -> Result<FeeToken, String> {
    let deprecation_message = "Specifying '--fee-token' flag is deprecated and will be removed in the future. Use '--version' instead";
    print_as_warning(&Error::msg(deprecation_message));

    let parsed_token: FeeToken = s.parse()?;

    if parsed_token == FeeToken::Eth {
        print_as_warning(&Error::msg(
            "Eth transactions will stop being supported in the future due to 'SNIP-16'",
        ));
    }

    Ok(parsed_token)
}

fn print_max_fee_conversion_info(max_fee: Felt, max_gas: Felt, max_gas_unit_price: Felt) {
    println!(
        "Specifying '--max-fee' flag while using v3 transactions results in conversion to '--max-gas' and '--max-gas-unit-price' flags\nConverted {max_fee} max fee to {max_gas} max gas and {max_gas_unit_price} max gas unit price\n",
    );
}

fn parse_non_zero_felt(s: &str) -> Result<NonZeroFelt, String> {
    let felt: Felt = s.parse().map_err(|_| "Failed to parse value")?;
    felt.try_into()
        .map_err(|_| "Value should be greater than 0".to_string())
}
