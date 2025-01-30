use anyhow::{ensure, Context, Result};
use cairo_annotations::annotations::coverage::{
    CodeLocation, ColumnNumber, CoverageAnnotationsV1, LineNumber, VersionedCoverageAnnotations,
};
use cairo_annotations::annotations::profiler::{
    FunctionName, ProfilerAnnotationsV1, VersionedProfilerAnnotations,
};
use cairo_annotations::annotations::TryFromDebugInfo;
use cairo_lang_sierra::program::StatementIdx;
use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
use cairo_lang_starknet_classes::contract_class::ContractClass;
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use cheatnet::state::EncounteredError;
use indoc::{formatdoc, indoc};
use itertools::Itertools;
use rayon::prelude::*;
use scarb_api::metadata::Metadata;
use semver::Version;
use starknet_api::core::ClassHash;
use std::collections::HashMap;
use std::fmt::Display;
use std::{env, fmt, fs};
use toml_edit::{DocumentMut, Table};

const BACKTRACE_ENV: &str = "SNFORGE_BACKTRACE";
const MINIMAL_SCARB_VERSION: Version = Version::new(2, 8, 0);

const CAIRO_BACKTRACE_REQUIRED_ENTRIES: [(&str, &str); 2] = [
    ("unstable-add-statements-functions-debug-info", "true"),
    ("unstable-add-statements-code-locations-debug-info", "true"),
];

pub fn add_backtrace_footer(
    message: String,
    contracts_data: &ContractsData,
    encountered_errors: &[EncounteredError],
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

    BacktraceContractRepository::new(contracts_data, encountered_errors)
        .map(|repository| {
            encountered_errors
                .iter()
                .filter_map(|error| repository.get_backtrace(error.pc, error.class_hash))
                .map(|backtrace| backtrace.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        })
        .map_or_else(
            |err| format!("{message}\nfailed to create backtrace: {err}"),
            |backtraces| format!("{message}\n{backtraces}"),
        )
}

pub fn can_be_generated(scarb_metadata: &Metadata) -> Result<()> {
    let manifest = fs::read_to_string(&scarb_metadata.runtime_manifest)?.parse::<DocumentMut>()?;

    ensure!(
        scarb_metadata.app_version_info.version >= MINIMAL_SCARB_VERSION,
        "Coverage generation requires scarb version >= {MINIMAL_SCARB_VERSION}",
    );

    let has_needed_entries = manifest
        .get("profile")
        .and_then(|profile| profile.get(&scarb_metadata.current_profile))
        .and_then(|profile| profile.get("cairo"))
        .and_then(|cairo| cairo.as_table())
        .is_some_and(|profile_cairo| {
            CAIRO_BACKTRACE_REQUIRED_ENTRIES
                .iter()
                .all(|(key, value)| contains_entry_with_value(profile_cairo, key, value))
        });

    ensure!(
        has_needed_entries,
        formatdoc! {
            "Scarb.toml must have the following Cairo compiler configuration to run backtrace:

            [profile.{profile}.cairo]
            unstable-add-statements-functions-debug-info = true
            unstable-add-statements-code-locations-debug-info = true
            inlining-strategy = \"avoid\"
            ... other entries ...
            ",
            profile = scarb_metadata.current_profile
        },
    );

    Ok(())
}

pub fn is_enabled() -> bool {
    env::var(BACKTRACE_ENV).is_ok_and(|value| value == "1")
}

fn contains_entry_with_value(table: &Table, key: &str, value: &str) -> bool {
    table
        .get(key)
        .is_some_and(|entry| entry.to_string().trim() == value)
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

    fn backtrace_from(&self, pc: usize) -> Option<BacktraceStack> {
        let sierra_statement_idx = StatementIdx(
            self.casm_debug_info_start_offsets
                .partition_point(|start_offset| *start_offset < pc - 1)
                .saturating_sub(1),
        );

        let code_locations = self
            .coverage_annotations
            .statements_code_locations
            .get(&sierra_statement_idx)?;

        let function_names = self
            .profiler_annotations
            .statements_functions
            .get(&sierra_statement_idx)?;

        let stack = code_locations
            .iter()
            .zip(function_names)
            .map(|(code_location, function_name)| Backtrace {
                code_location,
                function_name,
            })
            .collect();

        Some(BacktraceStack {
            pc,
            contract_name: &self.contract_name,
            stack,
        })
    }
}

struct BacktraceContractRepository(HashMap<ClassHash, ContractBacktraceData>);

impl BacktraceContractRepository {
    fn new(
        contracts_data: &ContractsData,
        encountered_errors: &[EncounteredError],
    ) -> Result<Self> {
        Ok(Self(
            encountered_errors
                .iter()
                .map(|error| error.class_hash)
                .unique()
                .collect::<Vec<_>>()
                .par_iter()
                .map(|class_hash| {
                    ContractBacktraceData::new(class_hash, contracts_data)
                        .map(|contract_data| (*class_hash, contract_data))
                })
                .collect::<Result<_>>()?,
        ))
    }

    fn get_backtrace(&self, pc: usize, class_hash: ClassHash) -> Option<BacktraceStack> {
        self.0.get(&class_hash)?.backtrace_from(pc)
    }
}

struct Backtrace<'a> {
    code_location: &'a CodeLocation,
    function_name: &'a FunctionName,
}

struct BacktraceStack<'a> {
    pc: usize,
    contract_name: &'a str,
    stack: Vec<Backtrace<'a>>,
}

impl Display for Backtrace<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let function_name = &self.function_name.0;
        let path = truncate_at_char(&self.code_location.0 .0, '[');
        let line = self.code_location.1.start.line + LineNumber(1); // most editors start line numbers from 1
        let col = self.code_location.1.start.col + ColumnNumber(1); // most editors start column numbers from 1

        write!(f, "{function_name}\n       at {path}:{line}:{col}",)
    }
}

impl Display for BacktraceStack<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "error occurred in contract '{}' at pc: '{}'",
            self.contract_name, self.pc
        )?;
        writeln!(f, "stack backtrace:")?;
        for (i, backtrace) in self.stack.iter().enumerate() {
            writeln!(f, "   {i}: {backtrace}")?;
        }
        Ok(())
    }
}

fn truncate_at_char(input: &str, delimiter: char) -> &str {
    match input.find(delimiter) {
        Some(index) => &input[..index],
        None => input,
    }
}
