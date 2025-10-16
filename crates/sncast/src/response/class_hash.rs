use crate::response::cast_message::SncastCommandMessage;
use crate::response::{cast_message::SncastMessage, command::CommandResponse};
use conversions::padded_felt::PaddedFelt;
use conversions::{serde::serialize::CairoSerialize, string::IntoPaddedHexStr};
use foundry_ui::styling;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, CairoSerialize, Debug, PartialEq)]
pub struct ClassHashResponse {
    pub class_hash: PaddedFelt,
}

impl CommandResponse for ClassHashResponse {}

impl SncastCommandMessage for SncastMessage<ClassHashResponse> {
    fn text(&self) -> String {
        styling::OutputBuilder::new()
            .field(
                "Class Hash",
                &self.command_response.class_hash.into_padded_hex_str(),
            )
            .build()
    }
}
