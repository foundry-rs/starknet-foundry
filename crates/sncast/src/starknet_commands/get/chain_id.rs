use anyhow::Result;
use clap::Args;
use sncast::{
    get_chain_id,
    helpers::{command::process_command_result, configuration::CastConfig, rpc::RpcArgs},
    response::{get::chain_id::ChainIdResponse, ui::UI},
};
use starknet_rust::core::utils::parse_cairo_short_string;
use starknet_rust_crypto::Felt;
use std::process::ExitCode;

#[derive(Args, Debug)]
pub struct ChainId {
    #[command(flatten)]
    pub rpc: RpcArgs,
}

pub async fn chain_id(chain_id: ChainId, config: CastConfig, ui: &UI) -> Result<ExitCode> {
    let provider = chain_id.rpc.get_provider(&config, ui).await?;

    let chain_id = get_chain_id(&provider).await?;

    let chain_name = if chain_id != Felt::ZERO
        && let Ok(chain_name) = parse_cairo_short_string(&chain_id)
        && chain_name.chars().all(|c| c.is_ascii_graphic())
    {
        Some(chain_name)
    } else {
        None
    };

    let result = ChainIdResponse {
        chain_name,
        chain_id,
    };

    Ok(process_command_result("get chain-id", Ok(result), ui, None))
}
