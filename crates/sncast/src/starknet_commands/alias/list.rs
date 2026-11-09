use anyhow::Result;
use sncast::helpers::command::process_command_result;
use std::process::ExitCode;

use sncast::helpers::configuration::CastConfig;
use sncast::response::alias::list::AliasesListMessage;
use sncast::response::ui::UI;

#[expect(clippy::unnecessary_wraps)]
pub fn list(config: &CastConfig, ui: &UI) -> Result<ExitCode> {
    Ok(process_command_result(
        "alias list",
        Ok(AliasesListMessage::new(&config.aliases)),
        ui,
        None,
    ))
}
