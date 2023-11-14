use cairo_felt::Felt252;
use cairo_lang_sierra::program::Program;
use cairo_lang_test_plugin::test_config::{PanicExpectation, TestExpectation};
use serde::Deserialize;
use starknet::core::types::BlockId;
use url::Url;

use crate::CrateLocation;

pub(crate) type CompiledTestCrateRaw = CompiledTestCrate<RawForkConfig>;
pub(crate) type CompiledTestCrateRunnable = CompiledTestCrate<ValidatedForkConfig>;

pub(crate) type TestCaseRunnable = TestCase<ValidatedForkConfig>;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CompiledTestCrate<T: ForkConfig> {
    pub sierra_program: Program,
    pub test_cases: Vec<TestCase<T>>,
    pub tests_location: CrateLocation,
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct TestCase<T: ForkConfig> {
    pub name: String,
    pub available_gas: Option<usize>,
    pub ignored: bool,
    pub expected_result: ExpectedTestResult,
    pub fork_config: Option<T>,
    pub fuzzer_config: Option<FuzzerConfig>,
}

pub trait ForkConfig {}

impl ForkConfig for ValidatedForkConfig {}

#[derive(Debug, Clone, PartialEq)]
pub struct ValidatedForkConfig {
    pub url: Url,
    pub block_id: BlockId,
}

impl ForkConfig for RawForkConfig {}

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

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct FuzzerConfig {
    pub fuzzer_runs: u32,
    pub fuzzer_seed: u64,
}

/// Expectation for a panic case.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum ExpectedPanicValue {
    /// Accept any panic value.
    Any,
    /// Accept only this specific vector of panics.
    Exact(Vec<Felt252>),
}

impl From<PanicExpectation> for ExpectedPanicValue {
    fn from(value: PanicExpectation) -> Self {
        match value {
            PanicExpectation::Any => ExpectedPanicValue::Any,
            PanicExpectation::Exact(vec) => ExpectedPanicValue::Exact(vec),
        }
    }
}

/// Expectation for a result of a test.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum ExpectedTestResult {
    /// Running the test should not panic.
    Success,
    /// Running the test should result in a panic.
    Panics(ExpectedPanicValue),
}

impl From<TestExpectation> for ExpectedTestResult {
    fn from(value: TestExpectation) -> Self {
        match value {
            TestExpectation::Success => ExpectedTestResult::Success,
            TestExpectation::Panics(panic_expectation) => {
                ExpectedTestResult::Panics(panic_expectation.into())
            }
        }
    }
}
