use crate::handle_account_factory_error;
use anyhow::{anyhow, bail, ensure, Result};
use clap::{Args, ValueEnum};
use starknet::accounts::{AccountDeploymentV1, AccountDeploymentV3, AccountFactory};
use starknet::core::types::FieldElement;

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

    /// Max gas price in STRK. If not provided, will be automatically estimated. (Only for STRK fee payment)
    #[clap(long)]
    pub max_gas_unit_price: Option<FieldElement>,
}

impl TryFrom<FeeArgs> for FeeSettings {
    type Error = anyhow::Error;

    fn try_from(args: FeeArgs) -> Result<Self> {
        match args
            .fee_token
            .ok_or_else(|| anyhow::anyhow!("Fee token is not provided"))?
        {
            FeeToken::Eth => {
                ensure!(
                    args.max_gas.is_none(),
                    "Max gas is not supported for ETH fee payment"
                );
                ensure!(
                    args.max_gas_unit_price.is_none(),
                    "Max gas unit price is not supported for ETH fee payment"
                );
                let settings = EthFeeSettings {
                    max_fee: args.max_fee,
                };
                Ok(FeeSettings::Eth(settings))
            }
            FeeToken::Strk => {
                match (args.max_fee, args.max_gas, args.max_gas_unit_price) {
                    (Some(max_fee), Some(max_gas), Some(max_gas_unit_price))
                        if max_fee != max_gas * max_gas_unit_price =>
                    {
                        bail!("Max fee should be equal to max gas amount multiplied by max gas unit price")
                    }
                    (Some(max_fee), Some(max_gas), None) if max_fee < max_gas => {
                        bail!("Max fee should be greater than or equal to max gas amount")
                    }
                    (Some(max_fee), None, Some(max_gas_unit_price))
                        if max_fee < max_gas_unit_price =>
                    {
                        bail!("Max fee should be greater than or equal to max gas unit price")
                    }
                    _ => {}
                }
                let settings = StrkFeeSettings {
                    max_fee: args.max_fee,
                    max_gas: args
                        .max_gas
                        .map(TryInto::try_into)
                        .transpose()
                        .map_err(|err| anyhow!("Failed to convert max gas amount: {}", err))?,
                    max_gas_unit_price: args
                        .max_gas_unit_price
                        .map(TryInto::try_into)
                        .transpose()
                        .map_err(|err| anyhow!("Failed to convert max gas unit price: {}", err))?,
                };

                Ok(FeeSettings::Strk(settings))
            }
        }
    }
}

#[derive(ValueEnum, Debug, Clone)]
pub enum FeeToken {
    Eth,
    Strk,
}

#[derive(Debug, PartialEq)]
pub struct EthFeeSettings {
    pub max_fee: Option<FieldElement>,
}

pub struct EthFee {
    pub max_fee: FieldElement,
}

impl EthFeeSettings {
    pub async fn get_or_estimate<T>(
        &self,
        deployment: &AccountDeploymentV1<'_, T>,
    ) -> Result<EthFee>
    where
        T: AccountFactory + Sync,
    {
        match self.max_fee {
            None => deployment
                .estimate_fee()
                .await
                .map_err(handle_account_factory_error::<T>)
                .map(|estimated_fee| EthFee {
                    max_fee: estimated_fee.overall_fee,
                }),
            Some(max_fee) => Ok(EthFee { max_fee }),
        }
    }
}
#[allow(clippy::struct_field_names)]
#[derive(Debug, PartialEq)]
pub struct StrkFeeSettings {
    pub max_fee: Option<FieldElement>,
    pub max_gas: Option<u64>,
    pub max_gas_unit_price: Option<u128>,
}

