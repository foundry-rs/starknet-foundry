use foundry_ui::{Message, OutputFormat};
use serde::Serialize;

use crate::NumbersFormat;

use super::{
    command::CommandResponse,
    print::{Format, OutputData},
};

#[derive(Serialize)]
pub struct CastMessage<T: Serialize> {
    pub command: String,
    pub numbers_format: NumbersFormat,
    pub message: T,
}

// TODO(#3391): This impl should be remove and the `Message` trait should be implemented for each response type
// individually. This is a temporary solution to avoid breaking changes in the UI.
impl<T> Message for CastMessage<T>
where
    T: CommandResponse,
{
    #[must_use]
    fn text(&self) -> String {
        OutputData::from(&self.message)
            .format_with(self.numbers_format)
            .to_string_pretty(&self.command, OutputFormat::Human)
            .expect("Failed to format response")
    }

    #[must_use]
    fn json(&self) -> String {
        OutputData::from(&self.message)
            .format_with(self.numbers_format)
            .to_string_pretty(&self.command, OutputFormat::Json)
            .expect("Failed to format response")
    }
}
