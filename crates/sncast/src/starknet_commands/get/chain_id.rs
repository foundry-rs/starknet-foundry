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

    let result = get_chain_id(&provider)
        .await
        .map(|chain_id| ChainIdResponse {
            chain_name: (chain_id != Felt::ZERO)
                .then(|| parse_cairo_short_string(&chain_id).ok())
                .flatten(),
            chain_id,
        });

    Ok(process_command_result("get chain-id", result, ui, None))
}
