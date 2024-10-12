use super::base::VerificationInterface;
use async_trait::async_trait;
use camino::Utf8PathBuf;
use sncast::Network;
use std::env;

pub struct VoyagerVerificationInterface {
    pub network: Network,
    pub workspace_dir: Utf8PathBuf,
}

impl VoyagerVerificationInterface {
    pub fn new(network: Network, workspace_dir: Utf8PathBuf) -> Self {
        VoyagerVerificationInterface {
            network,
            workspace_dir,
        }
    }
}

#[async_trait]
impl VerificationInterface for VoyagerVerificationInterface {
    fn get_workspace_dir(&self) -> Utf8PathBuf {
        self.workspace_dir.clone()
    }

    fn gen_explorer_url(&self) -> String {
        let custom_api_url = env::var("VOYAGER_API_URL");
        if let Ok(custom_api_url) = custom_api_url {
            return custom_api_url;
        }

        let api_verification_url = match self.network {
            Network::Mainnet => "https://api.voyager.online/beta/class-verify-v2",
            Network::Sepolia => "https://sepolia-api.voyager.online/beta/class-verify-v2",
        };

        api_verification_url.to_string()
    }
}
