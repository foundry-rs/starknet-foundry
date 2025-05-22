use super::{
    cast_message::CastMessage,
    command::CommandResponse,
    print::{Format, OutputData},
};
use foundry_ui::{Message, formats::OutputFormat};
use serde::Serialize;
use starknet_types_core::felt::Felt;

#[derive(Serialize, Clone)]
pub struct TransformedCallResponse {
    pub response: String,
    pub response_raw: Vec<Felt>,
}

impl CommandResponse for TransformedCallResponse {}

impl Message for TransformedCallResponse {}

impl CastMessage<TransformedCallResponse> {
    // TODO(#3391): Update text output to be more user friendly
    #[must_use]
    pub fn text(&self) -> String {
        OutputData::from(&self.message)
            .format_with(self.numbers_format)
            .to_string_pretty("call", OutputFormat::Human)
            .expect("Failed to format response")
    }

    #[must_use]
    pub fn json(&self) -> String {
        OutputData::from(&self.message)
            .format_with(self.numbers_format)
            .to_string_pretty("call", OutputFormat::Json)
            .expect("Failed to format response")
    }
}
