use crate::AccountData;
use ::serde::{Deserialize, Serialize, de::DeserializeOwned};
use anyhow::Error;
use reqwest::Client;
use serde_json::json;
use starknet_types_core::felt::Felt;
use url::Url;

/// A Devnet-RPC client.
#[derive(Debug, Clone)]
pub struct DevnetProvider {
    client: Client,
    url: Url,
}

/// All Devnet-RPC methods as listed in the official docs.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DevnetProviderMethod {
    /// The `devnet_getConfig` method.
    #[serde(rename = "devnet_getConfig")]
    GetConfig,

    /// The `devnet_getPredeployedAccounts` method.
    #[serde(rename = "devnet_getPredeployedAccounts")]
    GetPredeployedAccounts,
}

impl DevnetProvider {
    /// Constructs a new [`DevnetProvider`] from a transport.
    #[must_use]
    pub fn new(url: &str) -> Self {
        let url = Url::parse(url).expect("Invalid URL");
        Self {
            client: Client::new(),
            url,
        }
    }
}

impl DevnetProvider {
    async fn send_request<P, R>(&self, method: DevnetProviderMethod, params: P) -> Result<R, Error>
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
            .expect("Error occurred during request")
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

    /// Fetches the current Devnet configuration.
    pub async fn get_config(&self) -> Result<Config, Error> {
        self.send_request(DevnetProviderMethod::GetConfig, json!({}))
            .await
    }

    /// Fetches the list of predeployed accounts in Devnet.
    pub async fn get_predeployed_accounts(&self) -> Result<Vec<PredeployedAccount>, Error> {
        self.send_request(DevnetProviderMethod::GetPredeployedAccounts, json!({}))
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
