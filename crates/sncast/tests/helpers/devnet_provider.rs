use crate::helpers::constants::{DEVNET_ACCOUNTS_NUMBER, DEVNET_SEED, SEPOLIA_RPC_URL, URL};
use num_traits::ToPrimitive;
use sncast::helpers::{constants::OZ_CLASS_HASH, devnet_provider::DevnetProvider};

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
