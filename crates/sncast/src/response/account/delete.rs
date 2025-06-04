use serde::Serialize;

use crate::response::command::CommandResponse;

#[derive(Serialize, Clone)]
pub struct AccountDeleteResponse {
    pub result: String,
}

impl CommandResponse for AccountDeleteResponse {}

// TODO(#3391): Update text output to be more user friendly
// impl Message for SncastMessage<AccountDeleteResponse> {}
