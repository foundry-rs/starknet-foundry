use super::{TestCase, TestTarget};
use crate::{
    expected_result::{ExpectedPanicValue, ExpectedTestResult},
    filtering::TestCaseIsIgnored,
};
use cheatnet::runtime_extensions::forge_config_extension::config::{
    Expected, ExpectedTupleItem, RawAvailableResourceBoundsConfig, RawForgeConfig, RawForkConfig,
    RawFuzzerConfig, RawShouldPanicConfig,
};
use conversions::serde::serialize::SerializeToFeltVec;
use starknet_types_core::felt::Felt;

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
                        .flat_map(serialize_expected_tuple_item)
                        .collect(),
                ),
                Expected::ByteArray(arr) => ExpectedPanicValue::Exact(arr.serialize_with_magic()),
                Expected::ShortString(str) => ExpectedPanicValue::Exact(str.serialize_to_vec()),
            }),
        }
    }
}

fn serialize_expected_tuple_item(value: ExpectedTupleItem) -> Vec<Felt> {
    match value {
        ExpectedTupleItem::Felt(felt) => vec![felt],
        ExpectedTupleItem::ByteArray(byte_array) => {
            // If byte array is a standalone value, it should be serialized with magic,
            // but if it's part of a tuple, it should be serialized without magic.
            byte_array.serialize_to_vec()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use conversions::byte_array::ByteArray;
    use starknet_types_core::felt::Felt;

    #[test]
    fn should_panic_tuple_strings_are_flattened_without_magic() {
        let expected = ExpectedTestResult::from(Some(RawShouldPanicConfig {
            expected: Expected::Array(vec![
                ExpectedTupleItem::ByteArray(ByteArray::from("error")),
                ExpectedTupleItem::Felt(Felt::from(11_u8)),
                ExpectedTupleItem::ByteArray(ByteArray::from("hello")),
                ExpectedTupleItem::Felt(Felt::from(5_u8)),
                ExpectedTupleItem::Felt(Felt::from_bytes_be_slice(b"short_string")),
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

    #[test]
    fn should_panic_standalone_string_uses_bytearray_magic() {
        let byte_array = ByteArray::from("error");
        let expected = ExpectedTestResult::from(Some(RawShouldPanicConfig {
            expected: Expected::ByteArray(byte_array.clone()),
        }));

        assert_eq!(
            expected,
            ExpectedTestResult::Panics(ExpectedPanicValue::Exact(
                byte_array.serialize_with_magic()
            ))
        );
    }

    #[test]
    fn should_panic_tuple_empty_string() {
        let expected = ExpectedTestResult::from(Some(RawShouldPanicConfig {
            expected: Expected::Array(vec![
                ExpectedTupleItem::ByteArray(ByteArray::from("")),
                ExpectedTupleItem::Felt(Felt::from(11_u8)),
                ExpectedTupleItem::ByteArray(ByteArray::from("hello")),
            ]),
        }));

        let mut expected_data = ByteArray::from("").serialize_to_vec();
        expected_data.push(Felt::from(11_u8));
        expected_data.extend(ByteArray::from("hello").serialize_to_vec());

        assert_eq!(
            expected,
            ExpectedTestResult::Panics(ExpectedPanicValue::Exact(expected_data))
        );
    }

    #[test]
    fn should_panic_tuple_long_string() {
        let long_string = "this string is definitely longer than thirty one bytes";
        let expected = ExpectedTestResult::from(Some(RawShouldPanicConfig {
            expected: Expected::Array(vec![
                ExpectedTupleItem::ByteArray(ByteArray::from(long_string)),
                ExpectedTupleItem::Felt(Felt::from(5_u8)),
            ]),
        }));

        let mut expected_data = ByteArray::from(long_string).serialize_to_vec();
        expected_data.push(Felt::from(5_u8));

        assert_eq!(
            expected,
            ExpectedTestResult::Panics(ExpectedPanicValue::Exact(expected_data))
        );
    }

    #[test]
    fn should_panic_standalone_and_tuple_single_string_use_different_encodings() {
        let standalone = ExpectedTestResult::from(Some(RawShouldPanicConfig {
            expected: Expected::ByteArray(ByteArray::from("error")),
        }));
        let tuple = ExpectedTestResult::from(Some(RawShouldPanicConfig {
            expected: Expected::Array(vec![ExpectedTupleItem::ByteArray(ByteArray::from("error"))]),
        }));

        assert_ne!(standalone, tuple);
    }
}
