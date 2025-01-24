use anyhow::Result;
use camino::Utf8PathBuf;
// use reqwest::StatusCode;
use sncast::Network;
use sncast::{helpers::scarb_utils, response::structs::VerifyResponse};
use starknet_types_core::felt::Felt;
use std::env;
// use scarb_api::ScarbCommand;
use scarb_metadata::Metadata;
// use std::ffi::OsStr;
// use walkdir::WalkDir;

use super::explorer::{ContractIdentifier, VerificationInterface};

pub struct Voyager {
    network: Network,
    workspace_dir: Utf8PathBuf,
    metadata: Metadata,
}

#[async_trait::async_trait]
impl VerificationInterface for Voyager {
    fn new(network: Network, workspace_dir: Utf8PathBuf) -> Result<Self> {
        let manifest_path = scarb_utils::get_scarb_manifest_for(workspace_dir.as_ref())?;
        let metadata = scarb_utils::get_scarb_metadata_with_deps(&manifest_path)?;
        Ok(Voyager {
            network,
            workspace_dir,
            metadata,
        })
    }

    async fn verify(
        &self,
        _contract_identifier: ContractIdentifier,
        _contract_name: String,
        _package: Option<String>,
    ) -> Result<VerifyResponse> {
        todo!()
    }

    fn gen_explorer_url(&self) -> String {
        match env::var("VOYAGER_API_URL") {
            Ok(addr) => addr,
            Err(_) => match self.network {
                Network::Mainnet => "https://api.voyager.online/beta".to_string(),
                Network::Sepolia => "https://sepolia-api.voyager.online/beta".to_string(),
            },
        }
    }
}
