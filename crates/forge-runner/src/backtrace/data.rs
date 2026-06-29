use crate::backtrace::display::{Backtrace, BacktraceKind, BacktraceStack, render_fork_backtrace};
use anyhow::Context;
use anyhow::Result;
use cairo_annotations::annotations::TryFromDebugInfo;
use cairo_annotations::annotations::coverage::{
    CoverageAnnotationsV1, VersionedCoverageAnnotations,
};
use cairo_annotations::annotations::profiler::{
    ProfilerAnnotationsV1, VersionedProfilerAnnotations,
};
use cairo_lang_sierra::debug_info::DebugInfo;
use cairo_lang_sierra::program::StatementIdx;
use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
use cairo_lang_starknet_classes::contract_class::ContractClass;
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use itertools::Itertools;
use shared::utils::contract_name_from_module_path;
use starknet_api::core::ClassHash;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct LazyContractBacktraceDataMapping(Mutex<HashMap<ClassHash, Arc<ContractOrigin>>>);

impl LazyContractBacktraceDataMapping {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn get_or_build(
        &self,
        class_hash: &ClassHash,
        contracts_data: &ContractsData,
    ) -> Result<Arc<ContractOrigin>> {
        if let Some(existing) = self.lock().get(class_hash) {
            return Ok(existing.clone());
        }

        // Build outside the lock so a slow CASM build doesn't block other threads;
        // On a race both build, and the first insert wins.
        let contract_origin = Arc::new(ContractOrigin::new(class_hash, contracts_data)?);
        Ok(self
            .lock()
            .entry(*class_hash)
            .or_insert(contract_origin)
            .clone())
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<ClassHash, Arc<ContractOrigin>>> {
        self.0
            .lock()
            .expect("contract backtrace mapping mutex poisoned")
    }

    pub fn render_backtrace(
        &self,
        class_hash: &ClassHash,
        pcs: &[usize],
        contracts_data: &ContractsData,
    ) -> Result<String> {
        self.get_or_build(class_hash, contracts_data)?
            .render_backtrace(pcs)
    }
}

enum ContractOrigin {
    Fork(ClassHash),
    Local(ContractBacktraceData),
}

impl ContractOrigin {
    fn new(class_hash: &ClassHash, contracts_data: &ContractsData) -> Result<Self> {
        if contracts_data.is_fork_class_hash(class_hash) {
            Ok(ContractOrigin::Fork(*class_hash))
        } else {
            Ok(ContractOrigin::Local(ContractBacktraceData::new(
                class_hash,
                contracts_data,
            )?))
        }
    }
    fn render_backtrace(&self, pcs: &[usize]) -> Result<String> {
        match self {
            ContractOrigin::Fork(class_hash) => Ok(render_fork_backtrace(class_hash)),
            ContractOrigin::Local(data) => data.render_backtrace(pcs),
        }
    }
}

pub struct BacktraceAnnotations {
    coverage: CoverageAnnotationsV1,
    profiler: ProfilerAnnotationsV1,
}

impl BacktraceAnnotations {
    pub fn from_debug_info(sierra_debug_info: &DebugInfo) -> Result<Self> {
        let VersionedCoverageAnnotations::V1(coverage) =
            VersionedCoverageAnnotations::try_from_debug_info(sierra_debug_info)?;
        let VersionedProfilerAnnotations::V1(profiler) =
            VersionedProfilerAnnotations::try_from_debug_info(sierra_debug_info)?;

        Ok(Self { coverage, profiler })
    }
}

struct BacktraceSourceData {
    kind: BacktraceKind,
    name: String,
    casm_debug_info_start_offsets: Vec<usize>,
    annotations: Arc<BacktraceAnnotations>,
}

impl BacktraceSourceData {
    fn from_debug_info(
        kind: BacktraceKind,
        name: String,
        casm_debug_info_start_offsets: Vec<usize>,
        sierra_debug_info: &DebugInfo,
    ) -> Result<Self> {
        Ok(Self {
            kind,
            name,
            casm_debug_info_start_offsets,
            annotations: Arc::new(BacktraceAnnotations::from_debug_info(sierra_debug_info)?),
        })
    }

    fn backtrace_from(&self, pc: usize) -> Result<Vec<Backtrace<'_>>> {
        let sierra_statement_idx = StatementIdx(
            self.casm_debug_info_start_offsets
                .partition_point(|start_offset| *start_offset < pc - 1)
                .saturating_sub(1),
        );

