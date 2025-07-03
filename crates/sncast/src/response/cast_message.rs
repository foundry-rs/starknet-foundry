use serde::Serialize;

use super::command::CommandResponse;
#[derive(Serialize)]
pub struct SncastMessage<T: CommandResponse> {
    pub command: String,
    pub command_response: T,
}
