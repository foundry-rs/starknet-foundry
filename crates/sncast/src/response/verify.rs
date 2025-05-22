use serde::Serialize;

use super::command::CommandResponse;

#[derive(Serialize, Clone)]
pub struct VerifyResponse {
    pub message: String,
}

impl CommandResponse for VerifyResponse {}

// TODO(#3391): Update text output to be more user friendly
// impl Message for SnastMessage<VerifyResponse> {}
