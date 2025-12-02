use crate::response::cast_message::SncastCommandMessage;
use conversions::padded_felt::PaddedFelt;
use conversions::{serde::serialize::CairoSerialize, string::IntoPaddedHexStr};
use foundry_ui::styling;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, CairoSerialize, Debug, PartialEq)]
pub struct ClassHashResponse {
    pub class_hash: PaddedFelt,
}

impl SncastCommandMessage for ClassHashResponse {
    fn text(&self) -> String {
        styling::OutputBuilder::new()
            .field("Class Hash", &self.class_hash.into_padded_hex_str())
            .build()
    }
}
