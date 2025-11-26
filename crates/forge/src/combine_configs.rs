use crate::TestArgs;
use crate::scarb::config::ForgeConfigFromScarb;
use camino::Utf8PathBuf;
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use forge_runner::forge_config::{
    ExecutionDataToSave, ForgeConfig, OutputConfig, TestRunnerConfig,
};
use rand::{RngCore, thread_rng};
use std::env;
use std::num::NonZeroU32;
use std::sync::Arc;

pub fn combine_configs(
    args: &TestArgs,
    contracts_data: ContractsData,
    cache_dir: Utf8PathBuf,
    forge_config_from_scarb: &ForgeConfigFromScarb,
) -> ForgeConfig {
    let execution_data_to_save = ExecutionDataToSave::from_flags(
        args.save_trace_data || forge_config_from_scarb.save_trace_data,
        args.build_profile || forge_config_from_scarb.build_profile,
        args.coverage || forge_config_from_scarb.coverage,
        &args.additional_args,
    );

    ForgeConfig {
        test_runner_config: Arc::new(TestRunnerConfig {
            exit_first: args.exit_first || forge_config_from_scarb.exit_first,
            fuzzer_runs: args
                .fuzzer_runs
                .or(forge_config_from_scarb.fuzzer_runs)
                .unwrap_or(NonZeroU32::new(256).unwrap()),
            fuzzer_seed: args
                .fuzzer_seed
                .or(forge_config_from_scarb.fuzzer_seed)
                .unwrap_or_else(|| thread_rng().next_u64()),
            max_n_steps: args.max_n_steps.or(forge_config_from_scarb.max_n_steps),
            is_vm_trace_needed: execution_data_to_save.is_vm_trace_needed(),
            cache_dir,
            contracts_data,
            tracked_resource: args.tracked_resource,
            environment_variables: env::vars().collect(),
            launch_debugger: args.launch_debugger,
        }),
        output_config: Arc::new(OutputConfig {
            trace_args: args.trace_args.clone(),
            detailed_resources: args.detailed_resources
                || forge_config_from_scarb.detailed_resources,
            execution_data_to_save,
            gas_report: args.gas_report || forge_config_from_scarb.gas_report,
        }),
    }
}
