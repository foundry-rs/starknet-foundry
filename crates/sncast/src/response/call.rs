use super::{
    cast_message::CastMessage,
    command::CommandResponse,
    print::{Format, OutputData},
};
use conversions::serde::serialize::CairoSerialize;
use foundry_ui::{Message, formats::OutputFormat};
use serde::Serialize;
use starknet_types_core::felt::Felt;

#[derive(Serialize, CairoSerialize, Clone)]
pub struct CallResponse {
    pub response: Vec<Felt>,
}

impl CommandResponse for CallResponse {}

impl Message for CallResponse {}

impl CastMessage<CallResponse> {
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
