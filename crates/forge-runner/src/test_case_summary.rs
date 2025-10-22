use crate::backtrace::{add_backtrace_footer, get_backtrace, is_backtrace_enabled};
use crate::build_trace_data::build_profiler_call_trace;
use crate::debugging::{TraceArgs, build_debugging_trace};
use crate::expected_result::{ExpectedPanicValue, ExpectedTestResult};
use crate::gas::check_available_gas;
use crate::gas::report::SingleTestGasInfo;
use crate::gas::stats::GasStats;
use crate::package_tests::with_config_resolved::TestCaseWithResolvedConfig;
use crate::running::{RunCompleted, RunStatus};
use cairo_annotations::trace_data::VersionedCallTrace as VersionedProfilerCallTrace;
use camino::Utf8Path;
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::UsedResources;
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use conversions::byte_array::ByteArray;
use conversions::felt::ToShortString;
use debugging::ContractsDataStore;
use shared::utils::build_readable_text;
use starknet_api::execution_resources::GasVector;
use starknet_types_core::felt::Felt;
use std::fmt;
use std::option::Option;

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
            TestCaseSummary::Interrupted { .. } => None,
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
        }
    }
}

fn build_expected_panic_message(expected_panic_value: &ExpectedPanicValue) -> String {
    match expected_panic_value {
        ExpectedPanicValue::Any => "\n    Expected to panic, but no panic occurred\n".into(),
        ExpectedPanicValue::Exact(panic_data) => {
            let panic_string = join_short_strings(panic_data);

            format!(
                "\n    Expected to panic, but no panic occurred\n    Expected panic data:  {panic_data:?} ({panic_string})\n"
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
        Some(expected) if is_matching(actual_panic_value, expected) => (true, None),
        Some(expected) => {
            let panic_string = convert_felts_to_byte_array_string(actual_panic_value)
                .unwrap_or_else(|| join_short_strings(actual_panic_value));
            let expected_string = convert_felts_to_byte_array_string(expected)
                .unwrap_or_else(|| join_short_strings(expected));

            let message = Some(format!(
                "\n    Incorrect panic data\n    {}\n    {}\n",
                format_args!("Actual:    {actual_panic_value:?} ({panic_string})"),
                format_args!("Expected:  {expected:?} ({expected_string})")
            ));
            (false, message)
        }
        None => (true, None),
    }
}

impl TestCaseSummary<Single> {
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
        }: RunCompleted,
        test_case: &TestCaseWithResolvedConfig,
        contracts_data: &ContractsData,
        versioned_program_path: &Utf8Path,
        trace_args: &TraceArgs,
        gas_report_enabled: bool,
    ) -> Self {
        let name = test_case.name.clone();

        let contracts_data_store = ContractsDataStore::new(contracts_data, &fork_data);
        let debugging_trace = build_debugging_trace(
            &call_trace.borrow(),
            &contracts_data_store,
            trace_args,
            name.clone(),
        );

        let gas_info = SingleTestGasInfo::new(gas_used);
        let gas_info = if gas_report_enabled {
            gas_info.get_with_report_data(
                &call_trace.borrow(),
                &contracts_data_store,
            )
        } else {
            gas_info
        };

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
                            &fork_data,
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
                    name,
                    msg: build_readable_text(&value)
                        .map(|msg| add_backtrace_footer(msg, contracts_data, &encountered_errors)),
                    fuzzer_args,
                    test_statistics: (),
                    debugging_trace,
                },
                ExpectedTestResult::Panics(expected_panic_value) => {
                    let (matching, msg) =
                        check_if_matching_and_get_message(&value, expected_panic_value);
                    if matching {
                        TestCaseSummary::Passed {
                            name,
                            msg: is_backtrace_enabled()
                                .then(|| get_backtrace(contracts_data, &encountered_errors)),
                            test_statistics: (),
                            gas_info,
                            used_resources,
                            trace_data: VersionedProfilerCallTrace::V1(build_profiler_call_trace(
                                &call_trace,
                                contracts_data,
                                &fork_data,
                                versioned_program_path,
                            )),
                            debugging_trace,
                        }
                    } else {
                        TestCaseSummary::Failed {
                            name,
                            msg: msg.map(|msg| {
                                add_backtrace_footer(msg, contracts_data, &encountered_errors)
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

fn join_short_strings(data: &[Felt]) -> String {
    data.iter()
        .map(|felt| felt.to_short_string().unwrap_or_default())
        .collect::<Vec<String>>()
        .join(", ")
}

fn is_matching(data: &[Felt], pattern: &[Felt]) -> bool {
    let data_str = convert_felts_to_byte_array_string(data);
    let pattern_str = convert_felts_to_byte_array_string(pattern);

    if let (Some(data), Some(pattern)) = (data_str, pattern_str) {
        data.contains(&pattern) // If both data and pattern are byte arrays, pattern should be a substring of data
    } else {
        data == pattern // Otherwise, data should be equal to pattern
    }
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
}
