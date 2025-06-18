use serde::Serialize;

use crate::response::command::CommandResponse;

#[derive(Serialize, Clone)]
pub struct AccountImportResponse {
    pub add_profile: Option<String>,
    pub account_name: String,
}

impl CommandResponse for AccountImportResponse {}

// TODO(#3391): Update text output to be more user friendly
// impl Message for SncastMessage<AccountImportResponse> {}
