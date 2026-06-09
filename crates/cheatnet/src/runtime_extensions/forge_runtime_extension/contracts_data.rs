use super::cheatcodes::declare::get_class_hash;
use anyhow::Result;
use bimap::BiMap;
use camino::Utf8PathBuf;
use conversions::IntoConv;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use runtime::starknet::constants::TEST_CONTRACT_CLASS_HASH;
use scarb_api::{LoadedContracts, StarknetContractArtifacts};
use starknet_api::core::{ClassHash, EntryPointSelector};
use starknet_rust::core::types::contract::{AbiEntry, SierraClass};
use starknet_rust::core::utils::get_selector_from_name;
use starknet_types_core::felt::Felt;
use std::collections::HashMap;
use std::hash::BuildHasher;

type ContractName = String;
type ModulePath = String;
type FunctionName = String;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ContractsData {
    pub contracts: HashMap<ModulePath, ContractData>,
    pub class_hashes: BiMap<ModulePath, ClassHash>,
    pub selectors: HashMap<EntryPointSelector, FunctionName>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContractData {
    pub name: ContractName,
    pub artifacts: StarknetContractArtifacts,
    pub class_hash: ClassHash,
    pub source_sierra_path: Utf8PathBuf,
}

/// Outcome of resolving a contract name to a single contract.
#[derive(Debug)]
pub enum ContractResolutionError {
    /// No contract with the given name was found.
    NotFound,
    /// Several contracts share the same name. Carries their module paths (sorted) for diagnostics.
    Ambiguous(Vec<ModulePath>),
}

impl ContractsData {
    pub fn try_from(contracts: LoadedContracts) -> Result<Self> {
        let parsed_contracts: HashMap<ModulePath, SierraClass> = contracts
            .par_iter()
            .map(|(module_path, contract)| {
                Ok((
                    module_path.clone(),
                    serde_json::from_str(&contract.artifacts.sierra)?,
                ))
            })
            .collect::<Result<_>>()?;

        let class_hashes: Vec<(ModulePath, ClassHash)> = parsed_contracts
            .par_iter()
            .map(|(module_path, sierra_class)| {
                Ok((module_path.clone(), get_class_hash(sierra_class)?))
            })
            .collect::<Result<_>>()?;
        let class_hashes = BiMap::from_iter(class_hashes);

        let contracts = contracts
            .into_iter()
            .map(|(module_path, contract)| {
                let class_hash = *class_hashes.get_by_left(&module_path).unwrap();
                (
                    module_path,
                    ContractData {
                        name: contract.contract_name,
                        artifacts: contract.artifacts,
                        class_hash,
                        source_sierra_path: contract.sierra_path,
                    },
                )
            })
            .collect();

        let selectors = parsed_contracts
            .into_par_iter()
            .map(|(_, sierra_class)| build_name_selector_map(&sierra_class.abi))
            .flatten()
            .collect();

        Ok(ContractsData {
            contracts,
            class_hashes,
            selectors,
        })
    }

    /// Resolves a user-provided `contract_name` to a single contract.
    /// Returns an error if no contract or multiple contracts are found with the given name.
    pub fn resolve_by_name(
        &self,
        contract_name: &str,
    ) -> Result<&ContractData, ContractResolutionError> {
        let mut matches: Vec<(&ModulePath, &ContractData)> = self
            .contracts
            .iter()
            .filter(|(_, contract)| contract.name == contract_name)
            .collect();

        match matches.len() {
            0 => Err(ContractResolutionError::NotFound),
            1 => Ok(matches.pop().unwrap().1),
            _ => {
                let mut module_paths: Vec<ModulePath> = matches
                    .into_iter()
                    .map(|(module_path, _)| module_path.clone())
                    .collect();
                module_paths.sort();
                Err(ContractResolutionError::Ambiguous(module_paths))
            }
        }
    }

    #[must_use]
    pub fn get_contract_by_class_hash(&self, class_hash: &ClassHash) -> Option<&ContractData> {
        let module_path = self.class_hashes.get_by_right(class_hash)?;
        self.contracts.get(module_path)
    }

    #[must_use]
    pub fn get_contract_name(&self, class_hash: &ClassHash) -> Option<&ContractName> {
        self.get_contract_by_class_hash(class_hash)
            .map(|contract| &contract.name)
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
pub fn build_selectors_from_abi_map<S: BuildHasher>(
    abi: &HashMap<ClassHash, Vec<AbiEntry>, S>,
) -> HashMap<EntryPointSelector, FunctionName> {
    abi.values()
        .flat_map(|a| build_name_selector_map(a))
        .collect()
}

#[must_use]
pub fn build_name_selector_map(abi: &[AbiEntry]) -> HashMap<EntryPointSelector, FunctionName> {
    let mut selector_map = HashMap::new();
    for abi_entry in abi {
        match abi_entry {
            AbiEntry::Interface(abi_interface) => {
                for abi_entry in &abi_interface.items {
                    add_simple_abi_entry_to_mapping(abi_entry, &mut selector_map);
                }
            }
            _ => add_simple_abi_entry_to_mapping(abi_entry, &mut selector_map),
        }
    }
    selector_map
}

fn add_simple_abi_entry_to_mapping(
    abi_entry: &AbiEntry,
    selector_map: &mut HashMap<EntryPointSelector, FunctionName>,
) {
    match abi_entry {
        AbiEntry::Function(abi_function) | AbiEntry::L1Handler(abi_function) => {
            selector_map.insert(
                get_selector_from_name(&abi_function.name).unwrap().into_(),
                abi_function.name.clone(),
            );
        }
        AbiEntry::Constructor(abi_constructor) => {
            selector_map.insert(
                get_selector_from_name(&abi_constructor.name)
                    .unwrap()
                    .into_(),
                abi_constructor.name.clone(),
            );
        }
        _ => {}
    }
}
