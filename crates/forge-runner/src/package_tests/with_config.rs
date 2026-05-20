use super::{TestCase, TestTarget};
use crate::{
    expected_result::{ExpectedPanicValue, ExpectedTestResult},
    filtering::TestCaseIsIgnored,
};
use cheatnet::runtime_extensions::forge_config_extension::config::{
    Expected, ExpectedValue, RawAvailableResourceBoundsConfig, RawForgeConfig, RawForkConfig,
    RawFuzzerConfig, RawShouldPanicConfig,
};
use conversions::serde::serialize::SerializeToFeltVec;

pub type TestTargetWithConfig = TestTarget<TestCaseConfig>;

pub type TestCaseWithConfig = TestCase<TestCaseConfig>;

/// Test case with config that has not yet been resolved
/// see [`super::with_config_resolved::TestCaseResolvedConfig`] for more info
#[derive(Debug, Clone)]
pub struct TestCaseConfig {
    pub available_gas: Option<RawAvailableResourceBoundsConfig>,
    pub ignored: bool,
    pub expected_result: ExpectedTestResult,
    pub fork_config: Option<RawForkConfig>,
    pub fuzzer_config: Option<RawFuzzerConfig>,
    pub disable_predeployed_contracts: bool,
}

impl TestCaseIsIgnored for TestCaseConfig {
    fn is_ignored(&self) -> bool {
        self.ignored
    }
}

impl From<RawForgeConfig> for TestCaseConfig {
    fn from(value: RawForgeConfig) -> Self {
        Self {
            available_gas: value.available_gas,
            ignored: value.ignore.is_some_and(|v| v.is_ignored),
            expected_result: value.should_panic.into(),
            fork_config: value.fork,
            fuzzer_config: value.fuzzer,
            disable_predeployed_contracts: value
                .disable_predeployed_contracts
                .is_some_and(|v| v.is_disabled),
        }
    }
}

impl From<Option<RawShouldPanicConfig>> for ExpectedTestResult {
    fn from(value: Option<RawShouldPanicConfig>) -> Self {
        match value {
            None => Self::Success,
            Some(RawShouldPanicConfig { expected }) => Self::Panics(match expected {
                Expected::Any => ExpectedPanicValue::Any,
                Expected::Array(arr) => ExpectedPanicValue::Exact(
                    arr.into_iter()
                        .flat_map(expected_value_into_felts)
                        .collect(),
                ),
                Expected::ByteArray(arr) => ExpectedPanicValue::Exact(arr.serialize_with_magic()),
                Expected::ShortString(str) => ExpectedPanicValue::Exact(str.serialize_to_vec()),
            }),
        }
    }
}

fn expected_value_into_felts(value: ExpectedValue) -> Vec<starknet_types_core::felt::Felt> {
    match value {
        ExpectedValue::Felt(felt) => vec![felt],
        ExpectedValue::ByteArray(byte_array) => byte_array.serialize_to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use conversions::byte_array::ByteArray;
    use starknet_types_core::felt::Felt;

    #[test]
    fn mixed_tuple_byte_arrays_are_serialized_without_magic() {
        let expected = ExpectedTestResult::from(Some(RawShouldPanicConfig {
            expected: Expected::Array(vec![
                ExpectedValue::ByteArray(ByteArray::from("error")),
                ExpectedValue::Felt(Felt::from(11_u8)),
                ExpectedValue::ByteArray(ByteArray::from("hello")),
                ExpectedValue::Felt(Felt::from(5_u8)),
                ExpectedValue::Felt(Felt::from_bytes_be_slice(b"short_string")),
            ]),
        }));

        assert_eq!(
            expected,
            ExpectedTestResult::Panics(ExpectedPanicValue::Exact(vec![
                Felt::from(0_u8),
                Felt::from_bytes_be_slice(b"error"),
                Felt::from(5_u8),
                Felt::from(11_u8),
                Felt::from(0_u8),
                Felt::from_bytes_be_slice(b"hello"),
                Felt::from(5_u8),
                Felt::from(5_u8),
                Felt::from_bytes_be_slice(b"short_string"),
            ]))
        );
    }
}
