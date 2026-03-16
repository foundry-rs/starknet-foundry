use crate::response::cast_message::SncastCommandMessage;
use conversions::padded_felt::PaddedFelt;
use conversions::string::IntoPaddedHexStr;
use foundry_ui::styling;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct SelectorResponse {
    pub selector: PaddedFelt,
}

impl SncastCommandMessage for SelectorResponse {
    fn text(&self) -> String {
        styling::OutputBuilder::new()
            .field("Selector", &self.selector.into_padded_hex_str())
            .build()
    }
}
