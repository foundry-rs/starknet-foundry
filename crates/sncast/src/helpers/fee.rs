use anyhow::{bail, ensure, Result};
use clap::{Args, ValueEnum};
use conversions::serde::deserialize::CairoDeserialize;
use conversions::TryIntoConv;
use starknet::core::types::{BlockId, Felt};
use starknet::providers::Provider;
use starknet_types_core::felt::NonZeroFelt;

#[derive(Args, Debug, Clone)]
pub struct FeeArgs {
    /// Token that transaction fee will be paid in
    #[clap(long)]
    pub fee_token: Option<FeeToken>,

    /// Max fee for the transaction. If not provided, will be automatically estimated.
    #[clap(short, long)]
    pub max_fee: Option<Felt>,

    /// Max gas amount. If not provided, will be automatically estimated. (Only for STRK fee payment)
    #[clap(long)]
    pub max_gas: Option<Felt>,

    /// Max gas price in Fri. If not provided, will be automatically estimated. (Only for STRK fee payment)
    #[clap(long)]
    pub max_gas_unit_price: Option<Felt>,
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
                max_gas: max_gas.map(Into::into),
                max_gas_unit_price: max_gas_unit_price.map(Into::into),
            },
        }
    }
}

impl FeeArgs {
    #[must_use]
    pub fn fee_token(self, fee_token: Option<FeeToken>) -> Self {
        Self {
            fee_token: fee_token.or(self.fee_token),
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
                        max_gas: self.max_gas.map(TryIntoConv::try_into_).transpose()?,
                        max_gas_unit_price: self
                            .max_gas_unit_price
                            .map(TryIntoConv::try_into_)
                            .transpose()?,
                    },
                    (Some(max_fee), None, Some(max_gas_unit_price)) => FeeSettings::Strk {
                        max_gas: Some(
                            max_fee
                                .floor_div(&NonZeroFelt::from_felt_unchecked(max_gas_unit_price))
                                .try_into_()?,
                        ),
                        max_gas_unit_price: Some(max_gas_unit_price.try_into_()?),
                    },
                    (Some(max_fee), Some(max_gas), None) => FeeSettings::Strk {
                        max_gas: Some(max_gas.try_into_()?),
                        max_gas_unit_price: Some(
                            max_fee
                                .floor_div(&NonZeroFelt::from_felt_unchecked(max_gas))
                                .try_into_()?,
                        ),
                    },
                    (Some(max_fee), None, None) => {
                        let max_gas_unit_price = provider
                            .get_block_with_tx_hashes(block_id)
                            .await?
                            .l1_gas_price()
                            .price_in_fri;

                        FeeSettings::Strk {
                            max_gas: Some(
                                max_fee
                                    .floor_div(&NonZeroFelt::from_felt_unchecked(
                                        max_gas_unit_price,
                                    ))
                                    .try_into_()?,
                            ),
                            max_gas_unit_price: Some(max_gas_unit_price.try_into_()?),
                        }
                    }
                };

                Ok(settings)
            }
        }
    }
}

#[derive(ValueEnum, Debug, Clone, PartialEq)]
pub enum FeeToken {
    Eth,
    Strk,
}

/// Struct used in `sncast script` for deserializing from cairo, `FeeSettings` can't be
/// used as it missing `max_fee` for `Strk`
#[derive(Debug, PartialEq, CairoDeserialize)]
pub enum ScriptFeeSettings {
    Eth {
        max_fee: Option<Felt>,
    },
    Strk {
        max_fee: Option<Felt>,
        max_gas: Option<u64>,
        max_gas_unit_price: Option<u128>,
    },
}

#[derive(Debug, PartialEq)]
pub enum FeeSettings {
    Eth {
        max_fee: Option<Felt>,
    },
    Strk {
        max_gas: Option<u64>,
        max_gas_unit_price: Option<u128>,
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
    fn validate(&self) -> Result<()>;
    fn token_from_version(&self) -> Option<FeeToken>;
}

#[macro_export]
macro_rules! impl_payable_transaction {
    ($type:ty, $err_func:ident, $($version:pat => $token:expr),+) => {
        impl PayableTransaction for $type {
            fn error_message(&self, token: &str, version: &str) -> String {
                $err_func(token, version)
            }

            fn validate(&self) -> Result<()> {
                match (
                    &self.token_from_version(),
                    &self.fee_args.fee_token,
                ) {
                    (Some(token_from_version), Some(token)) if token_from_version != token => {
                        Err(anyhow!(self.error_message(
                            &format!("{token:?}").to_lowercase(),
                            &format!("{:?}", self.version.clone().unwrap()).to_lowercase()
                        )))
                    }
                    (None, None) => {
                        Err(anyhow!("Either --fee-token or --version must be provided"))
                    }
                    _ => Ok(()),
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
