use anyhow::Result;
use clap::Args;
use configuration::find_config_file;
use foundry_ui::components::warning::WarningMessage;
use sncast::helpers::config::get_or_create_global_config_path;
use sncast::response::config_path::ConfigPathResponse;
use sncast::response::ui::UI;

#[derive(Args)]
#[command(
    about = "Show paths to the config files contributing to the effective configuration",
    long_about = None
)]
pub struct ConfigPath {}

pub fn config_path(ui: &UI) -> Result<ConfigPathResponse> {
    let local = find_config_file()
        .ok()
        .map(|path| path.canonicalize_utf8().unwrap_or(path));

    // TODO: consider adding a wrapper-helper for this logic
    let global = match get_or_create_global_config_path() {
        Ok(path) => Some(path.canonicalize_utf8().unwrap_or(path)),
        Err(err) => {
            ui.print_warning(WarningMessage::new(format!(
                "Could not get or create global config file: {err:?}. Proceeding without global config."
            )));
            None
        }
    };

    Ok(ConfigPathResponse {
        local_config: local,
        global_config: global,
    })
}
