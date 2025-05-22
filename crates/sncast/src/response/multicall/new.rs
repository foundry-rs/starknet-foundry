use camino::Utf8PathBuf;
use foundry_ui::{Message, OutputFormat};
use serde::Serialize;

use crate::response::{
    cast_message::CastMessage,
    command::CommandResponse,
    print::{Format, OutputData},
};

#[derive(Serialize)]
pub struct MulticallNewResponse {
    pub path: Utf8PathBuf,
    pub content: String,
}

impl CommandResponse for MulticallNewResponse {}

impl Message for MulticallNewResponse {}

impl CastMessage<MulticallNewResponse> {
    // TODO(#3391): Update text output to be more user friendly
    #[must_use]
    pub fn text(&self) -> String {
        OutputData::from(&self.message)
            .format_with(self.numbers_format)
            .to_string_pretty("multicall new", OutputFormat::Human)
            .expect("Failed to format response")
    }

    #[must_use]
    pub fn json(&self) -> String {
        OutputData::from(&self.message)
            .format_with(self.numbers_format)
            .to_string_pretty("multicall new", OutputFormat::Json)
            .expect("Failed to format response")
    }
}
