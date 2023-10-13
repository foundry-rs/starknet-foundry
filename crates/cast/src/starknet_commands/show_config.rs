use anyhow::Result;
use camino::Utf8PathBuf;
use cast::helpers::response_structs::ShowConfigResponse;
use cast::helpers::scarb_utils::CastConfig;
use cast::{chain_id_to_network_name, get_chain_id};
use clap::Args;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;

#[derive(Args)]
#[command(about = "Show current configuration being used", long_about = None)]
pub struct ShowConfig {}

#[allow(clippy::ptr_arg)]
pub async fn show_config(
    provider: &JsonRpcClient<HttpTransport>,
    cast_config: CastConfig,
    profile: Option<String>,
    scarb_path: Option<Utf8PathBuf>,
) -> Result<ShowConfigResponse> {
    let chain_id_field = get_chain_id(provider).await?;
    let chain_id = chain_id_to_network_name(chain_id_field);
    let rpc_url = Some(cast_config.rpc_url).filter(|p| !p.is_empty());
    let account = Some(cast_config.account).filter(|p| !p.is_empty());
    let mut accounts_file_path =
        Some(cast_config.accounts_file).filter(|p| p != &Utf8PathBuf::default());
    let keystore = Some(cast_config.keystore).filter(|p| p != &Utf8PathBuf::default());

    if keystore.is_some() {
        accounts_file_path = None;
    }

    Ok(ShowConfigResponse {
        profile,
        chain_id,
        rpc_url,
        account,
        scarb_path,
        accounts_file_path,
        keystore,
    })
}
