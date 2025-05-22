use foundry_ui::{Message, OutputFormat};
use serde::Serialize;

use super::{
    cast_message::CastMessage,
    command::CommandResponse,
    print::{Format, OutputData},
};

#[derive(Serialize)]
pub struct VerifyResponse {
    pub message: String,
}

impl CommandResponse for VerifyResponse {}

impl Message for VerifyResponse {}

impl CastMessage<VerifyResponse> {
    // TODO(#3391): Update text output to be more user friendly
    #[must_use]
    pub fn text(&self) -> String {
        OutputData::from(&self.message)
            .format_with(self.numbers_format)
            .to_string_pretty("verify", OutputFormat::Human)
            .expect("Failed to format response")
    }

    #[must_use]
    pub fn json(&self) -> String {
        OutputData::from(&self.message)
            .format_with(self.numbers_format)
            .to_string_pretty("verify", OutputFormat::Json)
            .expect("Failed to format response")
    }
}
