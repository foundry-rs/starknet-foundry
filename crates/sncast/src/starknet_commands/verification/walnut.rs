use super::base::VerificationInterface;
use async_trait::async_trait;
use camino::Utf8PathBuf;
use sncast::Network;
use std::env;

pub struct WalnutVerificationInterface {
    pub network: Network,
    pub workspace_dir: Utf8PathBuf,
}

impl WalnutVerificationInterface {
    pub fn new(network: Network, workspace_dir: Utf8PathBuf) -> Self {
        WalnutVerificationInterface {
            network,
            workspace_dir,
        }
    }
}

#[async_trait]
impl VerificationInterface for WalnutVerificationInterface {
    fn get_workspace_dir(&self) -> Utf8PathBuf {
        self.workspace_dir.clone()
    }

    fn gen_explorer_url(&self) -> String {
        let api_base_url =
            env::var("WALNUT_API_URL").unwrap_or_else(|_| "https://api.walnut.dev".to_string());
        let path = match self.network {
            Network::Mainnet => "/v1/sn_main/verify",
            Network::Sepolia => "/v1/sn_sepolia/verify",
        };
        format!("{api_base_url}{path}")
    }
}
