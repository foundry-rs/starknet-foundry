use super::{TestCase, TestTarget};
use crate::expected_result::ExpectedTestResult;
use cheatnet::runtime_extensions::forge_config_extension::config::{
    RawAvailableGasConfig, RawFuzzerConfig,
};
use starknet_api::block::BlockNumber;
use url::Url;

pub type TestTargetWithResolvedConfig = TestTarget<TestCaseResolvedConfig>;

pub type TestCaseWithResolvedConfig = TestCase<TestCaseResolvedConfig>;

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedForkConfig {
    pub url: Url,
    pub block_number: BlockNumber,
}

/// Test case with config that has been resolved, that is
///     `#[fork("name")]` -> url and block id
///     fetches block number
#[derive(Debug, Clone, PartialEq)]
pub struct TestCaseResolvedConfig {
    pub available_gas: Option<RawAvailableGasConfig>,
    pub ignored: bool,
    pub expected_result: ExpectedTestResult,
    pub fork_config: Option<ResolvedForkConfig>,
    pub fuzzer_config: Option<RawFuzzerConfig>,
    pub disable_strk_predeployment: bool,
}
