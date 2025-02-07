// Our custom structs used to prevent name changes in structs on side of cairo compiler from breaking the test collector backwards compatibility
use cairo_lang_test_plugin::test_config::{PanicExpectation, TestExpectation};
use serde::Deserialize;
use starknet_types_core::felt::Felt;

/// Expectation for a panic case.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum ExpectedPanicValue {
    /// Accept any panic value.
    Any,
    /// Accept only this specific vector of panics.
    Exact(Vec<Felt>),
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
