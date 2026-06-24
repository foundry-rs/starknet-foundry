use super::cheatcodes::declare::get_class_hash;
use anyhow::Result;
use bimap::BiMap;
use camino::Utf8PathBuf;
use conversions::IntoConv;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use runtime::starknet::constants::TEST_CONTRACT_CLASS_HASH;
use scarb_api::{ContractsData as ScarbContractsData, StarknetContractArtifacts};
use shared::utils::contract_name_from_module_path;
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
    pub artifacts: StarknetContractArtifacts,
    pub class_hash: ClassHash,
    pub source_sierra_path: Utf8PathBuf,
}

#[derive(Debug)]
pub enum ContractResolutionError {
    /// No contract with the given name was found.
    NameNotFound,
    /// Several contracts share the same name. Carries their module tree paths.
    AmbiguousName(Vec<ModulePath>),
}

impl ContractsData {
    pub fn try_from(contracts: ScarbContractsData) -> Result<Self> {
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

        let contracts: HashMap<ModulePath, ContractData> = contracts
            .into_iter()
            .map(|(module_path, contract)| {
                let class_hash = *class_hashes.get_by_left(&module_path).unwrap();
                (
                    module_path,
                    ContractData {
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

    /// Resolves a user-provided identifier to a single contract.
    ///
    /// The identifier can be either a contract name (e.g. `MyContract`), an absolute
    /// module tree path (e.g. `my_package::module::MyContract`) or a partial module tree path
    /// (e.g. `module::MyContract`).
    /// Returns an error if no contract or multiple contracts match the given identifier.
    pub fn resolve_contract(
        &self,
        contract_identifier: &str,
    ) -> Result<&ContractData, ContractResolutionError> {
        let contract_identifier = contract_identifier
            .strip_prefix("::")
            .unwrap_or(contract_identifier);
        let module_path_suffix = format!("::{contract_identifier}");

        let matches: Vec<(&ModulePath, &ContractData)> = self
            .contracts
            .iter()
            .filter(|(module_path, _)| {
                *module_path == contract_identifier || module_path.ends_with(&module_path_suffix)
            })
            .collect();

        match matches.as_slice() {
            [] => Err(ContractResolutionError::NameNotFound),
            [(_, contract)] => Ok(contract),
            _ => {
                let mut module_paths: Vec<ModulePath> =
                    matches.iter().map(|(path, _)| (*path).clone()).collect();
                module_paths.sort();
                Err(ContractResolutionError::AmbiguousName(module_paths))
            }
        }
    }

    #[must_use]
    pub fn get_contract_by_class_hash(&self, class_hash: &ClassHash) -> Option<&ContractData> {
        let module_path = self.class_hashes.get_by_right(class_hash)?;
        self.contracts.get(module_path)
    }

    #[must_use]
    pub fn get_contract_name(&self, class_hash: &ClassHash) -> Option<ContractName> {
        let module_path = self.class_hashes.get_by_right(class_hash)?;
        Some(contract_name_from_module_path(module_path).to_string())
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
