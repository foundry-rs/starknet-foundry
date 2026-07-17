use crate::backtrace::{
    BacktraceSources, add_test_backtrace_footer, get_backtrace, is_backtrace_enabled,
};
use crate::build_trace_data::build_profiler_call_trace;
use crate::debugging::{TraceArgs, build_contracts_data_store, build_debugging_trace};
use crate::expected_result::{ExpectedPanicValue, ExpectedTestResult};
use crate::gas::check_available_gas;
use crate::gas::report::SingleTestGasInfo;
use crate::gas::stats::GasStats;
use crate::package_tests::with_config_resolved::TestCaseWithResolvedConfig;
use crate::running::{RunCompleted, RunStatus};
use blockifier::execution::syscalls::hint_processor::ENTRYPOINT_FAILED_ERROR_FELT;
use cairo_annotations::trace_data::VersionedCallTrace as VersionedProfilerCallTrace;
use cairo_lang_utils::byte_array::BYTE_ARRAY_MAGIC;
use camino::Utf8Path;
use cheatnet::forking::data::ForkData;
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use cheatnet::runtime_extensions::outer_call_runtime_extension::rpc::UsedResources;
use conversions::byte_array::ByteArray;
use conversions::string::TryFromHexStr;
use num_traits::ToPrimitive;
use shared::utils::build_readable_text;
use starknet_api::execution_resources::GasVector;
use starknet_api::execution_utils::format_panic_data;
use starknet_types_core::felt::Felt;
use std::fmt;
use std::option::Option;
use std::sync::LazyLock;

static BYTE_ARRAY_MAGIC_FELT: LazyLock<Felt> =
    LazyLock::new(|| TryFromHexStr::try_from_hex_str(&format!("0x{BYTE_ARRAY_MAGIC}")).unwrap());
const BYTE_ARRAY_FIXED_PART_LEN: usize = 4;

#[derive(Debug, PartialEq, Clone, Default)]
pub struct GasFuzzingInfo {
    pub l1_gas: GasStats,
    pub l1_data_gas: GasStats,
    pub l2_gas: GasStats,
}

impl fmt::Display for GasFuzzingInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "(l1_gas: {}, l1_data_gas: {}, l2_gas: {})",
            self.l1_gas, self.l1_data_gas, self.l2_gas
        )
    }
}

impl fmt::Display for GasStats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{{max: ~{}, min: ~{}, mean: ~{:.0}, std deviation: ~{:.0}}}",
            self.max, self.min, self.mean, self.std_deviation
        )
    }
}

