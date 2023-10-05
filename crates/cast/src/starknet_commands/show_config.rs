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
    profile: Option<String>,
    scarb_path: Option<Utf8PathBuf>,
    network: String,
) -> Result<ShowConfigResponse> {
    let rpc_url = Some(cast_config.rpc_url).filter(|p| !p.is_empty());
    let account = Some(cast_config.account).filter(|p| !p.is_empty());
    let account_file_path =
        Some(cast_config.accounts_file).filter(|p| p != &Utf8PathBuf::default());
    let keystore = Some(cast_config.keystore).filter(|p| p != &Utf8PathBuf::default());

    Ok(ShowConfigResponse {
        profile,
        scarb_path,
        network,
        rpc_url,
        account,
        account_file_path,
        keystore,
    })
}
