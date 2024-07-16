use anyhow::{bail, ensure, Result};
use clap::{Args, ValueEnum};
use conversions::serde::deserialize::CairoDeserialize;
use starknet::core::types::{BlockId, FieldElement};
use starknet::providers::Provider;

#[derive(Args, Debug)]
pub struct FeeArgs {
    /// Token that transaction fee will be paid in
    #[clap(long)]
    pub fee_token: Option<FeeToken>,

    /// Max fee for the transaction. If not provided, will be automatically estimated.
    #[clap(short, long)]
    pub max_fee: Option<FieldElement>,

    /// Max gas amount. If not provided, will be automatically estimated. (Only for STRK fee payment)
    #[clap(long)]
    pub max_gas: Option<FieldElement>,

    /// Max gas price in Fri. If not provided, will be automatically estimated. (Only for STRK fee payment)
    #[clap(long)]
    pub max_gas_unit_price: Option<FieldElement>,
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
        self,
        provider: P,
        block_id: BlockId,
    ) -> Result<FeeSettings> {
        match self.fee_token.unwrap_or_else(|| unreachable!()) {
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
                        max_gas: self.max_gas.map(TryInto::try_into).transpose()?,
                        max_gas_unit_price: self
                            .max_gas_unit_price
                            .map(TryInto::try_into)
                            .transpose()?,
                    },
                    (Some(max_fee), None, Some(max_gas_unit_price)) => FeeSettings::Strk {
                        max_gas: Some(max_fee.floor_div(max_gas_unit_price).try_into()?),
                        max_gas_unit_price: Some(max_gas_unit_price.try_into()?),
                    },
                    (Some(max_fee), Some(max_gas), None) => FeeSettings::Strk {
                        max_gas: Some(max_gas.try_into()?),
                        max_gas_unit_price: Some(max_fee.floor_div(max_gas).try_into()?),
                    },
                    (Some(max_fee), None, None) => {
                        let max_gas_unit_price: u128 = provider
                            .get_block_with_tx_hashes(block_id)
                            .await?
                            .l1_gas_price()
                            .price_in_fri
                            .try_into()?;

                        FeeSettings::Strk {
                            max_gas: Some(max_fee.floor_div(max_gas_unit_price.into()).try_into()?),
                            max_gas_unit_price: Some(max_gas_unit_price),
                        }
                    }
                };

                Ok(settings)
            }
        }
    }
}

#[derive(ValueEnum, Debug, Clone)]
pub enum FeeToken {
    Eth,
    Strk,
}

/// Struct used in `sncast script` for deserializing from cairo, `FeeSettings` can't be
/// used as it missing `max_fee` for `Strk`
#[derive(Debug, PartialEq, CairoDeserialize)]
pub enum ScriptFeeSettings {
    Eth {
        max_fee: Option<FieldElement>,
    },
    Strk {
        max_fee: Option<FieldElement>,
        max_gas: Option<u64>,
        max_gas_unit_price: Option<u128>,
    },
}

#[derive(Debug, PartialEq)]
pub enum FeeSettings {
    Eth {
        max_fee: Option<FieldElement>,
    },
    Strk {
        max_gas: Option<u64>,
        max_gas_unit_price: Option<u128>,
    },
}
