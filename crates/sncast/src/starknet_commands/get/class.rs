use std::process::ExitCode;

use anyhow::Result;
use clap::Args;
use conversions::padded_felt::PaddedFelt;
use sncast::{
    get_block_id,
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
use starknet_rust::providers::Provider;
use starknet_rust_crypto::Felt;

use crate::starknet_commands::utils::felt_or_id::ClassHash;

#[derive(Args, Debug)]
pub struct Class {
    #[command(flatten)]
    pub rpc: RpcArgs,

    /// Block identifier from which class should be fetched
    /// Possible values: `pre_confirmed`, `latest`, block hash (0x prefixed string)
    /// and block number (u64)
    #[arg(short, long, default_value = "pre_confirmed")]
    pub block_id: String,

    /// Class hash of a contract
    pub hash: ClassHash,
}

pub async fn class(args: Class, config: CastConfig, ui: &UI) -> Result<ExitCode> {
    let block_id = get_block_id(&args.block_id)?;
    let class_hash = args.hash.resolve(&config)?;
    let provider = args.rpc.get_provider(&config, ui).await?;

    let result = provider
        .get_class(block_id, class_hash)
        .await
        .map(ClassResponse)
        .map_err(|err| StarknetCommandError::ProviderError(err.into()))
        .map_err(handle_starknet_command_error);

    let chain_id = provider.chain_id().await?;

    let class_hash_link = args.hash.resolve(&config).map(ClassHashLink);
    let block_explorer_link =
        block_explorer_link_if_allowed(&class_hash_link, chain_id, &config).await;

    Ok(process_command_result(
        "get class",
        result,
        ui,
        block_explorer_link,
    ))
}

#[derive(Clone)]
struct ClassHashLink(Felt);

impl OutputLink for ClassHashLink {
    const TITLE: &str = "class";

    fn format_links(&self, provider: Box<dyn LinkProvider>) -> String {
        format!("class: {}", provider.class(PaddedFelt(self.0)))
    }
}
