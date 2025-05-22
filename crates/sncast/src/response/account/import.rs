use foundry_ui::{Message, OutputFormat};
use serde::Serialize;

use crate::response::{
    cast_message::CastMessage,
    command::CommandResponse,
    print::{Format, OutputData},
};

#[derive(Serialize)]
pub struct AccountImportResponse {
    pub add_profile: String,
    pub account_name: Option<String>,
}

impl CommandResponse for AccountImportResponse {}

impl Message for AccountImportResponse {}

impl CastMessage<AccountImportResponse> {
    // TODO(#3391): Update text output to be more user friendly
    #[must_use]
    pub fn text(&self) -> String {
        OutputData::from(&self.message)
            .format_with(self.numbers_format)
            .to_string_pretty("account import", OutputFormat::Human)
            .expect("Failed to format response")
    }

    #[must_use]
    pub fn json(&self) -> String {
        OutputData::from(&self.message)
            .format_with(self.numbers_format)
            .to_string_pretty("account import", OutputFormat::Json)
            .expect("Failed to format response")
    }
}
