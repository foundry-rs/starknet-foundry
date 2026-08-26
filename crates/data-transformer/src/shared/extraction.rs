use starknet_rust::core::types::EntryPointType;
use starknet_rust::core::types::contract::{AbiEntry, AbiFunction, StateMutability};
use starknet_rust::core::utils::get_selector_from_name;
use starknet_types_core::felt::Felt;

const CONSTRUCTOR_AS_SELECTOR: Felt =
    Felt::from_hex_unchecked("0x28ffe4ff0f226a9107253e17a904099aa4f63a02a5621de0576e5aa71bc5194");

#[must_use]
pub fn extract_function_from_selector(
    abi: &[AbiEntry],
    searched_selector: Felt,
) -> Option<AbiFunction> {
    search_for_function(abi, searched_selector)
        // If the user doesn't explicitly define a constructor in the contract,
        // it won't be present in the ABI. In such cases, an implicit constructor
        // with no arguments is assumed.
        .or_else(|| (searched_selector == CONSTRUCTOR_AS_SELECTOR).then(default_constructor))
}

#[must_use]
pub fn extract_entry_point_from_selector(
    abi: &[AbiEntry],
    searched_selector: Felt,
    entry_point_type: EntryPointType,
) -> Option<AbiFunction> {
    search_for_entry_point(abi, searched_selector, entry_point_type).or_else(|| {
        (entry_point_type == EntryPointType::Constructor
            && searched_selector == CONSTRUCTOR_AS_SELECTOR)
            .then(default_constructor)
    })
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

fn search_for_entry_point(
    abi: &[AbiEntry],
    searched_selector: Felt,
    entry_point_type: EntryPointType,
) -> Option<AbiFunction> {
    abi.iter()
        .find_map(|entry| match (entry_point_type, entry) {
            (EntryPointType::External, AbiEntry::Function(function))
            | (EntryPointType::L1Handler, AbiEntry::L1Handler(function)) => {
                let selector = get_selector_from_name(&function.name).ok()?;
                (selector == searched_selector).then(|| function.clone())
            }
            (EntryPointType::Constructor, AbiEntry::Constructor(constructor)) => {
                let selector = get_selector_from_name(&constructor.name).ok()?;
                (selector == searched_selector).then(|| AbiFunction {
                    name: constructor.name.clone(),
                    inputs: constructor.inputs.clone(),
                    outputs: vec![],
                    state_mutability: StateMutability::View,
                })
            }
            (_, AbiEntry::Interface(interface)) => {
                search_for_entry_point(&interface.items, searched_selector, entry_point_type)
            }
            _ => None,
        })
}

#[cfg(test)]
mod tests {
    use super::{extract_entry_point_from_selector, extract_function_from_selector};
    use starknet_rust::core::types::EntryPointType;
    use starknet_rust::core::types::contract::{
        AbiEntry, AbiFunction, AbiNamedMember, StateMutability,
    };
    use starknet_rust::core::utils::get_selector_from_name;

    fn entry(name: &str, input_type: &str) -> AbiFunction {
        AbiFunction {
            name: name.to_string(),
            inputs: vec![AbiNamedMember {
                name: "value".to_string(),
                r#type: input_type.to_string(),
            }],
            outputs: vec![],
            state_mutability: StateMutability::External,
        }
    }

    #[test]
    fn entry_point_lookup_distinguishes_function_from_l1_handler() {
        let name = "shared_name";
        let selector = get_selector_from_name(name).unwrap();
        let function = entry(name, "core::integer::u8");
        let l1_handler = entry(name, "core::integer::u16");

        for abi in [
            vec![
                AbiEntry::L1Handler(l1_handler.clone()),
                AbiEntry::Function(function.clone()),
            ],
            vec![
                AbiEntry::Function(function.clone()),
                AbiEntry::L1Handler(l1_handler.clone()),
            ],
        ] {
            let regular_lookup = extract_function_from_selector(&abi, selector).unwrap();
            assert_eq!(regular_lookup.inputs[0].r#type, "core::integer::u8");

            let external =
                extract_entry_point_from_selector(&abi, selector, EntryPointType::External)
                    .unwrap();
            assert_eq!(external.inputs[0].r#type, "core::integer::u8");

            let l1_handler =
                extract_entry_point_from_selector(&abi, selector, EntryPointType::L1Handler)
                    .unwrap();
            assert_eq!(l1_handler.inputs[0].r#type, "core::integer::u16");
        }

        let l1_handler_only = [AbiEntry::L1Handler(l1_handler)];
        assert!(extract_function_from_selector(&l1_handler_only, selector).is_none());
    }
}
