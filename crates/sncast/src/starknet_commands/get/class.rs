use std::process::ExitCode;

use anyhow::{Context, Result};
use clap::Args;
use conversions::padded_felt::PaddedFelt;
use sncast::{
    get_block_id, get_class_hash_by_address,
    helpers::{
        block_explorer::LinkProvider, command::process_command_result, configuration::CastConfig,
        rpc::RpcArgs,
    },
    response::{
        errors::{StarknetCommandError, handle_starknet_command_error},
        explorer_link::{OutputLink, block_explorer_link_if_allowed},
        get::class::ClassResponse,
        ui::UI,
    },
};
use starknet_rust::{
    core::types::{BlockId, ContractClass, contract::legacy::LegacyContractClass},
    providers::{JsonRpcClient, Provider, jsonrpc::HttpTransport},
};
use starknet_rust_crypto::Felt;

use crate::starknet_commands::utils::felt_or_id::{ClassHash, ContractAddress};

#[derive(Args, Debug)]
pub struct Class {
    #[command(flatten)]
    pub rpc: RpcArgs,

    /// Block identifier from which class should be fetched
    /// Possible values: `pre_confirmed`, `latest`, block hash (0x prefixed string)
    /// and block number (u64)
    #[arg(short, long, default_value = "pre_confirmed")]
    pub block_id: String,

    #[command(flatten)]
    pub selector: ClassSelector,
}

#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
pub struct ClassSelector {
    /// Class hash of a contract
    #[arg(short = 'g', long)]
    pub hash: Option<ClassHash>,

    /// Address of the contract (hex, decimal, or @alias from snfoundry.toml)
    #[arg(short = 'd', long)]
    pub address: Option<ContractAddress>,
}

impl ClassSelector {
    async fn get_class(
        &self,
        block_id: BlockId,
        provider: &JsonRpcClient<HttpTransport>,
        config: &CastConfig,
    ) -> Result<ClassResponse> {
        if let Some(hash) = self.hash.as_ref() {
            let hash = hash.resolve(config)?;

            return provider
                .get_class(block_id, hash)
                .await
                .map(ClassResponse)
                .map_err(|err| StarknetCommandError::ProviderError(err.into()))
                .map_err(handle_starknet_command_error);
        }

        if let Some(address) = self.address.as_ref() {
            let address = address.resolve(config)?;

            return provider
                .get_class_at(block_id, address)
                .await
                .map(ClassResponse)
                .map_err(|err| StarknetCommandError::ProviderError(err.into()))
                .map_err(handle_starknet_command_error);
        }

        unreachable!("Either `hash` or `address` is `Some`");
    }

    /// Resolves class hash.
    ///
    /// Returns first successful result in order:
    /// 1. From arguments
    /// 2. From `get_class_at` result
    /// 3. From call to `get_class_hash_at`
    async fn get_class_hash(
        &self,
        result: Option<&ClassResponse>,
        provider: &JsonRpcClient<HttpTransport>,
        config: &CastConfig,
    ) -> Result<Felt> {
        if let Some(hash) = self.hash.as_ref() {
            return hash.resolve(config);
        }

        if let Ok(hash) = get_class_hash_from_response(result) {
            return Ok(hash);
        }

        let address = self
            .address
            .as_ref()
            .expect("Either `hash` or `address` is `Some`")
            .resolve(config)?;

        get_class_hash_by_address(provider, address).await
    }
}

pub async fn class(args: Class, config: CastConfig, ui: &UI) -> Result<ExitCode> {
    let block_id = get_block_id(&args.block_id)?;
    let provider = args.rpc.get_provider(&config, ui).await?;

    let result = args.selector.get_class(block_id, &provider, &config).await;

    let chain_id = provider.chain_id().await?;

    let class_hash_link = args
        .selector
        .get_class_hash(result.as_ref().ok(), &provider, &config)
        .await
        .map(ClassHashLink);

    let block_explorer_link =
        block_explorer_link_if_allowed(&class_hash_link, chain_id, &config).await;

    Ok(process_command_result(
        "get class",
        result,
        ui,
        block_explorer_link,
    ))
}

#[derive(Clone, Debug)]
struct ClassHashLink(Felt);

impl OutputLink for ClassHashLink {
    const TITLE: &str = "class";

    fn format_links(&self, provider: Box<dyn LinkProvider>) -> String {
        format!("class: {}", provider.class(PaddedFelt(self.0)))
    }
}

fn get_class_hash_from_response(result: Option<&ClassResponse>) -> Result<Felt> {
    let response = result.context("Missing response")?;

    match &response.0 {
        ContractClass::Sierra(class) => Ok(class.class_hash()),
        ContractClass::Legacy(class) => {
            let class: LegacyContractClass = class.clone().try_into()?;
            class.class_hash().context("Failed to compute class hash")
        }
    }
}
