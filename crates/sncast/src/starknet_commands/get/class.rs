use std::process::ExitCode;

use anyhow::Result;
use clap::Args;
use sncast::{
    get_block_id,
    helpers::{command::process_command_result, configuration::CastConfig, rpc::RpcArgs},
    response::{get::class::ClassResponse, ui::UI},
};
use starknet_rust::providers::Provider;

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
    let provider = args.rpc.get_provider(&config, ui).await?;

    let class_hash = args.hash.resolve(&config)?;
    let class = provider.get_class(block_id, class_hash).await?;

    Ok(process_command_result(
        "get class",
        Ok(ClassResponse(class)),
        ui,
        None,
    ))
}
