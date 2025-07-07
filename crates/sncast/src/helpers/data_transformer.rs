use crate::response::{call::CallResponse, transformed_call::TransformedCallResponse};
use anyhow::Result;
use data_transformer::reverse_transform_output;
use starknet::core::types::{ContractClass, contract::AbiEntry};
use starknet_types_core::felt::Felt;

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
