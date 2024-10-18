use crate::helpers::constants::URL;
use sncast::helpers::constants::OZ_CLASS_HASH;
use sncast::helpers::fee::{FeeArgs, FeeSettings, FeeToken};
use starknet::accounts::{AccountFactory, OpenZeppelinAccountFactory};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Provider};
use starknet::signers::{LocalWallet, SigningKey};
use url::Url;

const MAX_FEE: u64 = 1_000_000_000_000;

async fn get_factory() -> OpenZeppelinAccountFactory<LocalWallet, JsonRpcClient<HttpTransport>> {
    let parsed_url = Url::parse(URL).unwrap();
    let provider = JsonRpcClient::new(HttpTransport::new(parsed_url));
    let chain_id = provider.chain_id().await.unwrap();
    let signer = LocalWallet::from_signing_key(SigningKey::from_random());

    OpenZeppelinAccountFactory::new(OZ_CLASS_HASH, chain_id, signer, provider)
        .await
        .unwrap()
}

#[tokio::test]
async fn test_happy_case_eth() {
    let factory = get_factory().await;

    let args = FeeArgs {
        fee_token: Some(FeeToken::Eth),
        max_fee: Some(100_u32.into()),
        max_gas: None,
        max_gas_unit_price: None,
    };

    let settings = args
        .try_into_fee_settings(factory.provider(), factory.block_id())
        .await
        .unwrap();

    assert_eq!(
        settings,
        FeeSettings::Eth {
            max_fee: Some(100_u32.into())
        }
    );
}

#[tokio::test]
async fn test_max_gas_eth() {
    let factory = get_factory().await;

    let args = FeeArgs {
        fee_token: Some(FeeToken::Eth),
        max_fee: Some(100_u32.into()),
        max_gas: Some(100_u32.into()),
        max_gas_unit_price: None,
    };

    let error = args
        .try_into_fee_settings(factory.provider(), factory.block_id())
        .await
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("--max-gas is not supported for ETH fee payment"));
}

#[tokio::test]
async fn test_max_gas_unit_price_eth() {
    let factory = get_factory().await;

    let args = FeeArgs {
        fee_token: Some(FeeToken::Eth),
        max_fee: Some(100_u32.into()),
        max_gas: None,
        max_gas_unit_price: Some(100_u32.into()),
    };

    let error = args
        .try_into_fee_settings(factory.provider(), factory.block_id())
        .await
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("--max-gas-unit-price is not supported for ETH fee payment"));
}

#[tokio::test]
async fn test_all_args() {
    let factory = get_factory().await;

    let args = FeeArgs {
        fee_token: Some(FeeToken::Strk),
        max_fee: Some(100_u32.into()),
        max_gas: Some(100_u32.into()),
        max_gas_unit_price: Some(100_u32.into()),
    };

    let error = args
        .try_into_fee_settings(factory.provider(), factory.block_id())
        .await
        .unwrap_err();

    assert!(error.to_string().contains(
        "Passing all --max-fee, --max-gas and --max-gas-unit-price is conflicting. Please pass only two of them or less"
    ));
}

#[tokio::test]
async fn test_max_fee_less_than_max_gas() {
    let factory = get_factory().await;

    let args = FeeArgs {
        fee_token: Some(FeeToken::Strk),
        max_fee: Some(50_u32.into()),
        max_gas: Some(100_u32.into()),
        max_gas_unit_price: None,
    };

    let error = args
        .try_into_fee_settings(factory.provider(), factory.block_id())
        .await
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("--max-fee should be greater than or equal to --max-gas amount"));
}

