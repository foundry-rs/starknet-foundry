use std::num::NonZeroU32;
use std::sync::Arc;

#[derive(Debug, PartialEq)]
pub struct ForgeConfig {
    pub runner_config: Arc<RunnerConfig>,
    pub runtime_config: Arc<RuntimeConfig>,
    pub output_config: OutputConfig,
}

#[derive(Debug, PartialEq)]
pub struct RunnerConfig {
    pub exit_first: bool,
    pub fuzzer_runs: NonZeroU32,
    pub fuzzer_seed: u64,
}

#[derive(Debug, PartialEq)]
pub struct RuntimeConfig {
    pub max_n_steps: Option<u32>,
}

#[derive(Debug, PartialEq)]
pub struct OutputConfig {
    pub detailed_resources: bool,
    pub execution_data_to_save: ExecutionDataToSave,
}

impl OutputConfig {
    #[must_use]
    pub fn new(detailed_resources: bool, save_trace_data: bool, build_profile: bool) -> Self {
        Self {
            detailed_resources,
            execution_data_to_save: ExecutionDataToSave::from_flags(save_trace_data, build_profile),
        }
    }
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

impl ExecutionDataToSave {
    #[must_use]
    pub fn is_vm_trace_needed(self) -> bool {
        match self {
            ExecutionDataToSave::Trace | ExecutionDataToSave::TraceAndProfile => true,
            ExecutionDataToSave::None => false,
        }
    }
}
