use crate::AccountData;
use ::serde::{Deserialize, Serialize, de::DeserializeOwned};
use anyhow::{Context, Error, ensure};
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
    #[serde(rename = "devnet_getConfig")]
    GetConfig,

    #[serde(rename = "devnet_getPredeployedAccounts")]
    GetPredeployedAccounts,
}

impl DevnetProvider {
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
    async fn send_request<P, R>(&self, method: DevnetProviderMethod, params: P) -> anyhow::Result<R>
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
            .context("Failed to send request")?
            .json::<serde_json::Value>()
            .await
            .context("Failed to parse response")?;

        if let Some(error) = res.get("error") {
            Err(anyhow::anyhow!(error.to_string()))
        } else if let Some(result) = res.get("result") {
            serde_json::from_value(result.clone()).map_err(anyhow::Error::from)
        } else {
            panic!("Malformed RPC response: {res}")
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

    /// Ensures the Devnet instance is alive.
    pub async fn ensure_alive(&self) -> Result<(), Error> {
        let is_alive = self
            .client
            .get(format!(
                "{}/is_alive",
                self.url.to_string().replace("/rpc", "")
            ))
            .send()
            .await
            .map(|res| res.status().is_success())
            .unwrap_or(false);

        ensure!(
            is_alive,
            "Node at {} is not responding to the Devnet health check (GET `/is_alive`). It may not be a Devnet instance or it may be down.",
            self.url
        );
        Ok(())
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
