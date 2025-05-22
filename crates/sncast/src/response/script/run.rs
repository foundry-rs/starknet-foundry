use foundry_ui::{Message, OutputFormat};
use serde::Serialize;

use crate::response::{
    cast_message::CastMessage,
    command::CommandResponse,
    print::{Format, OutputData},
};

#[derive(Serialize, Debug)]
pub struct ScriptRunResponse {
    pub status: String,
    pub message: Option<String>,
}

impl CommandResponse for ScriptRunResponse {}

impl Message for ScriptRunResponse {}

impl CastMessage<ScriptRunResponse> {
    // TODO(#3391): Update text output to be more user friendly
    #[must_use]
    pub fn text(&self) -> String {
        OutputData::from(&self.message)
            .format_with(self.numbers_format)
            .to_string_pretty("script run", OutputFormat::Human)
            .expect("Failed to format response")
    }

    #[must_use]
    pub fn json(&self) -> String {
        OutputData::from(&self.message)
            .format_with(self.numbers_format)
            .to_string_pretty("script run", OutputFormat::Json)
            .expect("Failed to format response")
    }
}