impl GasFuzzingInfo {
    #[must_use]
    pub fn new(gas_usages: &[GasVector]) -> Self {
        let l1_gas_values: Vec<u64> = gas_usages.iter().map(|gv| gv.l1_gas.0).collect();
        let l1_data_gas_values: Vec<u64> = gas_usages.iter().map(|gv| gv.l1_data_gas.0).collect();
        let l2_gas_values: Vec<u64> = gas_usages.iter().map(|gv| gv.l2_gas.0).collect();

        GasFuzzingInfo {
            l1_gas: { GasStats::new(l1_gas_values.as_ref()) },
            l1_data_gas: { GasStats::new(l1_data_gas_values.as_ref()) },
            l2_gas: { GasStats::new(l2_gas_values.as_ref()) },
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct FuzzingStatistics {
    pub runs: usize,
}

pub trait TestType {
    type GasInfo: fmt::Debug + Clone;
    type TestStatistics: fmt::Debug + Clone;
    type TraceData: fmt::Debug + Clone;
}

#[derive(Debug, PartialEq, Clone)]
pub struct Fuzzing;
impl TestType for Fuzzing {
    type GasInfo = GasFuzzingInfo;
    type TestStatistics = FuzzingStatistics;
    type TraceData = ();
}

#[derive(Debug, PartialEq, Clone)]
pub struct Single;
impl TestType for Single {
    type GasInfo = SingleTestGasInfo;
    type TestStatistics = ();
    type TraceData = VersionedProfilerCallTrace;
}

/// Summary of running a single test case
#[expect(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum TestCaseSummary<T: TestType> {
    /// Test case passed
    Passed {
        /// Name of the test case
        name: String,
        /// Message to be printed after the test case run
        msg: Option<String>,
        /// Trace of the test case run
        debugging_trace: Option<debugging::Trace>,
        /// Information on used gas
        gas_info: <T as TestType>::GasInfo,
        /// Resources used during test
        used_resources: UsedResources,
        /// Statistics of the test run
        test_statistics: <T as TestType>::TestStatistics,
        /// Test trace data
        trace_data: <T as TestType>::TraceData,
    },
    /// Test case failed
    Failed {
        /// Name of the test case
        name: String,
        /// Message returned by the test case run
        msg: Option<String>,
        /// Trace of the test case run
        debugging_trace: Option<debugging::Trace>,
        /// Random arguments used in the fuzz test case run
        fuzzer_args: Vec<String>,
        /// Statistics of the test run
        test_statistics: <T as TestType>::TestStatistics,
    },
    /// Test case ignored due to `#[ignored]` attribute or `--ignored` flag
    Ignored {
        /// Name of the test case
        name: String,
    },
    /// Test case skipped due to exit first or execution interrupted, test result is ignored.
    Interrupted {},
    /// Test case excluded from current partition
    ExcludedFromPartition {},
}

#[derive(Debug)]
// We allow large enum variant because `Single` is the bigger variant and it is used most often
#[expect(clippy::large_enum_variant)]
pub enum AnyTestCaseSummary {
    Fuzzing(TestCaseSummary<Fuzzing>),
    Single(TestCaseSummary<Single>),
}

impl<T: TestType> TestCaseSummary<T> {
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        match self {
            TestCaseSummary::Failed { name, .. }
            | TestCaseSummary::Passed { name, .. }
            | TestCaseSummary::Ignored { name, .. } => Some(name),
            TestCaseSummary::Interrupted { .. } | TestCaseSummary::ExcludedFromPartition { .. } => {
                None
            }
        }
    }

    #[must_use]
    pub fn msg(&self) -> Option<&str> {
        match self {
            TestCaseSummary::Failed { msg: Some(msg), .. }
            | TestCaseSummary::Passed { msg: Some(msg), .. } => Some(msg),
            _ => None,
        }
    }

    #[must_use]
    pub fn debugging_trace(&self) -> Option<&debugging::Trace> {
        match self {
            TestCaseSummary::Passed {
                debugging_trace, ..
            }
            | TestCaseSummary::Failed {
                debugging_trace, ..
            } => debugging_trace.as_ref(),
            _ => None,
        }
    }
}

impl TestCaseSummary<Fuzzing> {
    #[must_use]
    pub fn from(results: Vec<TestCaseSummary<Single>>) -> Self {
        let last: TestCaseSummary<Single> = results
            .iter()
            .last()
            .cloned()
            .expect("Fuzz test should always run at least once");
        // Only the last result matters as fuzzing is cancelled after first fail
        match last {
            TestCaseSummary::Passed {
                name,
                msg,
                gas_info: _,
                used_resources: _,
                test_statistics: (),
                trace_data: _,
                debugging_trace,
            } => {
                let runs = results.len();
                let gas_usages: Vec<GasVector> = results
                    .into_iter()
                    .map(|a| match a {
                        TestCaseSummary::Passed { gas_info, .. } => gas_info.gas_used,
                        _ => unreachable!(),
                    })
                    .collect();

                TestCaseSummary::Passed {
                    name,
                    msg,
                    gas_info: GasFuzzingInfo::new(gas_usages.as_ref()),
                    used_resources: UsedResources::default(),
                    test_statistics: FuzzingStatistics { runs },
                    trace_data: (),
                    debugging_trace,
                }
            }
            TestCaseSummary::Failed {
                name,
                msg,
                fuzzer_args,
                debugging_trace,
                test_statistics: (),
            } => TestCaseSummary::Failed {
                name,
                msg,
                fuzzer_args,
                test_statistics: FuzzingStatistics {
                    runs: results.len(),
                },
                debugging_trace,
            },
            TestCaseSummary::Ignored { name } => TestCaseSummary::Ignored { name: name.clone() },
            TestCaseSummary::Interrupted {} => TestCaseSummary::Interrupted {},
            TestCaseSummary::ExcludedFromPartition {} => TestCaseSummary::ExcludedFromPartition {},
        }
    }
}

fn build_expected_panic_message(expected_panic_value: &ExpectedPanicValue) -> String {
    match expected_panic_value {
        ExpectedPanicValue::Any => "\n    Expected to panic, but no panic occurred\n".into(),
        ExpectedPanicValue::Exact(panic_data) => {
            format!(
                "\n    Expected to panic, but no panic occurred\n    Expected panic data:  {}\n",
                format_panic_data(panic_data)
            )
        }
    }
}

fn check_if_matching_and_get_message(
    actual_panic_value: &[Felt],
    expected_panic_value: &ExpectedPanicValue,
) -> (bool, Option<String>) {
    let expected_data = match expected_panic_value {
        ExpectedPanicValue::Exact(data) => Some(data),
        ExpectedPanicValue::Any => None,
    };

    match expected_data {
        Some(expected) if is_matching_should_panic_data(actual_panic_value, expected) => {
            (true, None)
        }
        Some(expected) => {
            let message = Some(format!(
                "\n    Incorrect panic data\n    {}\n    {}\n",
                format_args!(
                    "Actual:    {}",
                    format_panic_data_with_types(actual_panic_value)
                ),
                format_args!("Expected:  {}", format_panic_data_with_types(expected))
            ));
            (false, message)
        }
        None => (true, None),
    }
}

fn format_panic_data_with_types(data: &[Felt]) -> String {
    let mut formatted_values = Vec::with_capacity(data.len());
    let mut remaining = data;

    while !remaining.is_empty() {
        if let Some(byte_array_data) = take_byte_array(remaining) {
            let consumed = byte_array_data.len();
            formatted_values.push(format!("ByteArray({})", format_panic_data(byte_array_data)));
            remaining = &remaining[consumed..];
        } else {
            let (felt, rest) = remaining
                .split_first()
                .expect("remaining panic data is not empty");
            formatted_values.push(format!(
                "felt252 {}",
                format_panic_data(std::slice::from_ref(felt))
            ));
            remaining = rest;
        }
    }

    if let [single_value] = formatted_values.as_slice() {
        single_value.clone()
    } else {
        format!("({})", formatted_values.join(", "))
    }
}

fn take_byte_array(data: &[Felt]) -> Option<&[Felt]> {
    if data.first() != Some(&BYTE_ARRAY_MAGIC_FELT) {
        return None;
    }

    let words_len = data.get(1)?.to_usize()?;
    let byte_array_len = words_len.checked_add(BYTE_ARRAY_FIXED_PART_LEN)?;
    let byte_array_data = data.get(..byte_array_len)?;

    ByteArray::deserialize_with_magic(byte_array_data).ok()?;

    Some(byte_array_data)
}

impl TestCaseSummary<Single> {
    #[expect(clippy::too_many_lines)]
    #[must_use]
    pub(crate) fn from_run_completed(
        RunCompleted {
            status,
            call_trace,
            gas_used,
            used_resources,
            encountered_errors,
            fuzzer_args,
            fork_data,
            test_backtrace,
        }: RunCompleted,
        test_case: &TestCaseWithResolvedConfig,
        contracts_data: &ContractsData,
        versioned_program_path: &Utf8Path,
        backtrace_sources: &BacktraceSources,
        trace_args: &TraceArgs,
        gas_report_enabled: bool,
    ) -> Self {
        let name = test_case.name.clone();
        let trace_components = trace_args.to_components();
        // `ContractsDataStore` is expensive to build and is needed only by gas report and debugging trace.
        // Keep this lazy so normal test runs do not parse and extract contract artifacts for every test case.
        let contracts_data_store = (gas_report_enabled || trace_components.is_some()).then(|| {
            build_contracts_data_store(
                contracts_data,
                fork_data.as_ref(),
                test_case.config.disable_predeployed_contracts,
            )
        });

        let empty = ForkData::default();
        let fork_data_ref = fork_data.as_ref().unwrap_or(&empty);
        let gas_info = SingleTestGasInfo::new(gas_used);
        let gas_info = if gas_report_enabled {
            gas_info.get_with_report_data(
                &call_trace.borrow(),
                contracts_data_store
                    .as_ref()
                    .expect("contracts data store must be initialized for gas report"),
            )
        } else {
            gas_info
        };
        let debugging_trace = trace_components.map(|components| {
            build_debugging_trace(
                &call_trace.borrow(),
                components,
                name.clone(),
                contracts_data_store
                    .expect("contracts data store must be initialized for debugging trace"),
            )
        });

        match status {
            RunStatus::Success(data) => match &test_case.config.expected_result {
                ExpectedTestResult::Success => {
                    let summary = TestCaseSummary::Passed {
                        name,
                        msg: build_readable_text(&data),
                        test_statistics: (),
                        gas_info,
                        used_resources,
                        trace_data: VersionedProfilerCallTrace::V1(build_profiler_call_trace(
                            &call_trace,
                            contracts_data,
                            fork_data_ref,
                            versioned_program_path,
                        )),
                        debugging_trace,
                    };
                    check_available_gas(test_case.config.available_gas, summary)
                }
                ExpectedTestResult::Panics(expected_panic_value) => TestCaseSummary::Failed {
                    name,
                    msg: Some(build_expected_panic_message(expected_panic_value)),
                    fuzzer_args,
                    test_statistics: (),
                    debugging_trace,
                },
            },
            RunStatus::Panic(value) => match &test_case.config.expected_result {
                ExpectedTestResult::Success => TestCaseSummary::Failed {
                    name: name.clone(),
                    msg: build_readable_text(&value).map(|msg| {
                        add_test_backtrace_footer(
                            msg,
                            contracts_data,
                            &encountered_errors,
                            &test_backtrace,
                            &name,
                            backtrace_sources,
                        )
                    }),
                    fuzzer_args,
                    test_statistics: (),
                    debugging_trace,
                },
                ExpectedTestResult::Panics(expected_panic_value) => {
                    let (matching, msg) =
                        check_if_matching_and_get_message(&value, expected_panic_value);
                    if matching {
                        let backtrace_msg = is_backtrace_enabled()
                            .then(|| {
                                get_backtrace(
                                    contracts_data,
                                    &encountered_errors,
                                    test_backtrace.context(),
                                    &name,
                                    backtrace_sources,
                                )
                            })
                            .flatten();

                        TestCaseSummary::Passed {
                            name,
                            msg: backtrace_msg,
                            test_statistics: (),
                            gas_info,
                            used_resources,
                            trace_data: VersionedProfilerCallTrace::V1(build_profiler_call_trace(
                                &call_trace,
                                contracts_data,
                                fork_data_ref,
                                versioned_program_path,
                            )),
                            debugging_trace,
                        }
                    } else {
                        TestCaseSummary::Failed {
                            name: name.clone(),
                            msg: msg.map(|msg| {
                                add_test_backtrace_footer(
                                    msg,
                                    contracts_data,
                                    &encountered_errors,
                                    &test_backtrace,
                                    &name,
                                    backtrace_sources,
                                )
                            }),
                            fuzzer_args,
                            test_statistics: (),
                            debugging_trace,
                        }
                    }
                }
            },
        }
    }
}

fn is_matching_should_panic_data(data: &[Felt], pattern: &[Felt]) -> bool {
    let data_str = convert_felts_to_byte_array_string(data);
    let pattern_str = convert_felts_to_byte_array_string(pattern);

    if let (Some(data), Some(pattern)) = (&data_str, &pattern_str) {
        data.contains(pattern.as_str()) // If both data and pattern are byte arrays, pattern should be a substring of data
    } else {
        // Compare logic depends on the presence of `ENTRYPOINT_FAILED_ERROR` in the expected data.
        // A ByteArray may contain the same felt value as `ENTRYPOINT_FAILED` in its payload,
        // so only non-ByteArray patterns can opt into exact generic-error matching.
        if pattern_str.is_none() && pattern.contains(&ENTRYPOINT_FAILED_ERROR_FELT) {
            // If data includes `ENTRYPOINT_FAILED_ERROR` compare as is.
            data == pattern
        } else {
            // Otherwise, remove propagated generic errors and then compare.
            let filtered = strip_trailing_entrypoint_failed(data);

            if let (Some(data), Some(pattern)) = (
                convert_felts_to_byte_array_string(filtered),
                pattern_str.as_ref(),
            ) {
                return data.contains(pattern);
            }

            filtered == pattern
        }
    }
}

fn strip_trailing_entrypoint_failed(mut data: &[Felt]) -> &[Felt] {
    while data.last() == Some(&ENTRYPOINT_FAILED_ERROR_FELT) {
        data = &data[..data.len() - 1];
    }
    data
}

fn convert_felts_to_byte_array_string(data: &[Felt]) -> Option<String> {
    ByteArray::deserialize_with_magic(data)
        .map(|byte_array| byte_array.to_string())
        .ok()
}

impl AnyTestCaseSummary {
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        match self {
            AnyTestCaseSummary::Fuzzing(case) => case.name(),
            AnyTestCaseSummary::Single(case) => case.name(),
        }
    }

