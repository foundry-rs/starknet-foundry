use crate::expected_result::ExpectedTestResult;
use cairo_lang_sierra::program::Program;
use serde::Deserialize;
use starknet_api::block::BlockNumber;
use url::Url;

#[derive(Debug, Clone)]
pub struct CompiledTestCrateRunnable {
    sierra_program: Program,
    test_cases: Vec<TestCaseRunnable>,
}

impl CompiledTestCrateRunnable {
    #[must_use]
    pub fn new(sierra_program: Program, test_cases: Vec<TestCaseRunnable>) -> Self {
        Self {
            sierra_program,
            test_cases,
        }
    }

    pub(crate) fn test_cases(&self) -> &Vec<TestCaseRunnable> {
        &self.test_cases
    }

    pub(crate) fn sierra_program(&self) -> &Program {
        &self.sierra_program
    }
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
    pub block_number: BlockNumber,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct FuzzerConfig {
    pub fuzzer_runs: u32,
    pub fuzzer_seed: u64,
}
