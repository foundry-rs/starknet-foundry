use super::cheatcodes::declare::get_class_hash;
use anyhow::Result;
use bimap::BiMap;
use camino::Utf8PathBuf;
use conversions::IntoConv;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use scarb_api::StarknetContractArtifacts;
use starknet::core::types::contract::{AbiEntry, SierraClass};
use starknet::core::utils::get_selector_from_name;
use starknet_api::core::{ClassHash, EntryPointSelector};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ContractsData {
    pub contracts: HashMap<String, StarknetContractArtifacts>,
    pub source_sierra_paths: HashMap<String, Utf8PathBuf>,
    pub class_hashes: BiMap<String, ClassHash>,
    pub selectors: HashMap<EntryPointSelector, String>,
}

impl ContractsData {
    pub fn try_from(
        contracts_artifacts: HashMap<String, StarknetContractArtifacts>,
        contracts_sierra_paths: HashMap<String, Utf8PathBuf>,
    ) -> Result<Self> {
        let parsed_contracts: HashMap<String, SierraClass> = contracts_artifacts
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
            contracts: contracts_artifacts,
            source_sierra_paths: contracts_sierra_paths,
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
                get_selector_from_name(&abi_function.name).unwrap().into_(),
                abi_function.name,
            );
        }
        AbiEntry::Constructor(abi_constructor) => {
            selector_map.insert(
                get_selector_from_name(&abi_constructor.name)
                    .unwrap()
                    .into_(),
                abi_constructor.name,
            );
        }
        _ => {}
    }
}
