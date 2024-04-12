use crate::expected_result::ExpectedTestResult;
use cairo_lang_sierra::{ids::GenericTypeId, program::Program};
use serde::Deserialize;
use starknet_api::block::BlockNumber;
use std::num::NonZeroU32;
use url::Url;

#[derive(Debug, Clone)]
pub struct CompiledTestCrateRunnable {
    pub sierra_program: Program,
    pub test_cases: Vec<TestCaseRunnable>,
}

#[derive(Debug, Clone)]
pub struct TestCaseRunnable {
    pub name: String,
    pub available_gas: Option<usize>,
    pub ignored: bool,
    pub expected_result: ExpectedTestResult,
    pub fork_config: Option<ValidatedForkConfig>,
    pub fuzzer_config: Option<FuzzerConfig>,
    pub test_details: TestDetails,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Default)]
pub struct TestDetails {
    #[serde(rename = "entry_point_offset")]
    pub sierra_entry_point_statement_idx: usize,
    pub parameter_types: Vec<(GenericTypeId, i16)>,
    pub return_types: Vec<(GenericTypeId, i16)>,
}

#[derive(Debug, Clone)]
pub struct ValidatedForkConfig {
    pub url: Url,
    pub block_number: BlockNumber,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct FuzzerConfig {
    pub fuzzer_runs: NonZeroU32,
    pub fuzzer_seed: u64,
}
