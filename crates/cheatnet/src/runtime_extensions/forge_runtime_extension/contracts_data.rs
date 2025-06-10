use super::cheatcodes::declare::get_class_hash;
use anyhow::Result;
use bimap::BiMap;
use camino::Utf8PathBuf;
use conversions::IntoConv;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use runtime::starknet::constants::TEST_CONTRACT_CLASS_HASH;
use scarb_api::StarknetContractArtifacts;
use starknet::core::types::contract::{AbiEntry, SierraClass};
use starknet::core::utils::get_selector_from_name;
use starknet_api::core::{ClassHash, EntryPointSelector};
use starknet_types_core::felt::Felt;
use std::collections::HashMap;

type ContractName = String;
type FunctionName = String;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ContractsData {
    pub contracts: HashMap<ContractName, ContractData>,
    pub class_hashes: BiMap<ContractName, ClassHash>,
    pub selectors: HashMap<EntryPointSelector, FunctionName>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContractData {
    pub artifacts: StarknetContractArtifacts,
    pub class_hash: ClassHash,
    source_sierra_path: Utf8PathBuf,
}

impl ContractsData {
    pub fn try_from(
        contracts: HashMap<ContractName, (StarknetContractArtifacts, Utf8PathBuf)>,
    ) -> Result<Self> {
        let parsed_contracts: HashMap<ContractName, SierraClass> = contracts
            .par_iter()
            .map(|(name, (artifact, _))| {
                Ok((name.clone(), serde_json::from_str(&artifact.sierra)?))
            })
            .collect::<Result<_>>()?;

        let class_hashes: Vec<(ContractName, ClassHash)> = parsed_contracts
            .par_iter()
            .map(|(name, sierra_class)| Ok((name.clone(), get_class_hash(sierra_class)?)))
            .collect::<Result<_>>()?;
        let class_hashes = BiMap::from_iter(class_hashes);

        let contracts = contracts
            .into_iter()
            .map(|(name, (artifacts, source_sierra_path))| {
                let class_hash = *class_hashes.get_by_left(&name).unwrap();
                (
                    name,
                    ContractData {
                        artifacts,
                        class_hash,
                        source_sierra_path,
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
            class_hashes,
            selectors,
        })
    }

    #[must_use]
    pub fn get_artifacts(&self, contract_name: &str) -> Option<&StarknetContractArtifacts> {
        self.contracts
            .get(contract_name)
            .map(|contract| &contract.artifacts)
    }

    #[must_use]
    pub fn get_class_hash(&self, contract_name: &str) -> Option<&ClassHash> {
        self.contracts
            .get(contract_name)
            .map(|contract| &contract.class_hash)
    }

    #[must_use]
    pub fn get_source_sierra_path(&self, contract_name: &str) -> Option<&Utf8PathBuf> {
        self.contracts
            .get(contract_name)
            .map(|contract| &contract.source_sierra_path)
    }

    #[must_use]
    pub fn get_contract_name(&self, class_hash: &ClassHash) -> Option<&ContractName> {
        self.class_hashes.get_by_right(class_hash)
    }

    #[must_use]
    pub fn get_function_name(
        &self,
        entry_point_selector: &EntryPointSelector,
    ) -> Option<&FunctionName> {
        self.selectors.get(entry_point_selector)
    }

    #[must_use]
    pub fn is_fork_class_hash(&self, class_hash: &ClassHash) -> bool {
        if class_hash.0 == Felt::from_hex_unchecked(TEST_CONTRACT_CLASS_HASH) {
            false
        } else {
            !self.class_hashes.contains_right(class_hash)
        }
    }
}

#[must_use]
pub fn build_name_selector_map(abi: Vec<AbiEntry>) -> HashMap<EntryPointSelector, FunctionName> {
    let mut selector_map = HashMap::new();
    for abi_entry in abi {
        match abi_entry {
            AbiEntry::Interface(abi_interface) => {
                for abi_entry in abi_interface.items {
                    add_simple_abi_entry_to_mapping(abi_entry, &mut selector_map);
                }
            }
            _ => add_simple_abi_entry_to_mapping(abi_entry, &mut selector_map),
        }
    }
    selector_map
}

fn add_simple_abi_entry_to_mapping(
    abi_entry: AbiEntry,
    selector_map: &mut HashMap<EntryPointSelector, FunctionName>,
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
