// use crate::helpers::constants::URL;
// use sncast::helpers::constants::OZ_CLASS_HASH;
use sncast::helpers::fee::{FeeArgs, FeeSettings};
// use starknet::providers::jsonrpc::HttpTransport;
// use starknet::providers::{JsonRpcClient, Provider};
// use starknet::signers::{LocalWallet, SigningKey};
use starknet_types_core::felt::Felt;
// use url::Url;

// FIXME
// const MAX_FEE: u64 = 1_000_000_000_000;

// async fn get_factory() -> OpenZeppelinAccountFactory<LocalWallet, JsonRpcClient<HttpTransport>> {
//     let parsed_url = Url::parse(URL).unwrap();
//     let provider = JsonRpcClient::new(HttpTransport::new(parsed_url));
//     let chain_id = provider.chain_id().await.unwrap();
//     let signer = LocalWallet::from_signing_key(SigningKey::from_random());

//     OpenZeppelinAccountFactory::new(OZ_CLASS_HASH, chain_id, signer, provider)
//         .await
//         .unwrap()
// }

#[tokio::test]
async fn test_happy_case() {
    // let factory = get_factory().await;

    let args = FeeArgs {
        max_fee: None,
        l1_gas: Some(Felt::from(100_u32)),
        l1_gas_price: Some(Felt::from(200_u32)),
        l2_gas: Some(Felt::from(100_u32)),
        l2_gas_price: Some(Felt::from(200_u32)),
        l1_data_gas: Some(Felt::from(100_u32)),
        l1_data_gas_price: Some(Felt::from(200_u32)),
    };

    let settings = args.try_into_fee_settings(&None).unwrap();

    assert_eq!(
        settings,
        FeeSettings {
            l1_gas: Some(100),
            l1_gas_price: Some(200),
            l2_gas: Some(100),
            l2_gas_price: Some(200),
            l1_data_gas: Some(100),
            l1_data_gas_price: Some(200),
        }
    );
}

#[tokio::test]
async fn test_all_args() {
    // let factory = get_factory().await;

    let args = FeeArgs {
        max_fee: Some(Felt::from(100_u32).try_into().unwrap()),
        l1_gas: Some(Felt::from(100_u32)),
        l1_gas_price: Some(Felt::from(200_u32)),
        l2_gas: Some(Felt::from(100_u32)),
        l2_gas_price: Some(Felt::from(200_u32)),
        l1_data_gas: Some(Felt::from(100_u32)),
        l1_data_gas_price: Some(Felt::from(200_u32)),
    };

    let error = args.try_into_fee_settings(&None).unwrap_err();

    assert!(error.to_string().contains("Passing all --max-fee, --l1-gas, --l1-gas-price, --l2-gas, --l2-gas-price, --l1-data-gas and --l1-data-gas-price is conflicting. Please pass only two of them or less"
    ));
}

use test_case::test_case;
#[test_case(FeeArgs {
        max_fee: Some(Felt::from(50_u32).try_into().unwrap()),
        l1_gas: Some(Felt::from(100_u32)),
        l1_gas_price: None,
        l2_gas: None,
        l2_gas_price: None,
        l1_data_gas: None,
        l1_data_gas_price: None,
    }, "--l1-gas")]
#[test_case(FeeArgs {
        max_fee: Some(Felt::from(50_u32).try_into().unwrap()),
        l1_gas: None,
        l1_gas_price: Some(Felt::from(100_u32)),
        l2_gas: None,
        l2_gas_price: None,
        l1_data_gas: None,
        l1_data_gas_price: None,
    }, "--l1-gas-price")]
#[test_case(FeeArgs {
        max_fee: Some(Felt::from(50_u32).try_into().unwrap()),
        l1_gas: None,
        l1_gas_price: None,
        l2_gas: Some(Felt::from(100_u32)),
        l2_gas_price: None,
        l1_data_gas: None,
        l1_data_gas_price: None,
    }, "--l2-gas")]
#[test_case(FeeArgs {
        max_fee: Some(Felt::from(50_u32).try_into().unwrap()),
        l1_gas: None,
        l1_gas_price: None,
        l2_gas: None,
        l2_gas_price: None,
        l1_data_gas: Some(Felt::from(100_u32)),
        l1_data_gas_price: None,
    }, "--l1-data-gas")]
#[test_case(FeeArgs {
        max_fee: Some(Felt::from(50_u32).try_into().unwrap()),
        l1_gas: None,
        l1_gas_price: None,
        l2_gas: None,
        l2_gas_price: None,
        l1_data_gas: None,
        l1_data_gas_price: Some(Felt::from(100_u32)),
    }, "--l1-data-gas-price")]
#[tokio::test]
async fn test_max_fee_less_than_resource_bounds_value(fee_args: FeeArgs, flag: &str) {
    let error = fee_args.try_into_fee_settings(&None).unwrap_err();
    assert!(
        error
            .to_string()
            .contains(format!("--max-fee should be greater than or equal to {flag}").as_str())
    );
}

// #[tokio::test]
// async fn test_get_missing_gas_prices() {
//     let factory = get_factory().await;

//     let args = FeeArgs {
//         max_fee: Some(Felt::from(MAX_FEE).try_into().unwrap()),
//         l1_gas: Some(Felt::from(100_u32).try_into().unwrap()),
//         l1_gas_price: None,
//         l2_gas: Some(Felt::from(100_u32).try_into().unwrap()),
//         l2_gas_price: None,
//         l1_data_gas: Some(Felt::from(100_u32).try_into().unwrap()),
//         l1_data_gas_price: None,
//     };

//     let settings = args
//         .try_into_fee_settings(factory.provider(), factory.block_id())
//         .await
//         .unwrap();

//     assert_eq!(
//         settings,
//         FeeSettings {
//             l1_gas: Some(100),
//             l1_gas_price: Some(100000000000),
//             l2_gas: Some(100),
//             l2_gas_price: Some(100000000000),
//             l1_data_gas: Some(100),
//             l1_data_gas_price: Some(100000000000),
//         }
//     );
// }

#[tokio::test]
async fn test_max_fee_none() {
    // let factory = get_factory().await;

    let args = FeeArgs {
        max_fee: None,
        l1_gas: Some(Felt::from(100_u32)),
        l1_gas_price: Some(Felt::from(100_u32)),
        l2_gas: Some(Felt::from(100_u32)),
        l2_gas_price: Some(Felt::from(100_u32)),
        l1_data_gas: Some(Felt::from(100_u32)),
        l1_data_gas_price: Some(Felt::from(100_u32)),
    };

    let settings = args.try_into_fee_settings(&None).unwrap();

    assert_eq!(
        settings,
        FeeSettings {
            l1_gas: Some(100),
            l1_gas_price: Some(100),
            l2_gas: Some(100),
            l2_gas_price: Some(100),
            l1_data_gas: Some(100),
            l1_data_gas_price: Some(100),
        }
    );
}

#[tokio::test]
async fn test_all_args_none() {
    // let factory = get_factory().await;

    let args = FeeArgs {
        max_fee: None,
        l1_gas: None,
        l1_gas_price: None,
        l2_gas: None,
        l2_gas_price: None,
        l1_data_gas: None,
        l1_data_gas_price: None,
    };

    let settings = args.try_into_fee_settings(&None).unwrap();

    assert_eq!(
        settings,
        FeeSettings {
            l1_gas: None,
            l1_gas_price: None,
            l2_gas: None,
            l2_gas_price: None,
            l1_data_gas: None,
            l1_data_gas_price: None,
        }
    );
}
