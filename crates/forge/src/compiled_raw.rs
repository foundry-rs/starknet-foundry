use cairo_lang_sierra::program::Program;
use forge_runner::compiled_runnable::{FuzzerConfig, ValidatedForkConfig};
use forge_runner::expected_result::ExpectedTestResult;
use serde::Deserialize;
use starknet::core::types::BlockId;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CompiledTestCrateRaw {
    pub sierra_program: Program,
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
}

#[derive(Debug, PartialEq, Clone, Copy, Deserialize)]
pub enum CrateLocation {
    /// Main crate in a package
    Lib,
    /// Crate in the `tests/` directory
    Tests,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum RawForkConfig {
    Id(String),
    Params(RawForkParams),
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RawForkParams {
    pub url: String,
    pub block_id: BlockId,
}

impl TryFrom<RawForkParams> for ValidatedForkConfig {
    type Error = anyhow::Error;

    fn try_from(value: RawForkParams) -> Result<Self, Self::Error> {
        Ok(ValidatedForkConfig {
            url: value.url.parse()?,
            block_id: value.block_id,
        })
    }
}
