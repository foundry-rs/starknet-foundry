use camino::Utf8PathBuf;
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use std::collections::HashMap;
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
}

#[derive(Debug, PartialEq)]
pub struct OutputConfig {
    pub detailed_resources: bool,
    pub execution_data_to_save: ExecutionDataToSave,
    pub versioned_programs_dir: Utf8PathBuf,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ExecutionDataToSave {
    None,
    Trace,
    /// Profile data requires saved trace data
    TraceAndProfile,
}

impl ExecutionDataToSave {
    #[must_use]
    pub fn from_flags(save_trace_data: bool, build_profile: bool) -> Self {
        if build_profile {
            return ExecutionDataToSave::TraceAndProfile;
        }
        if save_trace_data {
            return ExecutionDataToSave::Trace;
        }
        ExecutionDataToSave::None
    }
}

#[must_use]
pub fn is_vm_trace_needed(execution_data_to_save: ExecutionDataToSave) -> bool {
    match execution_data_to_save {
        ExecutionDataToSave::Trace | ExecutionDataToSave::TraceAndProfile => true,
        ExecutionDataToSave::None => false,
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
        }
    }
}
