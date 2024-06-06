use crate::expected_result::ExpectedTestResult;
use cairo_lang_sierra::ids::GenericTypeId;
use cairo_lang_sierra::program::{ProgramArtifact, VersionedProgram};
use serde::Deserialize;
use starknet_api::block::BlockNumber;
use std::num::NonZeroU32;
use url::Url;

#[derive(Debug, Clone)]
pub struct TestTarget {
    pub tests_location: CrateLocation,
    pub sierra_program: ProgramArtifact,
    pub test_cases: Vec<TestCase>,
}

#[derive(Debug, Clone)]
pub struct TestTargetWithConfig {
    pub tests_location: CrateLocation,
    pub sierra_program: ProgramArtifact,
    pub test_cases: Vec<TestCaseWithConfig>,
}
#[derive(Debug, Clone)]
pub struct TestTargetWithResolvedConfig {
    pub tests_location: CrateLocation,
    pub sierra_program: ProgramArtifact,
    pub test_cases: Vec<TestCaseWithResolvedConfig>,
}

#[derive(Debug, PartialEq, Clone, Copy, Deserialize, Hash, Eq)]
pub enum CrateLocation {
    /// Main crate in a package
    Lib,
    /// Crate in the `tests/` directory
    Tests,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Default)]
pub struct TestDetails {
    #[serde(rename = "entry_point_offset")]
    pub sierra_entry_point_statement_idx: usize,
    pub parameter_types: Vec<(GenericTypeId, i16)>,
    pub return_types: Vec<(GenericTypeId, i16)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForkConfig {
    pub url: Url,
    pub block_number: BlockNumber,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct FuzzerConfig {
    pub fuzzer_runs: NonZeroU32,
    pub fuzzer_seed: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestCase {
    pub test_details: TestDetails,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct TestCaseWithConfig {
    pub test_case: TestCase,
    pub config: TestCaseConfig,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestCaseWithResolvedConfig {
    pub test_case: TestCase,
    pub config: TestCaseResolvedConfig,
}

#[derive(Debug, Clone)]
pub struct TestCaseConfig {
    pub available_gas: Option<usize>,
    pub ignored: bool,
    pub expected_result: ExpectedTestResult,
    pub fork_config: Option<RawForkConfig>,
    pub fuzzer_config: Option<FuzzerConfig>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestCaseResolvedConfig {
    pub available_gas: Option<usize>,
    pub ignored: bool,
    pub expected_result: ExpectedTestResult,
    pub fork_config: Option<ForkConfig>,
    pub fuzzer_config: Option<FuzzerConfig>,
}

// raws

#[derive(Debug, Clone, Deserialize)]
pub struct CompiledTestCrateRaw {
    pub sierra_program: VersionedProgram,
    pub test_cases: Vec<TestCaseRaw>,
    pub tests_location: CrateLocation,
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct TestCaseRaw {
    pub name: String,
    pub available_gas: Option<usize>,
    pub ignored: bool,
    pub expected_result: ExpectedTestResult,
    pub fork_config: Option<RawForkConfig>,
    pub fuzzer_config: Option<FuzzerConfig>,
    pub test_details: TestDetails,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum RawForkConfig {
    Id(String),
    Params(RawForkParams),
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RawForkParams {
    pub url: String,
    pub block_id_type: String,
    pub block_id_value: String,
}

impl From<CompiledTestCrateRaw> for TestTargetWithConfig {
    fn from(value: CompiledTestCrateRaw) -> Self {
        Self {
            tests_location: value.tests_location,
            sierra_program: value.sierra_program.into_v1().unwrap(),
            test_cases: value
                .test_cases
                .into_iter()
                .map(|case| TestCaseWithConfig {
                    test_case: TestCase {
                        name: case.name,
                        test_details: case.test_details,
                    },
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
