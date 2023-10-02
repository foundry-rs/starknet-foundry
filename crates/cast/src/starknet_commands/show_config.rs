use anyhow::Result;
use camino::Utf8PathBuf;
use cast::helpers::response_structs::ShowConfigResponse;
use cast::helpers::scarb_utils::CastConfig;
use clap::Args;

#[derive(Args)]
#[command(about = "Show current configuration being used", long_about = None)]
pub struct ShowConfig {}

#[allow(clippy::ptr_arg)]
pub async fn show_config(
    cast_config: CastConfig,
    profile: String,
    scarb_path: Utf8PathBuf,
    network: String,
) -> Result<ShowConfigResponse> {
    Ok(ShowConfigResponse {
        profile,
        scarb_path,
        network,
        rpc_url: cast_config.rpc_url,
        account: cast_config.account,
        account_file_path: cast_config.accounts_file,
        keystore: cast_config.keystore,
    })
}
