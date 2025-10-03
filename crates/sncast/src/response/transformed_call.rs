use super::command::CommandResponse;
use crate::response::call::CallResponse;
use crate::response::cast_message::SncastMessage;
use anyhow::Result;
use conversions::string::IntoHexStr;
use data_transformer::reverse_transform_output;
use foundry_ui::Message;
use foundry_ui::styling;
use serde::Serialize;
use serde_json::Value;
use starknet::core::types::{ContractClass, contract::AbiEntry};
use starknet_types_core::felt::Felt;

#[derive(Serialize, Clone)]
pub struct TransformedCallResponse {
    pub response: String,
    pub response_raw: Vec<Felt>,
}

impl CommandResponse for TransformedCallResponse {}

impl Message for SncastMessage<TransformedCallResponse> {
    fn text(&self) -> String {
        let response_raw_values = self
            .command_response
            .response_raw
            .iter()
            .map(|felt| felt.into_hex_string())
            .collect::<Vec<_>>()
            .join(", ");

        styling::OutputBuilder::new()
            .success_message("Call completed")
            .blank_line()
            .field("Response", &self.command_response.response)
            .field("Response Raw", &format!("[{response_raw_values}]"))
            .build()
    }

    fn json(&self) -> Value {
        self.to_json()
    }
}

#[must_use]
pub fn transform_response(
    result: &Result<CallResponse>,
    contract_class: &ContractClass,
    selector: &Felt,
) -> Option<TransformedCallResponse> {
    let Ok(CallResponse { response, .. }) = result else {
        return None;
    };

    if response.is_empty() {
        return None;
    }

    let ContractClass::Sierra(sierra_class) = contract_class else {
        return None;
    };

    let abi: Vec<AbiEntry> = serde_json::from_str(sierra_class.abi.as_str()).ok()?;

    let transformed_response = reverse_transform_output(response, &abi, selector).ok()?;

    Some(TransformedCallResponse {
        response_raw: response.clone(),
        response: transformed_response,
    })
}
