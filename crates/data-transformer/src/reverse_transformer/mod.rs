mod transform;
mod types;

use crate::reverse_transformer::transform::{ReverseTransformer, TransformationError};
use crate::shared::extraction::extract_function_from_selector;
use starknet::core::types::contract::AbiEntry;
use starknet_types_core::felt::Felt;

#[derive(Debug, thiserror::Error)]
pub enum ReverseTransformError {
    #[error(r#"Function with selector "{0:#x}" not found in ABI of the contract"#)]
    FunctionNotFound(Felt),
    #[error(transparent)]
    TransformationError(#[from] TransformationError),
}

/// Transforms a calldata into a Cairo-like string representation of the arguments
pub fn reverse_transform_input(
    input: &[Felt],
    abi: &[AbiEntry],
    function_selector: &Felt,
) -> Result<String, ReverseTransformError> {
    let input_types: Vec<_> = extract_function_from_selector(abi, *function_selector)
        .ok_or(ReverseTransformError::FunctionNotFound(*function_selector))?
        .inputs
        .into_iter()
        .map(|input| input.r#type)
        .collect();

    reverse_transform(input, abi, &input_types)
}

/// Transforms a call output into a Cairo-like string representation of the return values
pub fn reverse_transform_output(
    output: &[Felt],
    abi: &[AbiEntry],
    function_selector: &Felt,
) -> Result<String, ReverseTransformError> {
    let output_types: Vec<_> = extract_function_from_selector(abi, *function_selector)
        .ok_or(ReverseTransformError::FunctionNotFound(*function_selector))?
        .outputs
        .into_iter()
        .map(|input| input.r#type)
        .collect();

    reverse_transform(output, abi, &output_types)
}

fn reverse_transform(
    felts: &[Felt],
    abi: &[AbiEntry],
    types: &[String],
) -> Result<String, ReverseTransformError> {
    let mut reverse_transformer = ReverseTransformer::new(felts, abi);

    Ok(types
        .iter()
        .map(|parameter_type| {
            Ok(reverse_transformer
                .parse_and_transform(parameter_type)?
                .to_string())
        })
        .collect::<Result<Vec<String>, TransformationError>>()?
        .join(", "))
}
