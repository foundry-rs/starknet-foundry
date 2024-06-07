use super::{
    raw::RawForkConfig, with_config_resolved::ResolvedFuzzerConfig, TestCase, TestTargetLocation,
};
use crate::expected_result::ExpectedTestResult;
use cairo_lang_sierra::program::ProgramArtifact;

#[derive(Debug, Clone)]
pub struct TestTargetWithConfig {
    pub tests_location: TestTargetLocation,
    pub sierra_program: ProgramArtifact,
    pub test_cases: Vec<TestCaseWithConfig>,
}

#[derive(Debug, Clone)]
pub struct TestCaseWithConfig {
    pub test_case: TestCase,
    pub config: TestCaseConfig,
}

#[derive(Debug, Clone)]
pub struct TestCaseConfig {
    pub available_gas: Option<usize>,
    pub ignored: bool,
    pub expected_result: ExpectedTestResult,
    pub fork_config: Option<RawForkConfig>,
    pub fuzzer_config: Option<ResolvedFuzzerConfig>,
}
