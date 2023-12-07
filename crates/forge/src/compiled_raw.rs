use cairo_felt::Felt252;
use cairo_lang_sierra::program::Program;
use conversions::IntoConv;
use forge_runner::compiled_runnable::{FuzzerConfig, ValidatedForkConfig};
use forge_runner::expected_result::ExpectedTestResult;
use num_bigint::BigInt;
use serde::Deserialize;
use starknet::core::types::BlockId;
use starknet::core::types::BlockTag::Latest;

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
    pub block_id_type: String,
    pub block_id_value: String,
}

impl TryFrom<RawForkParams> for ValidatedForkConfig {
    type Error = anyhow::Error;

    fn try_from(value: RawForkParams) -> Result<Self, Self::Error> {
        let block_id = match value.block_id_type.to_lowercase().as_str() {
            "number" => BlockId::Number(value.block_id_value.parse().unwrap()),
            "hash" => BlockId::Hash(
                Felt252::from(value.block_id_value.parse::<BigInt>().unwrap()).into_(),
            ),
            "tag" => {
                assert_eq!(value.block_id_value, "Latest");
                BlockId::Tag(Latest)
            }
            _ => unreachable!(),
        };
        Ok(ValidatedForkConfig {
            url: value.url.parse()?,
            block_id,
        })
    }
}
