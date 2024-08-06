use super::{
    raw::{RawForkConfig, RawFuzzerConfig},
    TestCase, TestTarget,
};
use crate::expected_result::ExpectedTestResult;

pub type TestTargetWithConfig = TestTarget<TestCaseConfig>;

pub type TestCaseWithConfig = TestCase<TestCaseConfig>;

/// Test case with config that has not yet been resolved
/// see [`super::with_config_resolved::TestCaseResolvedConfig`] for more info
#[derive(Debug, Clone)]
pub struct TestCaseConfig {
    pub available_gas: Option<usize>,
    pub ignored: bool,
    pub expected_result: ExpectedTestResult,
    pub fork_config: Option<RawForkConfig>,
    pub fuzzer_config: Option<RawFuzzerConfig>,
}
