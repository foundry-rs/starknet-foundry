use serde::Serialize;

use crate::response::command::CommandResponse;

#[derive(Serialize, Debug, Clone)]
pub struct ScriptRunResponse {
    pub status: String,
    pub message: Option<String>,
}

impl CommandResponse for ScriptRunResponse {}

// TODO(#3391): Update text output to be more user friendly
// impl Message for SncastMessage<ScriptRunResponse> {}
