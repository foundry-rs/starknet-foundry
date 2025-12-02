use crate::response::cast_message::SncastMessage;
use crate::response::{errors::ResponseError, explorer_link::ExplorerLinksMessage};
use anyhow::Result;

use crate::response::cast_message::SncastCommandMessage;
use crate::response::ui::UI;
use foundry_ui::Message;
use serde::Serialize;
use std::process::ExitCode;

pub fn process_command_result<T>(
    command: &str,
    result: Result<T>,
    ui: &UI,
    block_explorer_link: Option<ExplorerLinksMessage>,
) -> ExitCode
where
    T: SncastCommandMessage + Serialize,
    SncastMessage<T>: Message,
{
    let cast_msg = result.map(|command_response| SncastMessage(command_response));

    match cast_msg {
        Ok(response) => {
            ui.print_message(command, response);
            if let Some(link) = block_explorer_link {
                ui.print_notification(link);
            }
            ExitCode::SUCCESS
        }
        Err(err) => {
            let err = ResponseError::new(command.to_string(), format!("{err:#}"));
            ui.print_error(command, err);
            ExitCode::FAILURE
        }
    }
}
