use anyhow::{bail, ensure, Result};
use clap::{Args, ValueEnum};
use sncast::handle_account_factory_error;
use starknet::accounts::{AccountDeploymentV1, AccountDeploymentV3, AccountFactory};
use starknet::core::types::FieldElement;

#[derive(Args, Debug)]
pub struct FeeArgs {
    /// Token that transaction fee will be paid in
    #[clap(long)]
    pub fee_token: Option<FeeToken>,

    /// Max fee for the transaction. If not provided, will be automatically estimated. (Only for ETH fee payment)
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
                let settings = args.max_fee.map(|max_fee| EthFeeSettings { max_fee });
                Ok(FeeSettings::Eth(settings))
            }
            FeeToken::Strk => {
                ensure!(
                    args.max_fee.is_none(),
                    "Max fee is not supported for STRK fee payment"
                );
                let settings = match (args.max_gas, args.max_gas_unit_price) {
                    (None, None) => None,
                    (Some(max_gas), Some(max_gas_unit_price)) => Some(StrkFeeSettings {
                        max_gas: max_gas.try_into().map_err(|err| {
                            anyhow::anyhow!("Failed to convert max gas amount: {}", err)
                        })?,
                        max_gas_unit_price: max_gas_unit_price.try_into().map_err(|err| {
                            anyhow::anyhow!("Failed to convert max gas unit price: {}", err)
                        })?,
                    }),
                    (Some(_), None) => {
                        bail!("You only provided max gas amount, but not max gas unit price")
                    }
                    (None, Some(_)) => {
                        bail!("You only provided max gas unit price, but not max gas amount")
                    }
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
    pub max_fee: FieldElement,
}

impl EthFeeSettings {
    pub async fn estimate<T>(deployment: &AccountDeploymentV1<'_, T>) -> Result<Self>
    where
        T: AccountFactory + Sync,
    {
        deployment
            .estimate_fee()
            .await
            .map_err(handle_account_factory_error::<T>)
            .map(|estimated_fee| Self {
                max_fee: estimated_fee.overall_fee,
            })
    }
}

#[derive(Debug, PartialEq)]
pub struct StrkFeeSettings {
    pub max_gas: u64,
    pub max_gas_unit_price: u128,
}

impl StrkFeeSettings {
    pub async fn estimate<T>(deployment: &AccountDeploymentV3<'_, T>) -> Result<Self>
    where
        T: AccountFactory + Sync,
    {
        let estimate_fee = deployment
            .estimate_fee()
            .await
            .map_err(handle_account_factory_error::<T>)?;

        Ok(Self {
            max_gas: estimate_fee.gas_consumed.try_into()?,
            max_gas_unit_price: estimate_fee.gas_price.try_into()?,
        })
    }
}

#[derive(Debug, PartialEq)]
pub enum FeeSettings {
    Eth(Option<EthFeeSettings>),
    Strk(Option<StrkFeeSettings>),
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
            FeeSettings::Eth(Some(EthFeeSettings {
                max_fee: 100_u32.into()
            }))
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
            FeeSettings::Strk(Some(StrkFeeSettings {
                max_gas: 100,
                max_gas_unit_price: 100
            }))
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
            max_fee: Some(100_u32.into()),
            max_gas: Some(100_u32.into()),
            max_gas_unit_price: Some(100_u32.into()),
        };

        let error = FeeSettings::try_from(args).unwrap_err();

        assert!(error
            .to_string()
            .contains("Max fee is not supported for STRK fee payment"));
    }

    #[test]
    fn test_max_gas_strk() {
        let args = FeeArgs {
            fee_token: Some(FeeToken::Strk),
            max_fee: None,
            max_gas: None,
            max_gas_unit_price: Some(100_u32.into()),
        };

        let error = FeeSettings::try_from(args).unwrap_err();

        assert!(error
            .to_string()
            .contains("You only provided max gas unit price, but not max gas amount"));
    }

    #[test]
    fn test_max_gas_unit_price_strk() {
        let args = FeeArgs {
            fee_token: Some(FeeToken::Strk),
            max_fee: None,
            max_gas: Some(100_u32.into()),
            max_gas_unit_price: None,
        };

        let error = FeeSettings::try_from(args).unwrap_err();

        assert!(error
            .to_string()
            .contains("You only provided max gas amount, but not max gas unit price"));
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
}
