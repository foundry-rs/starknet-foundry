use crate::response::cast_message::SncastCommandMessage;
use conversions::serde::serialize::CairoSerialize;
use foundry_ui::styling;
use serde::{Deserialize, Serialize};
use starknet_types_core::felt::Felt;

#[derive(Serialize, Deserialize, CairoSerialize, Clone, Debug, PartialEq)]
pub struct SerializeResponse {
    pub calldata: Vec<Felt>,
}

impl SncastCommandMessage for SerializeResponse {
    fn text(&self) -> String {
        let calldata = format!("{:?}", &self.calldata);

        styling::OutputBuilder::new()
            .field("Calldata", &calldata)
            .build()
    }
}
