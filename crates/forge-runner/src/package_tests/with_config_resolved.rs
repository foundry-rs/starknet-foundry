use super::{TestCase, TestTargetLocation};
use crate::expected_result::ExpectedTestResult;
use cairo_lang_sierra::program::ProgramArtifact;
use serde::Deserialize;
use starknet_api::block::BlockNumber;
use std::num::NonZeroU32;
use url::Url;

#[derive(Debug, Clone)]
pub struct TestTargetWithResolvedConfig {
    pub tests_location: TestTargetLocation,
    pub sierra_program: ProgramArtifact,
    pub test_cases: Vec<TestCaseWithResolvedConfig>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedForkConfig {
    pub url: Url,
    pub block_number: BlockNumber,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ResolvedFuzzerConfig {
    pub fuzzer_runs: NonZeroU32,
    pub fuzzer_seed: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestCaseWithResolvedConfig {
    pub test_case: TestCase,
    pub config: TestCaseResolvedConfig,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestCaseResolvedConfig {
    pub available_gas: Option<usize>,
    pub ignored: bool,
    pub expected_result: ExpectedTestResult,
    pub fork_config: Option<ResolvedForkConfig>,
    pub fuzzer_config: Option<ResolvedFuzzerConfig>,
}
