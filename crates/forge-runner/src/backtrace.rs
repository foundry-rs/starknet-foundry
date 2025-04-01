use anyhow::{Context, Result};
use cairo_annotations::annotations::TryFromDebugInfo;
use cairo_annotations::annotations::coverage::{
    CodeLocation, ColumnNumber, CoverageAnnotationsV1, LineNumber, VersionedCoverageAnnotations,
};
use cairo_annotations::annotations::profiler::{
    FunctionName, ProfilerAnnotationsV1, VersionedProfilerAnnotations,
};
use cairo_lang_sierra::program::StatementIdx;
use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
use cairo_lang_starknet_classes::contract_class::ContractClass;
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use indoc::indoc;
use itertools::Itertools;
use rayon::prelude::*;
use starknet_api::core::ClassHash;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Display;
use std::{env, fmt};

const BACKTRACE_ENV: &str = "SNFORGE_BACKTRACE";

pub fn add_backtrace_footer(
    message: String,
    contracts_data: &ContractsData,
    encountered_errors: &BTreeMap<ClassHash, Vec<usize>>,
) -> String {
    if encountered_errors.is_empty() {
        return message;
    }

    let is_backtrace_enabled = env::var(BACKTRACE_ENV).is_ok_and(|value| value == "1");
    if !is_backtrace_enabled {
        return format!(
            "{message}\nnote: run with `{BACKTRACE_ENV}=1` environment variable to display a backtrace",
        );
    }

    let class_hashes = encountered_errors.keys().copied().collect();

    BacktraceContractRepository::new(contracts_data, class_hashes)
        .and_then(|repository| {
            encountered_errors
                .iter()
                .map(|(class_hash, pcs)| repository.get_backtrace(pcs, class_hash))
                .map(|backtrace| backtrace.map(|backtrace| backtrace.to_string()))
                .collect::<Result<Vec<_>>>()
                .map(|backtrace| backtrace.join("\n"))
        })
        .map_or_else(
            |err| format!("{message}\nfailed to create backtrace: {err}"),
            |backtraces| format!("{message}\n{backtraces}"),
        )
}

struct ContractBacktraceData {
    contract_name: String,
    casm_debug_info_start_offsets: Vec<usize>,
    coverage_annotations: CoverageAnnotationsV1,
    profiler_annotations: ProfilerAnnotationsV1,
}

impl ContractBacktraceData {
    fn new(class_hash: &ClassHash, contracts_data: &ContractsData) -> Result<Self> {
        let contract_name = contracts_data
            .get_contract_name(class_hash)
            .context(format!(
                "failed to get contract name for class hash: {class_hash}"
            ))?
            .clone();

        let contract_artifacts = contracts_data
            .get_artifacts(&contract_name)
            .context(format!(
                "failed to get artifacts for contract name: {contract_name}"
            ))?;

        let contract_class = serde_json::from_str::<ContractClass>(&contract_artifacts.sierra)?;

        let sierra_debug_info = contract_class
            .sierra_program_debug_info
            .as_ref()
            .context("debug info not found")?;

        let VersionedCoverageAnnotations::V1(coverage_annotations) =
            VersionedCoverageAnnotations::try_from_debug_info(sierra_debug_info).context(indoc! {
                "perhaps the contract was compiled without the following entry in Scarb.toml under [profile.dev.cairo]:
                unstable-add-statements-code-locations-debug-info = true

                or scarb version is less than 2.8.0
                "
            })?;

        let VersionedProfilerAnnotations::V1(profiler_annotations) =
            VersionedProfilerAnnotations::try_from_debug_info(sierra_debug_info).context(indoc! {
                "perhaps the contract was compiled without the following entry in Scarb.toml under [profile.dev.cairo]:
                unstable-add-statements-functions-debug-info = true

                or scarb version is less than 2.8.0
                "
            })?;

        // Not optimal, but USC doesn't produce debug info for the contract class
        let (_, debug_info) = CasmContractClass::from_contract_class_with_debug_info(
            contract_class,
            true,
            usize::MAX,
        )?;

        let casm_debug_info_start_offsets = debug_info
            .sierra_statement_info
            .iter()
            .map(|statement_debug_info| statement_debug_info.start_offset)
            .collect();

        Ok(Self {
            contract_name,
            casm_debug_info_start_offsets,
            coverage_annotations,
            profiler_annotations,
        })
    }

    fn backtrace_from(&self, pc: usize) -> Result<Vec<Backtrace>> {
        let sierra_statement_idx = StatementIdx(
            self.casm_debug_info_start_offsets
                .partition_point(|start_offset| *start_offset < pc - 1)
                .saturating_sub(1),
        );

        let code_locations = self
            .coverage_annotations
            .statements_code_locations
            .get(&sierra_statement_idx)
            .with_context(|| {
                format!("failed to get code locations for statement idx: {sierra_statement_idx}")
            })?;

        let function_names = self
            .profiler_annotations
            .statements_functions
            .get(&sierra_statement_idx)
            .with_context(|| {
                format!("failed to get function names for statement idx: {sierra_statement_idx}")
            })?;

        let stack = code_locations
            .iter()
            .zip(function_names)
            .enumerate()
            .map(|(index, (code_location, function_name))| {
                let is_not_last = index != function_names.len() - 1;
                Backtrace {
                    inlined: is_not_last,
                    code_location,
                    function_name,
                }
            })
            .collect();

        Ok(stack)
    }

    fn backtrace_stack_from(&self, pcs: &[usize]) -> Result<BacktraceStack> {
        let stack = pcs
            .iter()
            .map(|pc| self.backtrace_from(*pc))
            .flatten_ok()
            .collect::<Result<Vec<_>>>()?;

        let contract_name = &self.contract_name;

        Ok(BacktraceStack {
            contract_name,
            stack,
        })
    }
}

struct BacktraceContractRepository(HashMap<ClassHash, ContractBacktraceData>);

impl BacktraceContractRepository {
    fn new(contracts_data: &ContractsData, class_hashes: HashSet<ClassHash>) -> Result<Self> {
        Ok(Self(
            class_hashes
                .into_par_iter()
                .map(|class_hash| {
                    ContractBacktraceData::new(&class_hash, contracts_data)
                        .map(|contract_data| (class_hash, contract_data))
                })
                .collect::<Result<_>>()?,
        ))
    }

    fn get_backtrace(&self, pc: &[usize], class_hash: &ClassHash) -> Result<BacktraceStack> {
        self.0
            .get(class_hash)
            .expect("class hash should be present in the repository")
            .backtrace_stack_from(pc)
    }
}

struct Backtrace<'a> {
    code_location: &'a CodeLocation,
    function_name: &'a FunctionName,
    inlined: bool,
}

struct BacktraceStack<'a> {
    contract_name: &'a str,
    stack: Vec<Backtrace<'a>>,
}

impl Display for Backtrace<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let function_name = &self.function_name.0;
        let path = &self.code_location.0;
        let line = self.code_location.1.start.line + LineNumber(1); // most editors start line numbers from 1
        let col = self.code_location.1.start.col + ColumnNumber(1); // most editors start column numbers from 1

        if self.inlined {
            write!(f, "(inlined) ")?;
        }

        write!(f, "{function_name}\n       at {path}:{line}:{col}")
    }
}

impl Display for BacktraceStack<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "error occurred in contract '{}'", self.contract_name,)?;
        writeln!(f, "stack backtrace:")?;
        for (i, backtrace) in self.stack.iter().enumerate() {
            writeln!(f, "   {i}: {backtrace}")?;
        }
        Ok(())
    }
}