    #[must_use]
    pub fn msg(&self) -> Option<&str> {
        match self {
            AnyTestCaseSummary::Fuzzing(case) => case.msg(),
            AnyTestCaseSummary::Single(case) => case.msg(),
        }
    }

    #[must_use]
    pub fn debugging_trace(&self) -> Option<&debugging::Trace> {
        match self {
            AnyTestCaseSummary::Fuzzing(case) => case.debugging_trace(),
            AnyTestCaseSummary::Single(case) => case.debugging_trace(),
        }
    }

    #[must_use]
    pub fn is_passed(&self) -> bool {
        matches!(
            self,
            AnyTestCaseSummary::Single(TestCaseSummary::Passed { .. })
                | AnyTestCaseSummary::Fuzzing(TestCaseSummary::Passed { .. })
        )
    }

    #[must_use]
    pub fn is_failed(&self) -> bool {
        matches!(
            self,
            AnyTestCaseSummary::Single(TestCaseSummary::Failed { .. })
                | AnyTestCaseSummary::Fuzzing(TestCaseSummary::Failed { .. })
        )
    }

    #[must_use]
    pub fn is_interrupted(&self) -> bool {
        matches!(
            self,
            AnyTestCaseSummary::Single(TestCaseSummary::Interrupted { .. })
                | AnyTestCaseSummary::Fuzzing(TestCaseSummary::Interrupted { .. })
        )
    }

