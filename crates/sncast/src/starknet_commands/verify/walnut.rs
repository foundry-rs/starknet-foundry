use super::base::VerificationInterface;
use async_trait::async_trait;
use sncast::Network;

pub struct WalnutVerificationInterface {
    pub base_url: String,
    pub network: Network,
}

impl WalnutVerificationInterface {
    pub fn new(network: Network, base_url: Option<String>) -> Self {
        let base_url = match base_url {
            Some(custom_base_api_url) => custom_base_api_url.clone(),
            None => "https://api.walnut.dev".to_string(),
        };

        WalnutVerificationInterface { base_url, network }
    }
}

#[async_trait]
impl VerificationInterface for WalnutVerificationInterface {
    fn explorer_url(&self) -> String {
        let path = match self.network {
            Network::Mainnet => "/v1/sn_main/verify",
            Network::Sepolia => "/v1/sn_sepolia/verify",
        };
        format!("{}{}", self.base_url, path)
    }
}
