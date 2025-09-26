use num_traits::ToPrimitive;
use sncast::helpers::{constants::OZ_CLASS_HASH, devnet_provider::DevnetProvider};
use starknet::macros::felt;

use crate::helpers::constants::{ACCOUNTS_NUMBER, SEED, URL};

#[tokio::test]
async fn test_get_config() {
    let devnet_provider = DevnetProvider::new(URL);
    let config = devnet_provider
        .get_config()
        .await
        .expect("Failed to get config");

    assert!(config.account_contract_class_hash == OZ_CLASS_HASH);
    assert!(config.seed == SEED);
    assert!(config.total_accounts == ACCOUNTS_NUMBER);
}

#[tokio::test]
async fn test_get_predeployed_accounts() {
    let devnet_provider = DevnetProvider::new(URL);
    let predeployed_accounts = devnet_provider
        .get_predeployed_accounts()
        .await
        .expect("Failed to get predeployed accounts");

    assert!(predeployed_accounts.len().to_u8().unwrap() == ACCOUNTS_NUMBER);

    let first_account = &predeployed_accounts[0];
    assert!(
        first_account.address
            == felt!("0x06f4621e7ad43707b3f69f9df49425c3d94fdc5ab2e444bfa0e7e4edeff7992d")
    );
    assert!(
        first_account.private_key
            == felt!("0x0000000000000000000000000000000056c12e097e49ea382ca8eadec0839401")
    );
    assert!(
        first_account.public_key
            == felt!("0x048234b9bc6c1e749f4b908d310d8c53dae6564110b05ccf79016dca8ce7dfac")
    );
}
