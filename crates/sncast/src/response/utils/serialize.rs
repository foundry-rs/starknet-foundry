use crate::response::cast_message::SncastCommandMessage;
use crate::response::cast_message::SncastMessage;
use crate::response::command::CommandResponse;
use conversions::serde::serialize::CairoSerialize;
use foundry_ui::styling;
use serde::{Deserialize, Serialize};
use starknet_types_core::felt::Felt;

#[derive(Serialize, Deserialize, CairoSerialize, Clone, Debug, PartialEq)]
pub struct SerializeResponse {
    pub calldata: Vec<Felt>,
}

impl CommandResponse for SerializeResponse {}

impl SncastCommandMessage for SncastMessage<SerializeResponse> {
    fn text(&self) -> String {
        let calldata = format!("{:?}", &self.command_response.calldata);

        styling::OutputBuilder::new()
            .field("Calldata", &calldata)
            .build()
    }
}
