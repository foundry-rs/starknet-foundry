use anyhow::Result;
use camino::Utf8PathBuf;
use clap::Args;
use sncast::helpers::configuration::CastConfig;
use sncast::helpers::rpc::RpcArgs;
use sncast::response::structs::{Decimal, ShowConfigResponse};
use sncast::{chain_id_to_network_name, get_chain_id};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;

#[derive(Args)]
#[command(about = "Show current configuration being used", long_about = None)]
pub struct ShowConfig {
    #[clap(flatten)]
    pub rpc: RpcArgs,
}

#[allow(clippy::ptr_arg)]
pub async fn show_config(
    show: &ShowConfig,
    provider: Option<&JsonRpcClient<HttpTransport>>,
    cast_config: CastConfig,
    profile: Option<String>,
) -> Result<ShowConfigResponse> {
    let chain_id = if let Some(provider) = provider {
        let chain_id_field = get_chain_id(provider).await?;
        Some(chain_id_to_network_name(chain_id_field))
    } else {
        None
    };

    let rpc_url = Some(show.rpc.url.clone().unwrap_or(cast_config.url)).filter(|p| !p.is_empty());
    let account = Some(cast_config.account).filter(|p| !p.is_empty());
    let mut accounts_file_path =
        Some(cast_config.accounts_file).filter(|p| p != &Utf8PathBuf::default());
    let keystore = cast_config.keystore;
    if keystore.is_some() {
        accounts_file_path = None;
    }
    let wait_timeout = Some(cast_config.wait_params.get_timeout());
    let wait_retry_interval = Some(cast_config.wait_params.get_retry_interval());
    let block_explorer = cast_config.block_explorer;

    Ok(ShowConfigResponse {
        profile,
        chain_id,
        rpc_url,
        account,
        accounts_file_path,
        keystore,
        wait_timeout: wait_timeout.map(|x| Decimal(u64::from(x))),
        wait_retry_interval: wait_retry_interval.map(|x| Decimal(u64::from(x))),
        show_explorer_links: cast_config.show_explorer_links,
        block_explorer,
    })
}
