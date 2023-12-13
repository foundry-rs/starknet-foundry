use cairo_felt::Felt252;
use cairo_lang_sierra::program::Program;
use conversions::IntoConv;
use forge_runner::compiled_runnable::{
    CompiledTestCrateRunnable, CrateLocation, FuzzerConfig, TestCaseRunnable, ValidatedForkConfig,
};
use forge_runner::expected_result::ExpectedTestResult;
use num_bigint::BigInt;
use serde::Deserialize;
use starknet::core::types::BlockId;
use starknet::core::types::BlockTag::Latest;

use crate::scarb::config::ForkTarget;
use anyhow::{anyhow, Result};

#[derive(Debug, Clone, Deserialize)]
pub struct CompiledTestCrateRaw {
    pub sierra_program: Program,
    test_cases: Vec<TestCaseRaw>,
    tests_location: CrateLocation,
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub(crate) struct TestCaseRaw {
    pub name: String,
    pub available_gas: Option<usize>,
    pub ignored: bool,
    pub expected_result: ExpectedTestResult,
    pub fork_config: Option<RawForkConfig>,
    pub fuzzer_config: Option<FuzzerConfig>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub(crate) enum RawForkConfig {
    Id(String),
    Params(RawForkParams),
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RawForkParams {
    pub url: String,
    pub block_id_type: String,
    pub block_id_value: String,
}

impl TryFrom<RawForkParams> for ValidatedForkConfig {
    type Error = anyhow::Error;

    fn try_from(value: RawForkParams) -> Result<Self, Self::Error> {
        let block_id = match value.block_id_type.to_lowercase().as_str() {
            "number" => BlockId::Number(value.block_id_value.parse().unwrap()),
            "hash" => BlockId::Hash(
                Felt252::from(value.block_id_value.parse::<BigInt>().unwrap()).into_(),
            ),
            "tag" => {
                assert_eq!(value.block_id_value, "Latest");
                BlockId::Tag(Latest)
            }
            _ => unreachable!(),
        };
        Ok(ValidatedForkConfig {
            url: value.url.parse()?,
            block_id,
        })
    }
}

impl CompiledTestCrateRaw {
    pub(crate) fn to_runnable(
        self: Self,
        fork_targets: &[ForkTarget],
    ) -> Result<CompiledTestCrateRunnable> {
        let mut test_cases = vec![];

        for case in self.test_cases {
            let fork_config = if let Some(fc) = case.fork_config {
                let raw_fork_params = fc.replace_id_with_params(fork_targets)?;
                let fork_config = ValidatedForkConfig::try_from(raw_fork_params)?;
                Some(fork_config)
            } else {
                None
            };

            test_cases.push(TestCaseRunnable {
                name: case.name,
                available_gas: case.available_gas,
                ignored: case.ignored,
                expected_result: case.expected_result,
                fork_config,
                fuzzer_config: case.fuzzer_config,
            });
        }

        Ok(CompiledTestCrateRunnable {
            tests_location: self.tests_location,
            sierra_program: self.sierra_program,
            test_cases,
        })
    }
}

impl RawForkConfig {
    fn replace_id_with_params(self: Self, fork_targets: &[ForkTarget]) -> Result<RawForkParams> {
        match self {
            RawForkConfig::Params(raw_fork_params) => Ok(raw_fork_params),
            RawForkConfig::Id(name) => {
                let fork_target_from_runner_config = fork_targets
                    .iter()
                    .find(|fork| fork.name() == name)
                    .ok_or_else(|| {
                        anyhow!("Fork configuration named = {name} not found in the Scarb.toml")
                    })?;

                Ok(fork_target_from_runner_config.params().clone())
            }
        }
    }
}
