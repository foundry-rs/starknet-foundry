use super::{TestCase, TestTarget};
use crate::expected_result::ExpectedTestResult;
use anyhow::Result;
use cairo_vm::types::program::Program;
use cheatnet::runtime_extensions::forge_config_extension::config::{
    RawAvailableGasConfig, RawFuzzerConfig,
};
use starknet_api::block::BlockNumber;
use universal_sierra_compiler_api::AssembledProgramWithDebugInfo;
use url::Url;

pub type TestTargetWithResolvedConfig = TestTarget<TestCaseResolvedConfig>;

pub type TestCaseWithResolvedConfig = TestCase<TestCaseResolvedConfig>;

impl TestCaseWithResolvedConfig {
    pub fn try_into_program(
        &self,
        casm_program: &AssembledProgramWithDebugInfo,
    ) -> Result<Program> {
        self.test_details.try_into_program(casm_program)
    }
}

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
    pub disable_predeployed_contracts: bool,
}