#[tokio::test]
async fn test_max_fee_less_than_max_gas_unit_price() {
    let factory = get_factory().await;

    let args = FeeArgs {
        fee_token: Some(FeeToken::Strk),
        max_fee: Some(50_u32.into()),
        max_gas: None,
        max_gas_unit_price: Some(100_u32.into()),
    };

    let error = args
        .try_into_fee_settings(factory.provider(), factory.block_id())
        .await
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("--max-fee should be greater than or equal to --max-gas-unit-price"));
}
#[tokio::test]
async fn test_strk_fee_get_max_fee() {
    let factory = get_factory().await;

    let args = FeeArgs {
        fee_token: Some(FeeToken::Strk),
        max_fee: Some(MAX_FEE.into()),
        max_gas: None,
        max_gas_unit_price: None,
    };

    let settings = args
        .try_into_fee_settings(factory.provider(), factory.block_id())
        .await
        .unwrap();

    match settings {
        FeeSettings::Strk {
            max_gas,
            max_gas_unit_price,
        } => {
            assert_eq!(
                u128::from(max_gas.unwrap()) * max_gas_unit_price.unwrap(),
                MAX_FEE.into()
            );
        }
        FeeSettings::Eth { .. } => unreachable!(),
    }
}

#[tokio::test]
async fn test_strk_fee_get_max_fee_with_max_gas() {
    let factory = get_factory().await;

    let args = FeeArgs {
        fee_token: Some(FeeToken::Strk),
        max_fee: Some(MAX_FEE.into()),
        max_gas: Some(1_000_000_u32.into()),
        max_gas_unit_price: None,
    };

    let settings = args
        .try_into_fee_settings(factory.provider(), factory.block_id())
        .await
        .unwrap();

    assert_eq!(
        settings,
        FeeSettings::Strk {
            max_gas: Some(1_000_000),
            max_gas_unit_price: Some(u128::from(MAX_FEE / 1_000_000)),
        }
    );

    match settings {
        FeeSettings::Strk {
            max_gas,
            max_gas_unit_price,
        } => {
            assert_eq!(
                u128::from(max_gas.unwrap()) * max_gas_unit_price.unwrap(),
                MAX_FEE.into()
            );
        }
        FeeSettings::Eth { .. } => unreachable!(),
    }
}

#[tokio::test]
async fn test_strk_fee_get_max_gas_and_max_gas_unit_price() {
    let factory = get_factory().await;

    let args = FeeArgs {
        fee_token: Some(FeeToken::Strk),
        max_fee: None,
        max_gas: Some(1_000_000_u32.into()),
        max_gas_unit_price: Some(1_000_u32.into()),
    };

    let settings = args
        .try_into_fee_settings(factory.provider(), factory.block_id())
        .await
        .unwrap();

    assert_eq!(
        settings,
        FeeSettings::Strk {
            max_gas: Some(1_000_000),
            max_gas_unit_price: Some(1_000),
        }
    );
}

#[tokio::test]
async fn test_strk_fee_get_max_fee_with_max_gas_unit_price() {
    let factory = get_factory().await;

    let args = FeeArgs {
        fee_token: Some(FeeToken::Strk),
        max_fee: Some(MAX_FEE.into()),
        max_gas: None,
        max_gas_unit_price: Some(1_000_u32.into()),
    };

    let settings = args
        .try_into_fee_settings(factory.provider(), factory.block_id())
        .await
        .unwrap();

    assert_eq!(
        settings,
        FeeSettings::Strk {
            max_gas: Some(MAX_FEE / 1_000),
            max_gas_unit_price: Some(1_000),
        }
    );

    match settings {
        FeeSettings::Strk {
            max_gas,
            max_gas_unit_price,
        } => {
            assert_eq!(
                u128::from(max_gas.unwrap()) * max_gas_unit_price.unwrap(),
                MAX_FEE.into()
            );
        }
        FeeSettings::Eth { .. } => unreachable!(),
    }
}

#[tokio::test]
async fn test_strk_fee_get_none() {
    let factory = get_factory().await;

    let args = FeeArgs {
        fee_token: Some(FeeToken::Strk),
        max_fee: None,
        max_gas: None,
        max_gas_unit_price: None,
    };

    let settings = args
        .try_into_fee_settings(factory.provider(), factory.block_id())
        .await
        .unwrap();

    assert_eq!(
        settings,
        FeeSettings::Strk {
            max_gas: None,
            max_gas_unit_price: None,
        }
    );
}
