use super::base::VerificationInterface;
use async_trait::async_trait;
use sncast::{helpers::configuration::CastConfig, Network};

pub struct WalnutVerificationInterface {
    pub network: Network,
}

impl WalnutVerificationInterface {
    pub fn new(network: Network) -> Self {
        WalnutVerificationInterface { network }
    }
}

#[async_trait]
impl VerificationInterface for WalnutVerificationInterface {
    fn gen_explorer_url(&self, config: CastConfig) -> String {
        let api_base_url = match config.verification_base_url {
            Some(custom_base_api_url) => custom_base_api_url.clone(),
            None => "https://api.walnut.dev".to_string(),
        };

        let path = match self.network {
            Network::Mainnet => "/v1/sn_main/verify",
            Network::Sepolia => "/v1/sn_sepolia/verify",
        };
        format!("{api_base_url}{path}")
    }
}
