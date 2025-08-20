use crate::debugging::TraceArgs;
use blockifier::execution::contract_class::TrackedResource;
use camino::Utf8PathBuf;
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use clap::ValueEnum;
use serde::Deserialize;
use std::collections::HashMap;
use std::ffi::OsString;
use std::num::NonZeroU32;
use std::sync::Arc;

#[derive(Debug, PartialEq)]
pub struct ForgeConfig {
    pub test_runner_config: Arc<TestRunnerConfig>,
    pub output_config: Arc<OutputConfig>,
}

#[derive(Debug, PartialEq)]
pub struct TestRunnerConfig {
    pub exit_first: bool,
    pub fuzzer_runs: NonZeroU32,
    pub fuzzer_seed: u64,
    pub max_n_steps: Option<u32>,
    pub is_vm_trace_needed: bool,
    pub cache_dir: Utf8PathBuf,
    pub contracts_data: ContractsData,
    pub environment_variables: HashMap<String, String>,
    pub tracked_resource: ForgeTrackedResource,
    pub experimental_oracles: bool,
    pub use_native: bool,
}

#[derive(Debug, PartialEq)]
pub struct OutputConfig {
    pub trace_args: TraceArgs,
    pub detailed_resources: bool,
    pub execution_data_to_save: ExecutionDataToSave,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct ExecutionDataToSave {
    pub trace: bool,
    pub profile: bool,
    pub coverage: bool,
    pub additional_args: Vec<OsString>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Deserialize, Eq, ValueEnum)]
pub enum ForgeTrackedResource {
    #[default]
    CairoSteps,
    SierraGas,
}

impl From<&ForgeTrackedResource> for TrackedResource {
    fn from(m: &ForgeTrackedResource) -> Self {
        match m {
            ForgeTrackedResource::CairoSteps => TrackedResource::CairoSteps,
            ForgeTrackedResource::SierraGas => TrackedResource::SierraGas,
        }
    }
}

impl ExecutionDataToSave {
    #[must_use]
    pub fn from_flags(
        save_trace_data: bool,
        build_profile: bool,
        coverage: bool,
        additional_args: &[OsString],
    ) -> Self {
        Self {
            trace: save_trace_data,
            profile: build_profile,
            coverage,
            additional_args: additional_args.to_vec(),
        }
    }
    #[must_use]
    pub fn is_vm_trace_needed(&self) -> bool {
        self.trace || self.profile || self.coverage
    }
}

/// This struct should be constructed on demand to pass only relevant information from
/// [`TestRunnerConfig`] to another function.
pub struct RuntimeConfig<'a> {
    pub max_n_steps: Option<u32>,
    pub is_vm_trace_needed: bool,
    pub cache_dir: &'a Utf8PathBuf,
    pub contracts_data: &'a ContractsData,
    pub environment_variables: &'a HashMap<String, String>,
    pub tracked_resource: &'a ForgeTrackedResource,
    pub experimental_oracles: bool,
}

impl<'a> RuntimeConfig<'a> {
    #[must_use]
    pub fn from(value: &'a TestRunnerConfig) -> RuntimeConfig<'a> {
        Self {
            max_n_steps: value.max_n_steps,
            is_vm_trace_needed: value.is_vm_trace_needed,
            cache_dir: &value.cache_dir,
            contracts_data: &value.contracts_data,
            environment_variables: &value.environment_variables,
            tracked_resource: &value.tracked_resource,
            experimental_oracles: value.experimental_oracles,
        }
    }
}