pub struct StrkFee {
    pub max_gas: u64,
    pub max_gas_unit_price: u128,
}
impl StrkFeeSettings {
    pub async fn get_or_estimate<T>(
        &self,
        deployment: &AccountDeploymentV3<'_, T>,
    ) -> Result<StrkFee>
    where
        T: AccountFactory + Sync,
    {
        let estimate_fee = deployment
            .estimate_fee()
            .await
            .map_err(handle_account_factory_error::<T>)?;

        let max_gas = self
            .max_gas
            .unwrap_or(estimate_fee.gas_consumed.try_into()?);
        let max_gas_unit_price = self
            .max_gas_unit_price
            .unwrap_or(estimate_fee.gas_price.try_into()?);

        match (self.max_fee, self.max_gas, self.max_gas_unit_price) {
            (_, Some(_), Some(_)) | (None, _, _) => Ok(StrkFee {
                max_gas,
                max_gas_unit_price,
            }),
            (Some(max_fee), None, _) => Ok(StrkFee {
                max_gas: max_fee.floor_div(max_gas_unit_price.into()).try_into()?,
                max_gas_unit_price,
            }),
            (Some(max_fee), Some(max_gas), None) => Ok(StrkFee {
                max_gas,
                max_gas_unit_price: max_fee.floor_div(max_gas.into()).try_into()?,
            }),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum FeeSettings {
    Eth(EthFeeSettings),
    Strk(StrkFeeSettings),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_happy_case_eth() {
        let args = FeeArgs {
            fee_token: Some(FeeToken::Eth),
            max_fee: Some(100_u32.into()),
            max_gas: None,
            max_gas_unit_price: None,
        };

        let settings: FeeSettings = args.try_into().unwrap();

        assert_eq!(
            settings,
            FeeSettings::Eth(EthFeeSettings {
                max_fee: Some(100_u32.into())
            })
        );
    }

    #[test]
    fn test_happy_case_strk() {
        let args = FeeArgs {
            fee_token: Some(FeeToken::Strk),
            max_fee: None,
            max_gas: Some(100_u32.into()),
            max_gas_unit_price: Some(100_u32.into()),
        };

        let settings: FeeSettings = args.try_into().unwrap();

        assert_eq!(
            settings,
            FeeSettings::Strk(StrkFeeSettings {
                max_fee: None,
                max_gas: Some(100),
                max_gas_unit_price: Some(100),
            })
        );
    }

    #[test]
    fn test_max_gas_eth() {
        let args = FeeArgs {
            fee_token: Some(FeeToken::Eth),
            max_fee: Some(100_u32.into()),
            max_gas: Some(100_u32.into()),
            max_gas_unit_price: None,
        };

        let error = FeeSettings::try_from(args).unwrap_err();

        assert!(error
            .to_string()
            .contains("Max gas is not supported for ETH fee payment"));
    }

    #[test]
    fn test_max_gas_unit_price_eth() {
        let args = FeeArgs {
            fee_token: Some(FeeToken::Eth),
            max_fee: Some(100_u32.into()),
            max_gas: None,
            max_gas_unit_price: Some(100_u32.into()),
        };

        let error = FeeSettings::try_from(args).unwrap_err();

        assert!(error
            .to_string()
            .contains("Max gas unit price is not supported for ETH fee payment"));
    }

    #[test]
    fn test_max_fee_strk() {
        let args = FeeArgs {
            fee_token: Some(FeeToken::Strk),
            max_fee: Some(10000_u32.into()),
            max_gas: Some(100_u32.into()),
            max_gas_unit_price: Some(100_u32.into()),
        };

        let settings: FeeSettings = args.try_into().unwrap();

        assert_eq!(
            settings,
            FeeSettings::Strk(StrkFeeSettings {
                max_fee: Some(10000_u32.into()),
                max_gas: Some(100),
                max_gas_unit_price: Some(100),
            })
        );
    }

    #[test]
    fn test_max_gas_unit_price_strk() {
        let args = FeeArgs {
            fee_token: Some(FeeToken::Strk),
            max_fee: None,
            max_gas: None,
            max_gas_unit_price: Some(100_u32.into()),
        };

        let settings: FeeSettings = args.try_into().unwrap();

        assert_eq!(
            settings,
            FeeSettings::Strk(StrkFeeSettings {
                max_fee: None,
                max_gas: None,
                max_gas_unit_price: Some(100),
            })
        );
    }

    #[test]
    fn test_max_gas_strk() {
        let args = FeeArgs {
            fee_token: Some(FeeToken::Strk),
            max_fee: None,
            max_gas: Some(100_u32.into()),
            max_gas_unit_price: None,
        };

        let settings: FeeSettings = args.try_into().unwrap();

        assert_eq!(
            settings,
            FeeSettings::Strk(StrkFeeSettings {
                max_fee: None,
                max_gas: Some(100),
                max_gas_unit_price: None,
            })
        );
    }

    #[test]
    fn test_no_fee_token() {
        let args = FeeArgs {
            fee_token: None,
            max_fee: Some(100_u32.into()),
            max_gas: Some(100_u32.into()),
            max_gas_unit_price: Some(100_u32.into()),
        };

        let error = FeeSettings::try_from(args).unwrap_err();

        assert!(error.to_string().contains("Fee token is not provided"));
    }

    #[test]
    fn test_max_fee_not_eq_max_gas_and_max_gas_unit_price() {
        let args = FeeArgs {
            fee_token: Some(FeeToken::Strk),
            max_fee: Some(100_u32.into()),
            max_gas: Some(100_u32.into()),
            max_gas_unit_price: Some(100_u32.into()),
        };

        let error = FeeSettings::try_from(args).unwrap_err();

        assert!(error.to_string().contains(
            "Max fee should be equal to max gas amount multiplied by max gas unit price"
        ));
    }

    #[test]
    fn test_max_fee_less_than_max_gas() {
        let args = FeeArgs {
            fee_token: Some(FeeToken::Strk),
            max_fee: Some(50_u32.into()),
            max_gas: Some(100_u32.into()),
            max_gas_unit_price: None,
        };

        let error = FeeSettings::try_from(args).unwrap_err();

        assert!(error
            .to_string()
            .contains("Max fee should be greater than or equal to max gas amount"));
    }

    #[test]
    fn test_max_fee_less_than_max_gas_unit_price() {
        let args = FeeArgs {
            fee_token: Some(FeeToken::Strk),
            max_fee: Some(50_u32.into()),
            max_gas: None,
            max_gas_unit_price: Some(100_u32.into()),
        };

        let error = FeeSettings::try_from(args).unwrap_err();

        assert!(error
            .to_string()
            .contains("Max fee should be greater than or equal to max gas unit price"));
    }
}
