use std::process::ExitCode;

use anyhow::{Context, Result};
use clap::Args;
use sncast::{
    get_block_id, get_class_hash_by_address,
    helpers::{command::process_command_result, configuration::CastConfig, rpc::RpcArgs},
    response::{
        errors::{StarknetCommandError, handle_starknet_command_error},
        explorer_link::block_explorer_link_if_allowed,
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
    /// Class hash of a contract (hex, decimal, or @alias from snfoundry.toml)
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
        match (self.hash.as_ref(), self.address.as_ref()) {
            (Some(hash), None) => {
                let hash = hash.resolve(config)?;
                let response = provider
                    .get_class(block_id, hash)
                    .await
                    .map_err(|err| StarknetCommandError::ProviderError(err.into()))
                    .map_err(handle_starknet_command_error)?;

                Ok(ClassResponse { response, hash })
            }
            (None, Some(address)) => {
                let address = address.resolve(config)?;
                let response = provider
                    .get_class_at(block_id, address)
                    .await
                    .map_err(|err| StarknetCommandError::ProviderError(err.into()))
                    .map_err(handle_starknet_command_error)?;

                let hash = if let Ok(hash) = get_class_hash_from_response(&response) {
                    Ok(hash)
                } else {
                    get_class_hash_by_address(provider, address).await
                }?;

                Ok(ClassResponse { response, hash })
            }
            _ => unreachable!("Either `hash` or `address` is `Some`"),
        }
    }
}

pub async fn class(args: Class, config: CastConfig, ui: &UI) -> Result<ExitCode> {
    let block_id = get_block_id(&args.block_id)?;
    let provider = args.rpc.get_provider(&config, ui).await?;

    let response = args.selector.get_class(block_id, &provider, &config).await;
    let chain_id = provider.chain_id().await?;

    let block_explorer_link = block_explorer_link_if_allowed(&response, chain_id, &config).await;

    Ok(process_command_result(
        "get class",
        response,
        ui,
        block_explorer_link,
    ))
}

fn get_class_hash_from_response(response: &ContractClass) -> Result<Felt> {
    match &response {
        ContractClass::Sierra(class) => Ok(class.class_hash()),
        ContractClass::Legacy(class) => {
            let class: LegacyContractClass = class.clone().try_into()?;
            class.class_hash().context("Failed to compute class hash")
        }
    }
}
