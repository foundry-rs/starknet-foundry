use super::base::VerificationInterface;
use async_trait::async_trait;
use sncast::Network;

pub struct VoyagerVerificationInterface {
    pub base_url: String,
}

#[async_trait]
impl VerificationInterface for VoyagerVerificationInterface {
    fn new(network: Network, base_url: Option<String>) -> Self {
        let base_url = match base_url {
            Some(custom_base_api_url) => custom_base_api_url.clone(),
            None => match network {
                Network::Mainnet => "https://api.voyager.online/beta".to_string(),
                Network::Sepolia => "https://sepolia-api.voyager.online/beta".to_string(),
            },
        };

        VoyagerVerificationInterface { base_url }
    }

    fn gen_explorer_url(&self) -> String {
        format!("{}/class-verify-v2", self.base_url)
    }
}
