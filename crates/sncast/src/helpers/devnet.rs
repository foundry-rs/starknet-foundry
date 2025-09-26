use crate::AccountData;
use ::serde::{Deserialize, Serialize, de::DeserializeOwned};
use anyhow::Error;
use reqwest::Client;
use serde_json::json;
use starknet_types_core::felt::Felt;
use url::Url;

/// A Devnet-RPC client.
#[derive(Debug, Clone)]
pub struct DevnetClient {
    client: Client,
    url: Url,
}

/// All Devnet-RPC methods as listed in the official docs.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DevnetClientMethod {
    /// The `devnet_getConfig` method.
    #[serde(rename = "devnet_getConfig")]
    GetConfig,
    /// The `devnet_getPredeployedAccounts` method.
    #[serde(rename = "devnet_getPredeployedAccounts")]
    GetPredeployedAccounts,
    /// The `devnet_mint` method.
    #[serde(rename = "devnet_mint")]
    Mint,
}

impl DevnetClient {
    /// Constructs a new [`DevnetClient`] from a transport.
    #[must_use]
    pub fn new(url: &str) -> Self {
        let url = Url::parse(url).expect("Invalid URL");
        Self {
            client: Client::new(),
            url,
        }
    }
}

impl DevnetClient {
    async fn send_request<P, R>(&self, method: DevnetClientMethod, params: P) -> Result<R, Error>
    where
        P: Serialize + Send + Sync,
        R: DeserializeOwned,
    {
        let res = self
            .client
            .post(self.url.clone())
            .header("Content-Type", "application/json")
            .json(&json!({
                "jsonrpc": "2.0",
                "method": method,
                "params": params,
                "id": 1,
            }))
            .send()
            .await
            .expect("Error occured during request")
            .json::<serde_json::Value>()
            .await;

        match res {
            Ok(res_body) => {
                if let Some(error) = res_body.get("error") {
                    Err(anyhow::anyhow!("RPC error: {error}"))
                } else if let Some(result) = res_body.get("result") {
                    serde_json::from_value(result.clone()).map_err(anyhow::Error::from)
                } else {
                    Err(anyhow::anyhow!("Malformed RPC response: {res_body}"))
                }
            }
            Err(e) => Err(anyhow::anyhow!(e.to_string())),
        }
    }

    pub async fn get_config(&self) -> Result<Config, Error> {
        self.send_request(DevnetClientMethod::GetConfig, json!({}))
            .await
    }

    pub async fn get_predeployed_accounts(&self) -> Result<Vec<PredeployedAccount>, Error> {
        self.send_request(DevnetClientMethod::GetPredeployedAccounts, json!({}))
            .await
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub seed: u32,
    pub account_contract_class_hash: Felt,
    pub total_accounts: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PredeployedAccount {
    pub address: Felt,
    pub private_key: Felt,
    pub public_key: Felt,
}

impl From<&PredeployedAccount> for AccountData {
    fn from(predeployed_account: &PredeployedAccount) -> Self {
        Self {
            address: Some(predeployed_account.address),
            private_key: predeployed_account.private_key,
            public_key: predeployed_account.public_key,
            class_hash: None,
            salt: None,
            deployed: None,
            legacy: None,
            account_type: None,
        }
    }
}

#[tokio::test]
async fn test_get_config() {
    // let url = "http://127.0.0.1:5050/rpc";
    // let client = DevnetClient::new(url);

    // let config = client.get_config().await.expect("sdsd");

    // let predeployed_accounts = client.get_predeployed_accounts().await.expect("sdsd");
}
