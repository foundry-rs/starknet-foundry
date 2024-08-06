use super::{
    with_config::{TestCaseConfig, TestCaseWithConfig, TestTargetWithConfig},
    TestDetails, TestTargetLocation,
};
use crate::expected_result::ExpectedTestResult;
use cairo_lang_sierra::program::VersionedProgram;
use serde::Deserialize;
use std::num::NonZeroU32;

/// these structs are representation of scarb output for `scarb build --test`

/// produced by scarb
#[derive(Debug, Clone, Deserialize)]
pub struct TestTargetRaw {
    pub sierra_program: VersionedProgram,
    pub test_cases: Vec<TestCaseRaw>,
    pub tests_location: TestTargetLocation,
}

impl TestTargetRaw {
    #[must_use]
    pub fn with_config(self) -> TestTargetWithConfig {
        TestTargetWithConfig {
            tests_location: self.tests_location,
            sierra_program: self.sierra_program.into_v1().unwrap(),
            test_cases: self
                .test_cases
                .into_iter()
                .map(|case| TestCaseWithConfig {
                    name: case.name,
                    test_details: case.test_details,
                    config: TestCaseConfig {
                        available_gas: case.available_gas,
                        ignored: case.ignored,
                        expected_result: case.expected_result,
                        fork_config: case.fork_config,
                        fuzzer_config: case.fuzzer_config,
                    },
                })
                .collect(),
        }
    }
}

/// produced by scarb
#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct TestCaseRaw {
    pub name: String,
    pub available_gas: Option<usize>,
    pub ignored: bool,
    pub expected_result: ExpectedTestResult,
    pub fork_config: Option<RawForkConfig>,
    pub fuzzer_config: Option<RawFuzzerConfig>,
    pub test_details: TestDetails,
}

/// produced by scarb
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum RawForkConfig {
    Id(String),
    Params(RawForkParams),
}

/// produced by scarb
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RawForkParams {
    pub url: String,
    pub block_id_type: String,
    pub block_id_value: String,
}

/// produced by scarb
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RawFuzzerConfig {
    pub fuzzer_runs: NonZeroU32,
    pub fuzzer_seed: u64,
}
