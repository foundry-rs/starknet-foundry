use super::{TestCase, TestTarget};
use crate::expected_result::{ExpectedPanicValue, ExpectedTestResult};
use cheatnet::runtime_extensions::forge_config_extension::config::{
    Expected, RawAvailableGasConfig, RawForgeConfig, RawForkConfig, RawFuzzerConfig,
    RawShouldPanicConfig,
};
use conversions::serde::serialize::SerializeToFeltVec;

pub type TestTargetWithConfig = TestTarget<TestCaseConfig>;

pub type TestCaseWithConfig = TestCase<TestCaseConfig>;

/// Test case with config that has not yet been resolved
/// see [`super::with_config_resolved::TestCaseResolvedConfig`] for more info
#[derive(Debug, Clone)]
pub struct TestCaseConfig {
    pub available_gas: Option<RawAvailableGasConfig>,
    pub ignored: bool,
    pub expected_result: ExpectedTestResult,
    pub fork_config: Option<RawForkConfig>,
    pub fuzzer_config: Option<RawFuzzerConfig>,
}

impl From<RawForgeConfig> for TestCaseConfig {
    fn from(value: RawForgeConfig) -> Self {
        Self {
            available_gas: value.available_gas,
            ignored: value.ignore.is_some_and(|v| v.is_ignored),
            expected_result: value.should_panic.into(),
            fork_config: value.fork,
            fuzzer_config: value.fuzzer,
        }
    }
}

impl From<Option<RawShouldPanicConfig>> for ExpectedTestResult {
    fn from(value: Option<RawShouldPanicConfig>) -> Self {
        match value {
            None => Self::Success,
            Some(RawShouldPanicConfig { expected }) => Self::Panics(match expected {
                Expected::Any => ExpectedPanicValue::Any,
                Expected::Array(arr) => ExpectedPanicValue::Exact(arr),
                Expected::ByteArray(arr) => ExpectedPanicValue::Exact(arr.serialize_with_magic()),
                Expected::ShortString(str) => ExpectedPanicValue::Exact(str.serialize_to_vec()),
            }),
        }
    }
}
