use crate::response::cast_message::SncastCommandMessage;
use conversions::serde::serialize::CairoSerialize;
use conversions::string::IntoHexStr;
use foundry_ui::styling;
use serde::Serialize;
use starknet_types_core::felt::Felt;

#[derive(Serialize, CairoSerialize, Clone)]
pub struct CallResponse {
    pub response: Vec<Felt>,
}

impl SncastCommandMessage for CallResponse {
    fn text(&self) -> String {
        let response_values = self
            .response
            .iter()
            .map(|felt| felt.into_hex_string())
            .collect::<Vec<_>>()
            .join(", ");

        styling::OutputBuilder::new()
            .success_message("Call completed")
            .blank_line()
            .field("Response", &format!("[{response_values}]"))
            .build()
    }
}
