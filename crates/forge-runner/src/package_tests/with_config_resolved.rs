use super::{raw::RawFuzzerConfig, TestCase, TestTarget};
use crate::expected_result::ExpectedTestResult;
use starknet_api::block::BlockNumber;
use url::Url;

pub type TestTargetWithResolvedConfig = TestTarget<TestCaseResolvedConfig>;

pub type TestCaseWithResolvedConfig = TestCase<TestCaseResolvedConfig>;

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedForkConfig {
    pub url: Url,
    pub block_number: BlockNumber,
}

/// Test case with config that has been resolved (`#[fork("name")]` -> url and block id etc.)
#[derive(Debug, Clone, PartialEq)]
pub struct TestCaseResolvedConfig {
    pub available_gas: Option<usize>,
    pub ignored: bool,
    pub expected_result: ExpectedTestResult,
    pub fork_config: Option<ResolvedForkConfig>,
    pub fuzzer_config: Option<RawFuzzerConfig>,
}
