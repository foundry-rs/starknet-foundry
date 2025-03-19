use sncast::helpers::fee::{FeeArgs, FeeSettings};
use starknet_types_core::felt::Felt;

#[tokio::test]
async fn test_happy_case() {
    let args = FeeArgs {
        max_fee: None,
        l1_gas: Some(100),
        l1_gas_price: Some(200),
        l2_gas: Some(100),
        l2_gas_price: Some(200),
        l1_data_gas: Some(100),
        l1_data_gas_price: Some(200),
    };

    let settings = args.try_into_fee_settings(None).unwrap();

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

use test_case::test_case;
#[test_case(FeeArgs {
        max_fee: Some(Felt::from(50_u32).try_into().unwrap()),
        l1_gas: Some(100),
        l1_gas_price: None,
        l2_gas: None,
        l2_gas_price: None,
        l1_data_gas: None,
        l1_data_gas_price: None,
    }, "--l1-gas")]
#[test_case(FeeArgs {
        max_fee: Some(Felt::from(50_u32).try_into().unwrap()),
        l1_gas: None,
        l1_gas_price: Some(100),
        l2_gas: None,
        l2_gas_price: None,
        l1_data_gas: None,
        l1_data_gas_price: None,
    }, "--l1-gas-price")]
#[test_case(FeeArgs {
        max_fee: Some(Felt::from(50_u32).try_into().unwrap()),
        l1_gas: None,
        l1_gas_price: None,
        l2_gas: Some(100),
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
        l1_data_gas: Some(100),
        l1_data_gas_price: None,
    }, "--l1-data-gas")]
#[test_case(FeeArgs {
        max_fee: Some(Felt::from(50_u32).try_into().unwrap()),
        l1_gas: None,
        l1_gas_price: None,
        l2_gas: None,
        l2_gas_price: None,
        l1_data_gas: None,
        l1_data_gas_price: Some(100),
    }, "--l1-data-gas-price")]
#[tokio::test]
async fn test_max_fee_less_than_resource_bounds_value(fee_args: FeeArgs, flag: &str) {
    let error = fee_args.try_into_fee_settings(None).unwrap_err();
    assert!(
        error
            .to_string()
            .contains(format!("--max-fee should be greater than or equal to {flag}").as_str())
    );
}

#[tokio::test]
async fn test_max_fee_none() {
    let args = FeeArgs {
        max_fee: None,
        l1_gas: Some(100),
        l1_gas_price: Some(100),
        l2_gas: Some(100),
        l2_gas_price: Some(100),
        l1_data_gas: Some(100),
        l1_data_gas_price: Some(100),
    };

    let settings = args.try_into_fee_settings(None).unwrap();

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
    let args = FeeArgs {
        max_fee: None,
        l1_gas: None,
        l1_gas_price: None,
        l2_gas: None,
        l2_gas_price: None,
        l1_data_gas: None,
        l1_data_gas_price: None,
    };

    let settings = args.try_into_fee_settings(None).unwrap();

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