        let code_locations = self
            .annotations
            .coverage
            .statements_code_locations
            .get(&sierra_statement_idx)
            .with_context(|| {
                format!("failed to get code locations for statement idx: {sierra_statement_idx}")
            })?;

        let function_names = self
            .annotations
            .profiler
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
                // `function_names is the stack of:
                // "functions that were inlined or generated along the way up
                // to the first non-inlined function from the original code.
                // The vector represents the stack from the least meaningful elements."
                // ~ from doc of `ProfilerAnnotationsV1`
                // So we need to check if the function name is not the last one then it is inlined
                Backtrace {
                    inlined: is_not_last,
                    code_location,
                    function_name,
                }
            })
            .collect();

        Ok(stack)
    }

    fn render_backtrace(&self, pcs: &[usize]) -> Result<String> {
        let stack = pcs
            .iter()
            .map(|pc| self.backtrace_from(*pc))
            .flatten_ok()
            .collect::<Result<Vec<_>>>()?;

        let backtrace_stack = BacktraceStack {
            kind: self.kind,
            name: &self.name,
            stack,
        };

        Ok(backtrace_stack.to_string())
    }
}

struct ContractBacktraceData(BacktraceSourceData);

impl ContractBacktraceData {
    fn new(class_hash: &ClassHash, contracts_data: &ContractsData) -> Result<Self> {
        let module_path = contracts_data
            .class_hashes
            .get_by_right(class_hash)
            .context(format!(
                "module path not found for class hash: {class_hash}"
            ))?;
        let contract = contracts_data
            .contracts
            .get(module_path)
            .context(format!("contract not found for module path: {module_path}"))?;

        let contract_artifacts = &contract.artifacts;

        let contract_class = serde_json::from_str::<ContractClass>(&contract_artifacts.sierra)?;

        let sierra_debug_info = contract_class
            .sierra_program_debug_info
            .clone()
            .context("debug info not found")?;

        let extracted_sierra = contract_class
            .extract_sierra_program(false)
            .expect("extraction should succeed");

        // FIXME(https://github.com/software-mansion/universal-sierra-compiler/issues/98): Use CASM debug info from USC once it provides it.
        let (_, debug_info) = CasmContractClass::from_contract_class_with_debug_info(
            contract_class,
            extracted_sierra,
            true,
            usize::MAX,
        )?;

        let casm_debug_info_start_offsets = debug_info
            .sierra_statement_info
            .iter()
            .map(|statement_debug_info| statement_debug_info.start_offset)
            .collect();

        Ok(Self(BacktraceSourceData::from_debug_info(
            BacktraceKind::Contract,
            contract_name_from_module_path(module_path).to_string(),
            casm_debug_info_start_offsets,
            &sierra_debug_info,
        )?))
    }

    fn render_backtrace(&self, pcs: &[usize]) -> Result<String> {
        self.0.render_backtrace(pcs)
    }
}

pub struct TestBacktraceData(BacktraceSourceData);

impl TestBacktraceData {
    pub fn new(
        test_name: String,
        annotations: &Arc<BacktraceAnnotations>,
        casm_start_offsets: Vec<usize>,
    ) -> Self {
        Self(BacktraceSourceData {
            kind: BacktraceKind::Test,
            name: test_name,
            casm_debug_info_start_offsets: casm_start_offsets,
            annotations: Arc::clone(annotations),
        })
    }

    pub fn render_backtrace(&self, pcs: &[usize]) -> Result<String> {
        self.0.render_backtrace(pcs)
    }
}

/// Test-target backtrace annotations.
#[derive(Clone)]
pub enum TestAnnotations {
    Parsed(Arc<BacktraceAnnotations>),
    /// No backtrace can be produced: Backtrace is disabled, or the target has no sierra debug info.
    Missing,
    // Debug info present; Failed to parse.
    Failed(String),
}

impl TestAnnotations {
    #[must_use]
    pub fn from_debug_info(sierra_debug_info: Option<&DebugInfo>) -> Self {
        match sierra_debug_info {
            None => Self::Missing,
            Some(debug_info) => match BacktraceAnnotations::from_debug_info(debug_info) {
                Ok(annotations) => Self::Parsed(Arc::new(annotations)),
                Err(err) => Self::Failed(err.to_string()),
            },
        }
    }
}
