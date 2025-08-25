use crate::trace::types::{ContractName, Selector};
use cairo_lang_sierra::program::ProgramArtifact;
use cairo_lang_sierra_to_casm::compiler::CairoProgramDebugInfo;
use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
use cairo_lang_starknet_classes::contract_class::ContractClass;
use cheatnet::forking::data::ForkData;
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use starknet::core::types::contract::{AbiEntry, SierraClass};
use starknet_api::core::{ClassHash, EntryPointSelector};
use std::collections::HashMap;
use std::hash::Hash;

/// Data structure containing information about contracts,
/// including their ABI, names, selectors and programs that will be used to create a [`Trace`](crate::Trace).
pub struct ContractsDataStore {
    abi: HashMap<ClassHash, Vec<AbiEntry>>,
    contract_names: HashMap<ClassHash, ContractName>,
    selectors: HashMap<EntryPointSelector, Selector>,
    programs: HashMap<ClassHash, ProgramArtifact>,
    // FIXME(https://github.com/software-mansion/universal-sierra-compiler/issues/98): Use CASM debug info from USC once it provides it.
    casm_debug_infos: HashMap<ClassHash, CairoProgramDebugInfo>,
}

impl ContractsDataStore {
    /// Creates a new instance of [`ContractsDataStore`] from a similar structure from `cheatnet`: [`ContractsData`] and [`ForkData`].
    #[must_use]
    pub fn new(contracts_data: &ContractsData, fork_data: &ForkData) -> Self {
        let contract_names = contracts_data
            .class_hashes
            .iter()
            .map(|(name, class_hash)| (*class_hash, ContractName(name.clone())))
            .collect();

        let selectors = contracts_data
            .selectors
            .iter()
            .chain(&fork_data.selectors)
            .map(|(selector, function_name)| (*selector, Selector(function_name.clone())))
            .collect();

        let abi = contracts_data
            .contracts
            .par_iter()
            .map(|(_, contract_data)| {
                let sierra = serde_json::from_str::<SierraClass>(&contract_data.artifacts.sierra)
                    .expect("this should be valid `SierraClass`");
                (contract_data.class_hash, sierra.abi)
            })
            .chain(fork_data.abi.clone())
            .collect();

        let program_data = contracts_data
            .contracts
            .par_iter()
            .map(|(_, contract_data)| {
                let contract_class =
                    serde_json::from_str::<ContractClass>(&contract_data.artifacts.sierra)
                        .expect("this should be valid `ContractClass`");

                let program = contract_class
                    .extract_sierra_program()
                    .expect("extraction should succeed");

                let debug_info = contract_class.sierra_program_debug_info;

                let program_artifact = ProgramArtifact {
                    program,
                    debug_info,
                };

                let casm_debug_info = compile(ContractClass {
                    // Debug info is unused in the compilation. This saves us a costly clone.
                    sierra_program_debug_info: None,
                    ..contract_class
                });

                (
                    contract_data.class_hash,
                    (program_artifact, casm_debug_info),
                )
            })
            .collect::<Vec<_>>();

        let (programs, casm_debug_infos) = split_maps(program_data);

        Self {
            abi,
            contract_names,
            selectors,
            programs,
            casm_debug_infos,
        }
    }

    /// Gets the [`ContractName`] for a given contract [`ClassHash`].
    #[must_use]
    pub fn get_contract_name(&self, class_hash: &ClassHash) -> Option<&ContractName> {
        self.contract_names.get(class_hash)
    }

    /// Gets the `abi` for a given contract [`ClassHash`] from [`ContractsDataStore`].
    pub fn get_abi(&self, class_hash: &ClassHash) -> Option<&[AbiEntry]> {
        self.abi.get(class_hash).map(Vec::as_slice)
    }

    /// Gets the [`Selector`] in human-readable form for a given [`EntryPointSelector`] from [`ContractsDataStore`].
    #[must_use]
    pub fn get_selector(&self, entry_point_selector: &EntryPointSelector) -> Option<&Selector> {
        self.selectors.get(entry_point_selector)
    }

    /// Gets the [`ProgramArtifact`] for a given contract [`ClassHash`].
    #[must_use]
    pub fn get_program_artifact(&self, class_hash: &ClassHash) -> Option<&ProgramArtifact> {
        self.programs.get(class_hash)
    }

    /// Gets the [`CairoProgramDebugInfo`] for a given contract [`ClassHash`].
    #[must_use]
    pub fn get_casm_debug_info(&self, class_hash: &ClassHash) -> Option<&CairoProgramDebugInfo> {
        self.casm_debug_infos.get(class_hash)
    }

    /// Checks if the contract with the given [`ClassHash`] is a forked contract.
    pub fn is_fork(&self, class_hash: &ClassHash) -> bool {
        // We create contract names only from `ContractsData` and not from `ForkData`,
        // so if the contract name is not present in `contract_names`, it is a fork
        !self.contract_names.contains_key(class_hash)
    }
}

/// Compile the given [`ContractClass`] to `casm` and return [`CairoProgramDebugInfo`]
fn compile(contract_class: ContractClass) -> CairoProgramDebugInfo {
    let (_, casm_debug_info) =
        CasmContractClass::from_contract_class_with_debug_info(contract_class, false, usize::MAX)
            .expect("compilation should succeed");
    casm_debug_info
}

/// Splits an iterator of `(K, (V1, V2))` into two `HashMaps`:
/// `HashMap<K, V1>` and `HashMap<K, V2>`
fn split_maps<K, V1, V2, I>(iter: I) -> (HashMap<K, V1>, HashMap<K, V2>)
where
    K: Copy + Eq + Hash,
    I: IntoIterator<Item = (K, (V1, V2))>,
{
    let mut map1 = HashMap::new();
    let mut map2 = HashMap::new();

    for (key, (v1, v2)) in iter {
        map1.insert(key, v1);
        map2.insert(key, v2);
    }

    (map1, map2)
}