    #[must_use]
    pub fn is_ignored(&self) -> bool {
        matches!(
            self,
            AnyTestCaseSummary::Single(TestCaseSummary::Ignored { .. })
                | AnyTestCaseSummary::Fuzzing(TestCaseSummary::Ignored { .. })
        )
    }

    #[must_use]
    pub fn is_excluded_from_partition(&self) -> bool {
        matches!(
            self,
            AnyTestCaseSummary::Single(TestCaseSummary::ExcludedFromPartition { .. })
                | AnyTestCaseSummary::Fuzzing(TestCaseSummary::ExcludedFromPartition { .. })
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::test_case_summary::*;
    use starknet_api::execution_resources::{GasAmount, GasVector};

    const FLOAT_ERROR: f64 = 0.01;

    #[test]
    fn test_gas_statistics_new() {
        let gas_usages = vec![
            GasVector {
                l1_gas: GasAmount(10),
                l1_data_gas: GasAmount(20),
                l2_gas: GasAmount(30),
            },
            GasVector {
                l1_gas: GasAmount(20),
                l1_data_gas: GasAmount(40),
                l2_gas: GasAmount(60),
            },
            GasVector {
                l1_gas: GasAmount(30),
                l1_data_gas: GasAmount(60),
                l2_gas: GasAmount(90),
            },
        ];

        let stats = GasFuzzingInfo::new(&gas_usages);

        assert_eq!(stats.l1_gas.min, 10);
        assert_eq!(stats.l1_data_gas.min, 20);
        assert_eq!(stats.l2_gas.min, 30);

        assert_eq!(stats.l1_gas.max, 30);
        assert_eq!(stats.l1_data_gas.max, 60);
        assert_eq!(stats.l2_gas.max, 90);

        assert!((stats.l1_gas.mean - 20.0).abs() < FLOAT_ERROR);
        assert!((stats.l1_data_gas.mean - 40.0).abs() < FLOAT_ERROR);
        assert!((stats.l2_gas.mean - 60.0).abs() < FLOAT_ERROR);

        assert!((stats.l1_gas.std_deviation - 8.165).abs() < FLOAT_ERROR);
        assert!((stats.l1_data_gas.std_deviation - 16.33).abs() < FLOAT_ERROR);
        assert!((stats.l2_gas.std_deviation - 24.49).abs() < FLOAT_ERROR);
    }

    #[test]
    fn test_is_matching_should_panic_data_entrypoint_failed() {
        let data = vec![
            Felt::from(11_u8),
            Felt::from(22_u8),
            Felt::from(33_u8),
            ENTRYPOINT_FAILED_ERROR_FELT,
            ENTRYPOINT_FAILED_ERROR_FELT,
        ];

        assert!(is_matching_should_panic_data(&data, &data));

        let non_matching_pattern = vec![
            Felt::from(11_u8),
            Felt::from(22_u8),
            Felt::from(33_u8),
            ENTRYPOINT_FAILED_ERROR_FELT,
        ];
        assert!(!is_matching_should_panic_data(&data, &non_matching_pattern));

        let pattern = vec![Felt::from(11_u8), Felt::from(22_u8), Felt::from(33_u8)];
        assert!(is_matching_should_panic_data(&data, &pattern));

        let non_matching_pattern = vec![Felt::from(11_u8), Felt::from(22_u8)];
        assert!(!is_matching_should_panic_data(&data, &non_matching_pattern));
    }

    #[test]
    fn test_is_matching_should_panic_data_does_not_strip_entrypoint_failed_from_middle() {
        let data = vec![
            Felt::from(11_u8),
            ENTRYPOINT_FAILED_ERROR_FELT,
            Felt::from(22_u8),
        ];
        let pattern = vec![Felt::from(11_u8), Felt::from(22_u8)];

        assert!(!is_matching_should_panic_data(&data, &pattern));
    }

    #[test]
    fn test_is_matching_should_panic_data_mixed_tuple() {
        let byte_array = |s: &str| ByteArray::from(s).serialize_with_magic();

        let mut data = byte_array("this_string_is_longer_than_31_bytes");
        data.push(Felt::from(11_u8));
        data.extend(byte_array("hello"));
        data.push(Felt::from(5_u8));
        data.push(Felt::from_bytes_be_slice(b"short_string"));

        assert!(is_matching_should_panic_data(&data, &data));

        let mut non_matching_pattern = byte_array("this_string_is_longer_than_31_bytes");
        non_matching_pattern.push(Felt::from(11_u8));
        non_matching_pattern.extend(byte_array("world"));
        non_matching_pattern.push(Felt::from(5_u8));
        non_matching_pattern.push(Felt::from_bytes_be_slice(b"short_string"));

        assert!(!is_matching_should_panic_data(&data, &non_matching_pattern));
    }

    #[test]
    fn test_incorrect_panic_data_message_mixed_tuple() {
        let byte_array = |s: &str| ByteArray::from(s).serialize_with_magic();

        let mut actual = byte_array("this_string_is_longer_than_31_bytes");
        actual.push(Felt::from(11_u8));
        actual.extend(byte_array("hello"));
        actual.push(Felt::from(5_u8));
        actual.push(Felt::from_bytes_be_slice(b"short_string"));

        let mut expected = byte_array("this_string_is_longer_than_31_bytes");
        expected.push(Felt::from(11_u8));
        expected.extend(byte_array("helloo"));
        expected.push(Felt::from(5_u8));
        expected.push(Felt::from_bytes_be_slice(b"short_string"));

        let (matching, message) =
            check_if_matching_and_get_message(&actual, &ExpectedPanicValue::Exact(expected));

        assert!(!matching);
        assert_eq!(
            message.unwrap(),
            "\n    Incorrect panic data\
             \n    Actual:    (ByteArray(\"this_string_is_longer_than_31_bytes\"), felt252 0xb, ByteArray(\"hello\"), felt252 0x5, felt252 0x73686f72745f737472696e67 ('short_string'))\
             \n    Expected:  (ByteArray(\"this_string_is_longer_than_31_bytes\"), felt252 0xb, ByteArray(\"helloo\"), felt252 0x5, felt252 0x73686f72745f737472696e67 ('short_string'))\n"
        );
    }

    #[test]
    fn test_incorrect_panic_data_message_malformed_byte_array() {
        // Magic followed by data that is not a valid `ByteArray` serialization
        // must not crash and should fall back to felt-by-felt formatting.
        let magic = ByteArray::from("").serialize_with_magic()[0];
        let actual = vec![
            magic,
            Felt::from(0_u8),
            Felt::from_bytes_be_slice(b"x"),
            Felt::from(100_u8),
        ];
        let expected = vec![Felt::from_bytes_be_slice(b"panic message")];

        let (matching, message) =
            check_if_matching_and_get_message(&actual, &ExpectedPanicValue::Exact(expected));

        assert!(!matching);
        assert_eq!(
            message.unwrap(),
            "\n    Incorrect panic data\
             \n    Actual:    (felt252 0x46a6158a16a947e5916b2a2ca68501a45e93d7110e81aa2d6438b1c57c879a3, felt252 0x0 (''), felt252 0x78 ('x'), felt252 0x64 ('d'))\
             \n    Expected:  felt252 0x70616e6963206d657373616765 ('panic message')\n"
        );
    }

    #[test]
    fn test_incorrect_panic_data_message_treats_valid_magic_sequence_as_byte_array() {
        let actual = ByteArray::from("hello").serialize_with_magic();
        let expected = vec![Felt::from_bytes_be_slice(b"world")];

        let (matching, message) =
            check_if_matching_and_get_message(&actual, &ExpectedPanicValue::Exact(expected));

        assert!(!matching);
        assert_eq!(
            message.unwrap(),
            "\n    Incorrect panic data\
             \n    Actual:    ByteArray(\"hello\")\
             \n    Expected:  felt252 0x776f726c64 ('world')\n"
        );
    }

    #[test]
    fn test_is_matching_should_panic_data_propagated_byte_array_substring() {
        let mut data = ByteArray::from("This will panic for sure").serialize_with_magic();
        data.push(ENTRYPOINT_FAILED_ERROR_FELT);

        let pattern = ByteArray::from("will panic").serialize_with_magic();
        assert!(is_matching_should_panic_data(&data, &pattern));
    }

    #[test]
    fn test_is_matching_should_panic_data_propagated_entrypoint_failed_byte_array() {
        let mut data = ByteArray::from("ENTRYPOINT_FAILED").serialize_with_magic();
        data.push(ENTRYPOINT_FAILED_ERROR_FELT);

        let pattern = ByteArray::from("ENTRYPOINT_FAILED").serialize_with_magic();
        assert!(is_matching_should_panic_data(&data, &pattern));
    }
}
