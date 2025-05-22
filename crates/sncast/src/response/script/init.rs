use serde::Serialize;

use crate::response::command::CommandResponse;

#[derive(Serialize, Clone)]
pub struct ScriptInitResponse {
    pub message: String,
}

impl CommandResponse for ScriptInitResponse {}

// TODO(#3391): Update text output to be more user friendly
// impl Message for SnastMessage<ScriptInitResponse> {}
