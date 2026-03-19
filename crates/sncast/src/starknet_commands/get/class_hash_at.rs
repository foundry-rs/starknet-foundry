use anyhow::Result;
use clap::Args;
use conversions::IntoConv;
use sncast::get_block_id;
use sncast::helpers::command::process_command_result;
use sncast::helpers::configuration::CastConfig;
use sncast::helpers::rpc::RpcArgs;
use sncast::response::class_hash_at::ClassHashAtResponse;
use sncast::response::errors::StarknetCommandError;
use sncast::response::explorer_link::block_explorer_link_if_allowed;
use sncast::response::ui::UI;
use starknet_rust::providers::jsonrpc::HttpTransport;
use starknet_rust::providers::{JsonRpcClient, Provider};
use starknet_types_core::felt::Felt;

#[derive(Debug, Args)]
#[command(about = "Get the class hash of a contract deployed at a given address")]
pub struct ClassHashAt {
    /// Address of the contract
    pub contract_address: Felt,

    /// Block identifier on which class hash should be fetched.
    /// Possible values: `pre_confirmed`, `latest`, block hash (0x prefixed string)
    /// and block number (u64)
    #[arg(short, long, default_value = "pre_confirmed")]
    pub block_id: String,

    #[command(flatten)]
    pub rpc: RpcArgs,
}

pub async fn class_hash_at(args: ClassHashAt, config: CastConfig, ui: &UI) -> anyhow::Result<()> {
    let provider = args.rpc.get_provider(&config, ui).await?;

    let result = get_class_hash_at(&provider, args.contract_address, &args.block_id).await;

    let chain_id = provider.chain_id().await?;
    let block_explorer_link = block_explorer_link_if_allowed(&result, chain_id, &config).await;

    process_command_result("get class-hash-at", result, ui, block_explorer_link);
    Ok(())
}

async fn get_class_hash_at(
    provider: &JsonRpcClient<HttpTransport>,
    contract_address: Felt,
    block_id: &str,
) -> Result<ClassHashAtResponse> {
    let block_id = get_block_id(block_id)?;

    let class_hash = provider
        .get_class_hash_at(block_id, contract_address)
        .await
        .map_err(|err| StarknetCommandError::ProviderError(err.into()))?;

    Ok(ClassHashAtResponse {
        class_hash: class_hash.into_(),
    })
}
