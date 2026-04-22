use super::explorer::{ContractIdentifier, VerificationInterface};
use anyhow::Result;
use camino::Utf8PathBuf;
use foundry_ui::components::warning::WarningMessage;
use scarb_metadata::Metadata;
use sncast::Network;
use sncast::response::explorer_link::ExplorerError;
use sncast::response::ui::UI;
use sncast::{helpers::scarb_utils, response::verify::VerifyResponse};
use starknet_rust::{
    core::types::{BlockId, BlockTag},
    providers::{
        Provider,
        jsonrpc::{HttpTransport, JsonRpcClient},
    },
};
use starknet_types_core::felt::Felt;
use std::{collections::HashMap, env};
use url::Url;
use voyager_verifier::{
    api::ApiClient,
    core::project::ProjectType,
    voyager::{
        MAINNET_API_URL, SEPOLIA_API_URL, collect_verification_files, prepare_verification_request,
        submit_verification_request,
    },
};

pub struct Voyager<'a> {
    network: Network,
    metadata: Metadata,
    provider: &'a JsonRpcClient<HttpTransport>,
}

impl Voyager<'_> {
    pub fn gather_files(
        &self,
        include_test_files: bool,
    ) -> Result<(Utf8PathBuf, HashMap<String, Utf8PathBuf>)> {
        let files = collect_verification_files(&self.metadata, include_test_files)?;

        Ok((files.prefix, files.files))
    }
}

#[async_trait::async_trait]
impl<'a> VerificationInterface<'a> for Voyager<'a> {
    fn new(
        network: Network,
        workspace_dir: Utf8PathBuf,
        provider: &'a JsonRpcClient<HttpTransport>,
        _ui: &'a UI,
    ) -> Result<Self> {
        let manifest_path = scarb_utils::get_scarb_manifest_for(workspace_dir.as_ref())?;
        let metadata = scarb_utils::get_scarb_metadata_with_deps(&manifest_path)?;
        Ok(Voyager {
            network,
            metadata,
            provider,
        })
    }

    async fn verify(
        &self,
        contract_identifier: ContractIdentifier,
        contract_name: String,
        package: Option<String>,
        test_files: bool,
        ui: &UI,
    ) -> Result<VerifyResponse> {
        let class_hash = match contract_identifier {
            ContractIdentifier::ClassHash { class_hash } => Felt::from_hex(class_hash.as_ref())?,
            ContractIdentifier::Address { contract_address } => {
                self.provider
                    .get_class_hash_at(
                        BlockId::Tag(BlockTag::Latest),
                        Felt::from_hex(contract_address.as_ref())?,
                    )
                    .await?
            }
        };

        let prepared = prepare_verification_request(
            &self.metadata,
            &contract_name,
            package.as_deref(),
            test_files,
            ProjectType::Scarb,
            None,
        )?;

        if prepared.package.manifest_metadata.license.is_none() {
            ui.print_warning(WarningMessage::new("License not specified in Scarb.toml"));
        }

        let explorer_url = self.gen_explorer_url()?;
        let api_base_url = Url::parse(&explorer_url)?;
        let api_client = ApiClient::new(api_base_url.clone())?;
        let class_hash = format!("{class_hash:#066x}");
        let job_id =
            submit_verification_request(&api_base_url, &class_hash, &prepared.request).await?;
        let status_url = api_client.get_job_status_url(&job_id)?;
        let message = format!(
            "{contract_name} submitted for verification, you can query the status at: {status_url}"
        );

        Ok(VerifyResponse { message })
    }

    fn gen_explorer_url(&self) -> Result<String> {
        match env::var("VERIFIER_API_URL") {
            Ok(addr) => Ok(addr),
            Err(_) => match self.network {
                Network::Mainnet => Ok(MAINNET_API_URL.to_string()),
                Network::Sepolia => Ok(SEPOLIA_API_URL.to_string()),
                Network::Devnet => Err(ExplorerError::DevnetNotSupported.into()),
            },
        }
    }
}
