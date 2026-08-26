mod event;
mod transform;
mod types;

pub use self::event::{ReverseTransformEventError, reverse_transform_event};
use crate::reverse_transformer::transform::{ReverseTransformer, TransformationError};
use crate::shared::extraction::{
    extract_entry_point_from_selector, extract_function_from_selector,
};
use cairo_lang_parser::utils::SimpleParserDatabase;
use starknet_rust::core::types::EntryPointType;
use starknet_rust::core::types::contract::AbiEntry;
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
    let function = extract_function_from_selector(abi, *function_selector)
        .ok_or(ReverseTransformError::FunctionNotFound(*function_selector))?;

    reverse_transform(
        input,
        abi,
        function.inputs.iter().map(|input| input.r#type.as_str()),
    )
}

/// Transforms entry point calldata into a Cairo-like representation of its arguments.
pub fn reverse_transform_entry_point_input(
    input: &[Felt],
    abi: &[AbiEntry],
    function_selector: &Felt,
    entry_point_type: EntryPointType,
) -> Result<String, ReverseTransformError> {
    let function = extract_entry_point_from_selector(abi, *function_selector, entry_point_type)
        .ok_or(ReverseTransformError::FunctionNotFound(*function_selector))?;

    reverse_transform(
        input,
        abi,
        function.inputs.iter().map(|input| input.r#type.as_str()),
    )
}

/// Transforms a call output into a Cairo-like string representation of the return values
pub fn reverse_transform_output(
    output: &[Felt],
    abi: &[AbiEntry],
    function_selector: &Felt,
) -> Result<String, ReverseTransformError> {
    let function = extract_function_from_selector(abi, *function_selector)
        .ok_or(ReverseTransformError::FunctionNotFound(*function_selector))?;

    reverse_transform(
        output,
        abi,
        function.outputs.iter().map(|output| output.r#type.as_str()),
    )
}

/// Transforms entry point output into a Cairo-like representation of its return values.
pub fn reverse_transform_entry_point_output(
    output: &[Felt],
    abi: &[AbiEntry],
    function_selector: &Felt,
    entry_point_type: EntryPointType,
) -> Result<String, ReverseTransformError> {
    let function = extract_entry_point_from_selector(abi, *function_selector, entry_point_type)
        .ok_or(ReverseTransformError::FunctionNotFound(*function_selector))?;

    reverse_transform(
        output,
        abi,
        function.outputs.iter().map(|output| output.r#type.as_str()),
    )
}

fn reverse_transform<'a>(
    felts: &[Felt],
    abi: &[AbiEntry],
    parameter_types: impl IntoIterator<Item = &'a str>,
) -> Result<String, ReverseTransformError> {
    let db = SimpleParserDatabase::default();
    let mut reverse_transformer = ReverseTransformer::new(felts, abi);

    Ok(parameter_types
        .into_iter()
        .map(|parameter_type| {
            Ok(reverse_transformer
                .parse_and_transform(parameter_type, &db)?
                .to_string())
        })
        .collect::<Result<Vec<String>, TransformationError>>()?
        .join(", "))
}
