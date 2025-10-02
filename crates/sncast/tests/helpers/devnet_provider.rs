use num_traits::ToPrimitive;
use sncast::helpers::{
    constants::OZ_CLASS_HASH,
    devnet_provider::{DevnetProvider, PredeployedAccount},
};
use starknet::macros::felt;

use crate::helpers::constants::{DEVNET_ACCOUNTS_NUMBER, DEVNET_SEED, SEPOLIA_RPC_URL, URL};

#[tokio::test]
async fn test_get_config() {
    let devnet_provider = DevnetProvider::new(URL);
    let config = devnet_provider
        .get_config()
        .await
        .expect("Failed to get config");

    assert!(config.account_contract_class_hash == OZ_CLASS_HASH);
    assert!(config.seed == DEVNET_SEED);
    assert!(config.total_accounts == DEVNET_ACCOUNTS_NUMBER);
}

#[tokio::test]
async fn test_get_predeployed_accounts() {
    let devnet_provider = DevnetProvider::new(URL);
    let predeployed_accounts = devnet_provider
        .get_predeployed_accounts()
        .await
        .expect("Failed to get predeployed accounts");

    assert!(predeployed_accounts.len().to_u8().unwrap() == DEVNET_ACCOUNTS_NUMBER);

    let first_account = &predeployed_accounts[0];
    let expected_first_account = PredeployedAccount {
        address: felt!("0x06f4621e7ad43707b3f69f9df49425c3d94fdc5ab2e444bfa0e7e4edeff7992d"),
        private_key: felt!("0x0000000000000000000000000000000056c12e097e49ea382ca8eadec0839401"),
        public_key: felt!("0x048234b9bc6c1e749f4b908d310d8c53dae6564110b05ccf79016dca8ce7dfac"),
    };
    assert!(first_account.address == expected_first_account.address);
    assert!(first_account.private_key == expected_first_account.private_key);
    assert!(first_account.public_key == expected_first_account.public_key);
}

#[tokio::test]
async fn test_is_alive_happy_case() {
    let devnet_provider = DevnetProvider::new(URL);
    devnet_provider
        .ensure_alive()
        .await
        .expect("Failed to ensure the devnet is alive");
}

#[tokio::test]
async fn test_is_alive_fails_on_sepolia_node() {
    let devnet_provider = DevnetProvider::new(SEPOLIA_RPC_URL);
    let res = devnet_provider.ensure_alive().await;
    assert!(res.is_err(), "Expected an error");

    let err = res.unwrap_err().to_string();
    assert!(
        err == format!(
            "Node at {SEPOLIA_RPC_URL} is not responding to the Devnet health check (GET `/is_alive`). It may not be a Devnet instance or it may be down."
        ),
        "Unexpected error message: {err}"
    );
}
