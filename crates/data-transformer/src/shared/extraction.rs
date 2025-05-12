use starknet::core::types::contract::{AbiEntry, AbiFunction, StateMutability};
use starknet::core::utils::get_selector_from_name;
use starknet_types_core::felt::Felt;

pub fn extract_function_from_selector(
    abi: &[AbiEntry],
    searched_selector: Felt,
) -> Option<AbiFunction> {
    abi.iter().find_map(|entry| match entry {
        AbiEntry::Function(func) => {
            let selector = get_selector_from_name(&func.name).ok()?;
            (selector == searched_selector).then(|| func.clone())
        }
        // We treat constructor like a regular function
        // because it's searched for using Felt entrypoint selector, identically as functions.
        // Also, we don't need any constructor-specific properties, just argument types.
        AbiEntry::Constructor(constructor) => {
            let selector = get_selector_from_name(&constructor.name).ok()?;
            (selector == searched_selector).then(|| AbiFunction {
                name: constructor.name.clone(),
                inputs: constructor.inputs.clone(),
                outputs: vec![],
                state_mutability: StateMutability::View,
            })
        }
        AbiEntry::Interface(interface) => {
            extract_function_from_selector(&interface.items, searched_selector)
        }
        _ => None,
    })
}
