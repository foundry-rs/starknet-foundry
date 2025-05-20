use starknet::core::types::contract::{AbiEntry, AbiFunction, StateMutability};
use starknet::core::utils::get_selector_from_name;
use starknet_types_core::felt::Felt;

pub fn extract_function_from_selector(
    abi: &[AbiEntry],
    searched_selector: Felt,
) -> Option<AbiFunction> {
    const CONSTRUCTOR_AS_SELECTOR: Felt = Felt::from_hex_unchecked(
        "0x28ffe4ff0f226a9107253e17a904099aa4f63a02a5621de0576e5aa71bc5194",
    );

    search_for_function(abi, searched_selector)
        // If the user doesn't explicitly define a constructor in the contract,
        // it won't be present in the ABI. In such cases, an implicit constructor
        // with no arguments is assumed.
        .or_else(|| (searched_selector == CONSTRUCTOR_AS_SELECTOR).then(default_constructor))
}

fn default_constructor() -> AbiFunction {
    AbiFunction {
        name: "constructor".to_string(),
        inputs: vec![],
        outputs: vec![],
        state_mutability: StateMutability::View,
    }
}

fn search_for_function(abi: &[AbiEntry], searched_selector: Felt) -> Option<AbiFunction> {
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
        AbiEntry::Interface(interface) => search_for_function(&interface.items, searched_selector),
        _ => None,
    })
}
