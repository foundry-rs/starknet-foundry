use crate::response::{
    cast_message::SncastMessage, command::CommandResponse, errors::ResponseError,
    explorer_link::ExplorerLinksMessage,
};
use anyhow::Result;
use foundry_ui::{Message, UI};

pub fn process_command_result<T>(
    command: &str,
    result: Result<T>,
    ui: &UI,
    block_explorer_link: Option<ExplorerLinksMessage>,
) -> Result<()>
where
    T: CommandResponse,
    SncastMessage<T>: Message,
{
    let cast_msg = result.map(|command_response| SncastMessage {
        command: command.to_string(),
        command_response,
    });

    match cast_msg {
        Ok(response) => {
            ui.println(&response);
            if let Some(link) = block_explorer_link {
                ui.println(&link);
            }
            Ok(())
        }
        Err(err) => {
            let response_err = ResponseError::new(command.to_string(), format!("{err:#}"));
            ui.eprintln(&response_err);
            Err(err)
        }
    }
}
