use super::base::VerificationInterface;
use async_trait::async_trait;
use camino::Utf8PathBuf;
use sncast::{helpers::configuration::CastConfig, Network};

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
