use crate::helpers::constants::URL;
use sncast::helpers::constants::OZ_CLASS_HASH;
use sncast::helpers::fee::{EthFeeSettings, StrkFeeSettings};
use starknet::accounts::{AccountFactory, OpenZeppelinAccountFactory};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Provider};
use starknet::signers::{LocalWallet, SigningKey};
use starknet_crypto::FieldElement;
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
async fn test_eth_fee_get_or_estimate() {
    let factory = get_factory().await;
    let deployment = factory.deploy_v1(FieldElement::ZERO);

    let eth_fee_settings = EthFeeSettings {
        max_fee: Some(MAX_FEE.into()),
    };

    let eth_fee = eth_fee_settings.get_or_estimate(&deployment).await.unwrap();
    assert_eq!(eth_fee.max_fee, MAX_FEE.into());
}

#[tokio::test]
async fn test_strk_fee_get_or_estimate_max_fee() {
    let factory = get_factory().await;
    let deployment = factory.deploy_v3(FieldElement::ZERO);

    let strk_fee_settings = StrkFeeSettings {
        max_fee: Some(MAX_FEE.into()),
        max_gas: None,
        max_gas_unit_price: None,
    };

    let strk_fee = strk_fee_settings
        .get_or_estimate(&deployment)
        .await
        .unwrap();
    assert_eq!(
        (strk_fee.max_gas as u128) * strk_fee.max_gas_unit_price,
        MAX_FEE.into()
    );
}

#[tokio::test]
async fn test_strk_fee_get_or_estimate_max_fee_with_max_gas() {
    let factory = get_factory().await;
    let deployment = factory.deploy_v3(FieldElement::ZERO);

    let strk_fee_settings = StrkFeeSettings {
        max_fee: Some(MAX_FEE.into()),
        max_gas: Some(1_000_000),
        max_gas_unit_price: None,
    };

    let strk_fee = strk_fee_settings
        .get_or_estimate(&deployment)
        .await
        .unwrap();
    assert_eq!(strk_fee.max_gas, 1_000_000);
    assert_eq!(strk_fee.max_gas_unit_price, (MAX_FEE / 1_000_000) as u128);
    assert_eq!(
        strk_fee.max_gas as u128 * strk_fee.max_gas_unit_price,
        MAX_FEE.into()
    );
}

#[tokio::test]
async fn test_strk_fee_get_or_estimate_max_gas_and_max_gas_unit_price() {
    let factory = get_factory().await;
    let deployment = factory.deploy_v3(FieldElement::ZERO);

    let strk_fee_settings = StrkFeeSettings {
        max_fee: None,
        max_gas: Some(1_000_000),
        max_gas_unit_price: Some(1_000),
    };

    let strk_fee = strk_fee_settings
        .get_or_estimate(&deployment)
        .await
        .unwrap();
    assert_eq!(strk_fee.max_gas, 1_000_000);
    assert_eq!(strk_fee.max_gas_unit_price, 1_000);
}

#[tokio::test]
async fn test_strk_fee_get_or_estimate_max_fee_with_max_gas_unit_price() {
    let factory = get_factory().await;
    let deployment = factory.deploy_v3(FieldElement::ZERO);

    let strk_fee_settings = StrkFeeSettings {
        max_fee: Some(MAX_FEE.into()),
        max_gas: None,
        max_gas_unit_price: Some(1_000),
    };

    let strk_fee = strk_fee_settings
        .get_or_estimate(&deployment)
        .await
        .unwrap();
    assert_eq!(strk_fee.max_gas_unit_price, 1_000);
    assert_eq!(strk_fee.max_gas, MAX_FEE / 1_000);
    assert_eq!(
        strk_fee.max_gas as u128 * strk_fee.max_gas_unit_price,
        MAX_FEE.into()
    );
}
