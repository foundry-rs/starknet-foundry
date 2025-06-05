use foundry_ui::{Message, OutputFormat};
use serde::Serialize;
use serde_json::Value;

use crate::NumbersFormat;

use super::{
    command::CommandResponse,
    print::{Format, OutputData},
};

#[derive(Serialize)]
pub struct SncastMessage<T: CommandResponse> {
    pub command: String,
    pub command_response: T,
    pub numbers_format: NumbersFormat,
}

// TODO(#3391): This impl should be remove and the `Message` trait should be implemented for each response type
// individually. This is a temporary solution to avoid breaking changes in the UI.
impl<T> Message for SncastMessage<T>
where
    T: CommandResponse,
{
    #[must_use]
    fn text(&self) -> String {
        OutputData::from(&self.command_response)
            .format_with(self.numbers_format)
            .to_string_pretty(&self.command, OutputFormat::Human)
            .expect("Failed to format response")
    }

    #[must_use]
    fn json(&self) -> Value {
        serde_json::from_str(
            &OutputData::from(&self.command_response)
                .format_with(self.numbers_format)
                .to_string_pretty(&self.command, OutputFormat::Json)
                .expect("Failed to format response"),
        )
        .expect("Failed to parse JSON from response")
    }
}
