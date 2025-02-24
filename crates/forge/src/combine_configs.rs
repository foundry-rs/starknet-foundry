use crate::scarb::config::ForgeConfigFromScarb;
use camino::Utf8PathBuf;
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use forge_runner::forge_config::{
    ExecutionDataToSave, ForgeConfig, OutputConfig, TestRunnerConfig,
};
use rand::{RngCore, thread_rng};
use std::env;
use std::ffi::OsString;
use std::num::NonZeroU32;
use std::sync::Arc;

#[expect(clippy::too_many_arguments)]
#[expect(clippy::fn_params_excessive_bools)]
pub fn combine_configs(
    exit_first: bool,
    fuzzer_runs: Option<NonZeroU32>,
    fuzzer_seed: Option<u64>,
    detailed_resources: bool,
    save_trace_data: bool,
    build_profile: bool,
    coverage: bool,
    max_n_steps: Option<u32>,
    contracts_data: ContractsData,
    cache_dir: Utf8PathBuf,
    forge_config_from_scarb: &ForgeConfigFromScarb,
    additional_args: &[OsString],
) -> ForgeConfig {
    let execution_data_to_save = ExecutionDataToSave::from_flags(
        save_trace_data || forge_config_from_scarb.save_trace_data,
        build_profile || forge_config_from_scarb.build_profile,
        coverage || forge_config_from_scarb.coverage,
        additional_args,
    );

    ForgeConfig {
        test_runner_config: Arc::new(TestRunnerConfig {
            exit_first: exit_first || forge_config_from_scarb.exit_first,
            fuzzer_runs: fuzzer_runs
                .or(forge_config_from_scarb.fuzzer_runs)
                .unwrap_or(NonZeroU32::new(256).unwrap()),
            fuzzer_seed: fuzzer_seed
                .or(forge_config_from_scarb.fuzzer_seed)
                .unwrap_or_else(|| thread_rng().next_u64()),
            max_n_steps: max_n_steps.or(forge_config_from_scarb.max_n_steps),
            is_vm_trace_needed: execution_data_to_save.is_vm_trace_needed(),
            cache_dir,
            contracts_data,
            environment_variables: env::vars().collect(),
        }),
        output_config: Arc::new(OutputConfig {
            detailed_resources: detailed_resources || forge_config_from_scarb.detailed_resources,
            execution_data_to_save,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuzzer_default_seed() {
        let config = combine_configs(
            false,
            None,
            None,
            false,
            false,
            false,
            false,
            None,
            Default::default(),
            Default::default(),
            &Default::default(),
            &[],
        );
        let config2 = combine_configs(
            false,
            None,
            None,
            false,
            false,
            false,
            false,
            None,
            Default::default(),
            Default::default(),
            &Default::default(),
            &[],
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
        let config = combine_configs(
            false,
            None,
            None,
            false,
            false,
            false,
            false,
            None,
            Default::default(),
            Default::default(),
            &Default::default(),
            &[],
        );
        assert_eq!(
            config,
            ForgeConfig {
                test_runner_config: Arc::new(TestRunnerConfig {
                    exit_first: false,
                    fuzzer_runs: NonZeroU32::new(256).unwrap(),
                    fuzzer_seed: config.test_runner_config.fuzzer_seed,
                    max_n_steps: None,
                    is_vm_trace_needed: false,
                    cache_dir: Default::default(),
                    contracts_data: Default::default(),
                    environment_variables: config.test_runner_config.environment_variables.clone(),
                }),
                output_config: Arc::new(OutputConfig {
                    detailed_resources: false,
                    execution_data_to_save: ExecutionDataToSave::default(),
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
            max_n_steps: Some(1_000_000),
        };

        let config = combine_configs(
            false,
            None,
            None,
            false,
            false,
            false,
            false,
            None,
            Default::default(),
            Default::default(),
            &config_from_scarb,
            &[],
        );
        assert_eq!(
            config,
            ForgeConfig {
                test_runner_config: Arc::new(TestRunnerConfig {
                    exit_first: true,
                    fuzzer_runs: NonZeroU32::new(1234).unwrap(),
                    fuzzer_seed: 500,
                    max_n_steps: Some(1_000_000),
                    is_vm_trace_needed: true,
                    cache_dir: Default::default(),
                    contracts_data: Default::default(),
                    environment_variables: config.test_runner_config.environment_variables.clone(),
                }),
                output_config: Arc::new(OutputConfig {
                    detailed_resources: true,
                    execution_data_to_save: ExecutionDataToSave {
                        trace: true,
                        profile: true,
                        coverage: true,
                        additional_args: vec![],
                    },
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
            max_n_steps: Some(1234),
        };
        let config = combine_configs(
            true,
            Some(NonZeroU32::new(100).unwrap()),
            Some(32),
            true,
            true,
            true,
            true,
            Some(1_000_000),
            Default::default(),
            Default::default(),
            &config_from_scarb,
            &[],
        );

        assert_eq!(
            config,
            ForgeConfig {
                test_runner_config: Arc::new(TestRunnerConfig {
                    exit_first: true,
                    fuzzer_runs: NonZeroU32::new(100).unwrap(),
                    fuzzer_seed: 32,
                    max_n_steps: Some(1_000_000),
                    is_vm_trace_needed: true,
                    cache_dir: Default::default(),
                    contracts_data: Default::default(),
                    environment_variables: config.test_runner_config.environment_variables.clone(),
                }),
                output_config: Arc::new(OutputConfig {
                    detailed_resources: true,
                    execution_data_to_save: ExecutionDataToSave {
                        trace: true,
                        profile: true,
                        coverage: true,
                        additional_args: vec![],
                    },
                }),
            }
        );
    }
}
