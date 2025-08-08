use crate::trace::types::{ContractName, Selector};
use cairo_lang_sierra::program::{Program, ProgramArtifact};
use cairo_lang_sierra_to_casm::compiler::{
    CairoProgram, CairoProgramDebugInfo, SierraToCasmConfig,
};
use cairo_lang_sierra_to_casm::metadata::{MetadataComputationConfig, calc_metadata};
use cairo_lang_starknet_classes::contract_class::ContractClass;
use cheatnet::forking::data::ForkData;
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use starknet::core::types::contract::{AbiEntry, SierraClass};
use starknet_api::core::{ClassHash, EntryPointSelector};
use std::collections::HashMap;

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

        let programs: HashMap<_, _> = contracts_data
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

                (contract_data.class_hash, program_artifact)
            })
            .collect();

        let casm_debug_infos = programs
            .iter()
            .map(|(class_hash, program_artifact)| {
                let casm = compile(&program_artifact.program);
                (*class_hash, casm.debug_info)
            })
            .collect();

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
}

/// Compile the given [`Program`] to `casm`.
fn compile(program: &Program) -> CairoProgram {
    let metadata = calc_metadata(program, MetadataComputationConfig::default())
        .expect("metadata calculation should not fail");

    let config = SierraToCasmConfig {
        gas_usage_check: false,
        max_bytecode_size: usize::MAX,
    };

    cairo_lang_sierra_to_casm::compiler::compile(program, &metadata, config)
        .expect("compilation should not fail")
}
