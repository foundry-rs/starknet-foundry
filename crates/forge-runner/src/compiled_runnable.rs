use cairo_felt::Felt252;
use cairo_lang_sierra::program::Program;
use conversions::IntoConv;
use num_bigint::BigInt;
use snforge_test_collector_interface::{ExpectedTestResult, FuzzerConfig, RawForkParams};
use starknet::core::types::BlockId;
use starknet::core::types::BlockTag::Latest;
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
}

#[derive(Debug, Clone)]
pub struct ValidatedForkConfig {
    pub url: Url,
    pub block_id: BlockId,
}

impl TryInto<ValidatedForkConfig> for RawForkParams {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<ValidatedForkConfig, Self::Error> {
        let block_id = match self.block_id_type.to_lowercase().as_str() {
            "number" => BlockId::Number(self.block_id_value.parse().unwrap()),
            "hash" => {
                BlockId::Hash(Felt252::from(self.block_id_value.parse::<BigInt>().unwrap()).into_())
            }
            "tag" => {
                assert_eq!(self.block_id_value, "Latest");
                BlockId::Tag(Latest)
            }
            _ => unreachable!(),
        };
        Ok(ValidatedForkConfig {
            url: self.url.parse()?,
            block_id,
        })
    }
}
