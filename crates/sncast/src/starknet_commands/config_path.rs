use anyhow::Result;
use clap::Args;
use configuration::find_config_file;
use sncast::helpers::config::resolve_global_config_path_or_warn;
use sncast::response::config_path::ConfigPathResponse;
use sncast::response::ui::UI;

#[derive(Args)]
#[command(about = "Show paths to the config files contributing to the effective configuration")]
pub struct ConfigPath {}

#[allow(clippy::unnecessary_wraps)]
pub fn config_path(ui: &UI) -> Result<ConfigPathResponse> {
    let local = find_config_file()
        .ok()
        .map(|path| path.canonicalize_utf8().unwrap_or(path));

    let global =
        resolve_global_config_path_or_warn(ui).map(|path| path.canonicalize_utf8().unwrap_or(path));

    Ok(ConfigPathResponse {
        local_config: local,
        global_config: global,
    })
}
