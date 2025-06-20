use sncast::helpers::fee::{FeeArgs, FeeSettings};
use starknet::core::types::{FeeEstimate, PriceUnit};
use starknet_types_core::felt::{Felt, NonZeroFelt};

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
async fn test_max_fee_set() {
    let mock_fee_estimate = FeeEstimate {
        l1_gas_consumed: 1,
        l1_gas_price: 2,
        l2_gas_consumed: 3,
        l2_gas_price: 4,
        l1_data_gas_consumed: 5,
        l1_data_gas_price: 6,
        unit: PriceUnit::Fri,
        overall_fee: 44,
    };

    let args = FeeArgs {
        max_fee: Some(NonZeroFelt::try_from(Felt::from(100)).unwrap()),
        l1_gas: None,
        l1_gas_price: None,
        l2_gas: None,
        l2_gas_price: None,
        l1_data_gas: None,
        l1_data_gas_price: None,
    };

    let settings = args
        .try_into_fee_settings(Some(&mock_fee_estimate))
        .unwrap();

    assert_eq!(
        settings,
        FeeSettings {
            l1_gas: Some(1),
            l1_gas_price: Some(2),
            l2_gas: Some(3),
            l2_gas_price: Some(4),
            l1_data_gas: Some(5),
            l1_data_gas_price: Some(6),
        }
    );
}

#[tokio::test]
async fn test_max_fee_set_and_fee_estimate_higher() {
    let mock_fee_estimate = FeeEstimate {
        l1_gas_consumed: 10,
        l1_data_gas_price: 20,
        l2_gas_consumed: 30,
        l2_gas_price: 40,
        l1_data_gas_consumed: 50,
        l1_gas_price: 60,
        unit: PriceUnit::Fri,
        overall_fee: 4400,
    };

    let args = FeeArgs {
        max_fee: Some(NonZeroFelt::try_from(Felt::from(100)).unwrap()),
        l1_gas: None,
        l1_gas_price: None,
        l2_gas: None,
        l2_gas_price: None,
        l1_data_gas: None,
        l1_data_gas_price: None,
    };

    let err = args
        .try_into_fee_settings(Some(&mock_fee_estimate))
        .unwrap_err();

    assert_eq!(
        err.to_string(),
        format!(
            "Estimated fee ({}) is higher than provided max fee ({})",
            mock_fee_estimate.overall_fee,
            Felt::from(args.max_fee.unwrap())
        )
    );
}

#[tokio::test]
#[should_panic(expected = "Fee estimate must be passed when max_fee is provided")]
async fn test_max_fee_set_and_fee_estimate_none() {
    let args = FeeArgs {
        max_fee: Some(NonZeroFelt::try_from(Felt::from(100)).unwrap()),
        l1_gas: None,
        l1_gas_price: None,
        l2_gas: None,
        l2_gas_price: None,
        l1_data_gas: None,
        l1_data_gas_price: None,
    };

    args.try_into_fee_settings(None).unwrap();
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
