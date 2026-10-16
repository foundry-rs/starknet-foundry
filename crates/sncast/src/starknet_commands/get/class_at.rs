use clap::Args;
use starknet_rust::providers::Provider;
use std::process::ExitCode;

use anyhow::Result;
use sncast::{
    get_block_id,
    helpers::{command::process_command_result, configuration::CastConfig, rpc::RpcArgs},
    response::{get::class::ClassResponse, ui::UI},
};

use crate::starknet_commands::utils::felt_or_id::ContractAddress;

#[derive(Args, Debug)]
pub struct ClassAt {
    #[command(flatten)]
    rpc: RpcArgs,

    /// Block identifier from which class should be fetched
    /// Possible values: `pre_confirmed`, `latest`, block hash (0x prefixed string)
    /// and block number (u64)
    #[arg(short, long, default_value = "pre_confirmed")]
    pub block_id: String,

    /// Address of the contract (hex, decimal, or @alias from snfoundry.toml)
    pub contract_address: ContractAddress,
}

pub async fn class_at(args: ClassAt, config: CastConfig, ui: &UI) -> Result<ExitCode> {
    let block_id = get_block_id(&args.block_id)?;
    let contract_address = args.contract_address.try_into_felt()?;
    let provider = args.rpc.get_provider(&config, ui).await?;

    let class = provider.get_class_at(block_id, contract_address).await?;

    Ok(process_command_result(
        "get class-at",
        Ok(ClassResponse(class)),
        ui,
        None,
    ))
}
