use anyhow::Result;
use bimap::BiMap;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::declare::get_class_hash;
use std::collections::HashMap;

use conversions::IntoConv;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use scarb_api::StarknetContractArtifacts;
use starknet::core::types::contract::{AbiEntry, SierraClass};
use starknet::core::utils::get_selector_from_name;
use starknet_api::core::{ClassHash, EntryPointSelector};

#[derive(Debug, Clone)]
pub struct ContractsData {
    pub contracts: HashMap<String, StarknetContractArtifacts>,
    pub class_hashes: BiMap<String, ClassHash>,
    pub selectors: HashMap<EntryPointSelector, String>,
}

impl ContractsData {
    pub fn try_from(contracts: HashMap<String, StarknetContractArtifacts>) -> Result<Self> {
        let parsed_contracts: HashMap<String, SierraClass> = contracts
            .par_iter()
            .map(|(name, artifact)| Ok((name.clone(), serde_json::from_str(&artifact.sierra)?)))
            .collect::<Result<_>>()?;

        let class_hashes: Vec<(String, ClassHash)> = parsed_contracts
            .par_iter()
            .map(|(name, sierra_class)| Ok((name.clone(), get_class_hash(sierra_class)?)))
            .collect::<Result<_>>()?;

        let selectors = parsed_contracts
            .into_par_iter()
            .map(|(_, sierra_class)| build_name_selector_map(sierra_class.abi))
            .flatten()
            .collect();

        Ok(ContractsData {
            contracts,
            class_hashes: BiMap::from_iter(class_hashes),
            selectors,
        })
    }
}

fn build_name_selector_map(abi: Vec<AbiEntry>) -> HashMap<EntryPointSelector, String> {
    let mut selector_map = HashMap::new();
    for abi_entry in abi {
        match abi_entry {
            AbiEntry::Interface(abi_interface) => {
                for abi_entry in abi_interface.items {
                    add_simple_abi_entry_to_mapping(abi_entry, &mut selector_map);
                }
            }
            _ => add_simple_abi_entry_to_mapping(abi_entry, &mut selector_map),
        };
    }
    selector_map
}

fn add_simple_abi_entry_to_mapping(
    abi_entry: AbiEntry,
    selector_map: &mut HashMap<EntryPointSelector, String>,
) {
    match abi_entry {
        AbiEntry::Function(abi_function) | AbiEntry::L1Handler(abi_function) => {
            selector_map.insert(
                EntryPointSelector(get_selector_from_name(&abi_function.name).unwrap().into_()),
                abi_function.name,
            );
        }
        AbiEntry::Constructor(abi_constructor) => {
            selector_map.insert(
                EntryPointSelector(
                    get_selector_from_name(&abi_constructor.name)
                        .unwrap()
                        .into_(),
                ),
                abi_constructor.name,
            );
        }
        _ => {}
    }
}
