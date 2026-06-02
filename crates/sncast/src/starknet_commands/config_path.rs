use clap::Args;
use sncast::helpers::config::ConfigFilePaths;
use sncast::response::config_path::ConfigPathResponse;

#[derive(Args)]
#[command(
    about = "Show paths to the config files contributing to the effective configuration",
    long_about = None
)]
pub struct ConfigPath {}

#[must_use]
pub fn config_path(paths: ConfigFilePaths) -> ConfigPathResponse {
    ConfigPathResponse {
        local_config: paths.local,
        global_config: paths.global,
    }
}
