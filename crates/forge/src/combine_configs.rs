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
            deterministic_output: args.deterministic_output,
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

#[cfg(test)]
mod tests {
    use super::combine_configs;
    use crate::TestArgs;
    use crate::scarb::config::ForgeConfigFromScarb;
    use camino::Utf8PathBuf;
    use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
    use clap::Parser;
    use forge_runner::debugging::TraceArgs;
    use forge_runner::forge_config::{
        ExecutionDataToSave, ForgeConfig, ForgeTrackedResource, OutputConfig, TestRunnerConfig,
    };
    use std::num::NonZeroU32;
    use std::sync::Arc;

    #[test]
    fn fuzzer_default_seed() {
        let args = TestArgs::parse_from(["snforge"]);
        let config = combine_configs(
            &args,
            ContractsData::default(),
            Utf8PathBuf::default(),
            &ForgeConfigFromScarb::default(),
        );
        let config2 = combine_configs(
            &args,
            ContractsData::default(),
            Utf8PathBuf::default(),
            &ForgeConfigFromScarb::default(),
        );

        assert_ne!(config.test_runner_config.fuzzer_seed, 0);
        assert_ne!(config2.test_runner_config.fuzzer_seed, 0);
        assert_ne!(
            config.test_runner_config.fuzzer_seed,
            config2.test_runner_config.fuzzer_seed
        );
    }

    #[test]
    fn runner_config_default_arguments() {
        let args = TestArgs::parse_from(["snforge"]);
        let config = combine_configs(
            &args,
            ContractsData::default(),
            Utf8PathBuf::default(),
            &ForgeConfigFromScarb::default(),
        );
        assert_eq!(
            config,
            ForgeConfig {
                test_runner_config: Arc::new(TestRunnerConfig {
                    exit_first: false,
                    deterministic_output: false,
                    fuzzer_runs: NonZeroU32::new(256).unwrap(),
                    fuzzer_seed: config.test_runner_config.fuzzer_seed,
                    max_n_steps: None,
                    tracked_resource: ForgeTrackedResource::SierraGas,
                    is_vm_trace_needed: false,
                    cache_dir: Utf8PathBuf::default(),
                    contracts_data: ContractsData::default(),
                    environment_variables: config.test_runner_config.environment_variables.clone(),
                    launch_debugger: false,
                }),
                output_config: Arc::new(OutputConfig {
                    detailed_resources: false,
                    execution_data_to_save: ExecutionDataToSave::default(),
                    trace_args: TraceArgs::default(),
                    gas_report: false,
                }),
            }
        );
    }

    #[test]
    fn runner_config_just_scarb_arguments() {
        let config_from_scarb = ForgeConfigFromScarb {
            exit_first: true,
            fork: vec![],
            fuzzer_runs: Some(NonZeroU32::new(1234).unwrap()),
            fuzzer_seed: Some(500),
            detailed_resources: true,
            save_trace_data: true,
            build_profile: true,
            coverage: true,
            gas_report: true,
            max_n_steps: Some(1_000_000),
            tracked_resource: ForgeTrackedResource::CairoSteps,
        };

        let args = TestArgs::parse_from(["snforge"]);
        let config = combine_configs(
            &args,
            ContractsData::default(),
            Utf8PathBuf::default(),
            &config_from_scarb,
        );
        assert_eq!(
            config,
            ForgeConfig {
                test_runner_config: Arc::new(TestRunnerConfig {
                    exit_first: true,
                    deterministic_output: false,
                    fuzzer_runs: NonZeroU32::new(1234).unwrap(),
                    fuzzer_seed: 500,
                    max_n_steps: Some(1_000_000),
                    // tracked_resource comes from args only; ForgeConfigFromScarb.tracked_resource
                    // is not used by combine_configs, so this stays at the args default.
                    tracked_resource: ForgeTrackedResource::SierraGas,
                    is_vm_trace_needed: true,
                    cache_dir: Utf8PathBuf::default(),
                    contracts_data: ContractsData::default(),
                    environment_variables: config.test_runner_config.environment_variables.clone(),
                    launch_debugger: false,
                }),
                output_config: Arc::new(OutputConfig {
                    detailed_resources: true,
                    execution_data_to_save: ExecutionDataToSave {
                        trace: true,
                        profile: true,
                        coverage: true,
                        additional_args: vec![],
                    },
                    trace_args: TraceArgs::default(),
                    gas_report: true,
                }),
            }
        );
    }

    #[test]
    fn runner_config_argument_precedence() {
        let config_from_scarb = ForgeConfigFromScarb {
            exit_first: false,
            fork: vec![],
            fuzzer_runs: Some(NonZeroU32::new(1234).unwrap()),
            fuzzer_seed: Some(1000),
            detailed_resources: false,
            save_trace_data: false,
            build_profile: false,
            coverage: false,
            gas_report: false,
            max_n_steps: Some(1234),
            tracked_resource: ForgeTrackedResource::SierraGas,
        };
        // Note: --build-profile and --coverage conflict in clap, so only one can be used at a time.
        // We use --save-trace-data + --build-profile here to verify precedence for trace/profile flags.
        let args = TestArgs::parse_from([
            "snforge",
            "--exit-first",
            "--fuzzer-runs",
            "100",
            "--fuzzer-seed",
            "32",
            "--detailed-resources",
            "--save-trace-data",
            "--build-profile",
            "--gas-report",
            "--max-n-steps",
            "1000000",
            "--tracked-resource",
            "cairo-steps",
        ]);
        let config = combine_configs(
            &args,
            ContractsData::default(),
            Utf8PathBuf::default(),
            &config_from_scarb,
        );

        assert_eq!(
            config,
            ForgeConfig {
                test_runner_config: Arc::new(TestRunnerConfig {
                    exit_first: true,
                    deterministic_output: false,
                    fuzzer_runs: NonZeroU32::new(100).unwrap(),
                    fuzzer_seed: 32,
                    max_n_steps: Some(1_000_000),
                    tracked_resource: ForgeTrackedResource::CairoSteps,
                    is_vm_trace_needed: true,
                    cache_dir: Utf8PathBuf::default(),
                    contracts_data: ContractsData::default(),
                    environment_variables: config.test_runner_config.environment_variables.clone(),
                    launch_debugger: false,
                }),
                output_config: Arc::new(OutputConfig {
                    detailed_resources: true,
                    execution_data_to_save: ExecutionDataToSave {
                        trace: true,
                        profile: true,
                        coverage: false,
                        additional_args: vec![],
                    },
                    trace_args: TraceArgs::default(),
                    gas_report: true,
                }),
            }
        );
    }
}
