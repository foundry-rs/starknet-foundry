use super::base::VerificationInterface;
use async_trait::async_trait;
use sncast::{helpers::configuration::CastConfig, Network};

pub struct VoyagerVerificationInterface {
    pub network: Network,
}

impl VoyagerVerificationInterface {
    pub fn new(network: Network) -> Self {
        VoyagerVerificationInterface { network }
    }
}

#[async_trait]
impl VerificationInterface for VoyagerVerificationInterface {
    fn gen_explorer_url(&self, config: CastConfig) -> String {
        let base_api_url = match config.verification_base_url {
            Some(custom_base_api_url) => custom_base_api_url.clone(),
            None => match self.network {
                Network::Mainnet => "https://api.voyager.online/beta".to_string(),
                Network::Sepolia => "https://sepolia-api.voyager.online/beta".to_string(),
            },
        };

        format!("{base_api_url}/class-verify-v2")
    }
}
