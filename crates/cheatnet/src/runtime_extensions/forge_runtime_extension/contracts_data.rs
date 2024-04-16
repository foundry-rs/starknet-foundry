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
    contracts: HashMap<String, ContractData>,
    class_hash_index: BiMap<String, ClassHash>,
    selectors: HashMap<EntryPointSelector, String>,
}

#[derive(Debug, Clone)]
struct ContractData {
    artifacts: StarknetContractArtifacts,
    class_hash: ClassHash,
    _sierra_source_path: Utf8PathBuf,
}

impl ContractsData {
    pub fn try_from(
        contracts: HashMap<String, (StarknetContractArtifacts, Utf8PathBuf)>,
    ) -> Result<Self> {
        let parsed_contracts: HashMap<String, SierraClass> = contracts
            .par_iter()
            .map(|(name, (artifact, _))| {
                Ok((name.clone(), serde_json::from_str(&artifact.sierra)?))
            })
            .collect::<Result<_>>()?;

        let class_hashes: Vec<(String, ClassHash)> = parsed_contracts
            .par_iter()
            .map(|(name, sierra_class)| Ok((name.clone(), get_class_hash(sierra_class)?)))
            .collect::<Result<_>>()?;
        let class_hash_index = BiMap::from_iter(class_hashes);

        let contracts = contracts
            .into_iter()
            .map(|(name, (artifacts, sierra_source_path))| {
                let class_hash = *class_hash_index.get_by_left(&name).unwrap();
                (
                    name,
                    ContractData {
                        artifacts,
                        class_hash,
                        _sierra_source_path: sierra_source_path,
                    },
                )
            })
            .collect();

        let selectors = parsed_contracts
            .into_par_iter()
            .map(|(_, sierra_class)| build_name_selector_map(sierra_class.abi))
            .flatten()
            .collect();

        Ok(ContractsData {
            contracts,
            class_hash_index,
            selectors,
        })
    }

    #[must_use]
    pub fn get_artifacts_for_contract(&self, name: &str) -> Option<&StarknetContractArtifacts> {
        self.contracts.get(name).map(|contract| &contract.artifacts)
    }

    #[must_use]
    pub fn get_class_hash_for_contract(&self, name: &str) -> Option<&ClassHash> {
        self.contracts
            .get(name)
            .map(|contract| &contract.class_hash)
    }

    #[must_use]
    pub fn get_contract_name_from_class_hash(&self, class_hash: &ClassHash) -> Option<&String> {
        self.class_hash_index.get_by_right(class_hash)
    }

    #[must_use]
    pub fn get_function_name_from_entry_point_selector(
        &self,
        entry_point_selector: &EntryPointSelector,
    ) -> Option<&String> {
        self.selectors.get(entry_point_selector)
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
