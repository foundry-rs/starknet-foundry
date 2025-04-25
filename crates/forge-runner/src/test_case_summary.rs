use crate::backtrace::add_backtrace_footer;
use crate::build_trace_data::build_profiler_call_trace;
use crate::expected_result::{ExpectedPanicValue, ExpectedTestResult};
use crate::gas::check_available_gas;
use crate::package_tests::with_config_resolved::TestCaseWithResolvedConfig;
use cairo_annotations::trace_data::VersionedCallTrace as VersionedProfilerCallTrace;
use cairo_lang_runner::short_string::as_cairo_short_string;
use cairo_lang_runner::{RunResult, RunResultValue};
use camino::Utf8Path;
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::UsedResources;
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use cheatnet::state::{CallTrace as InternalCallTrace, EncounteredErrors};
use conversions::byte_array::ByteArray;
use num_traits::Pow;
use shared::utils::build_readable_text;
use starknet_api::execution_resources::GasVector;
use starknet_types_core::felt::Felt;
use std::cell::RefCell;
use std::fmt;
use std::option::Option;
use std::rc::Rc;

#[derive(Debug, PartialEq, Clone, Default)]
pub struct GasStatistics {
    pub l1_gas: GasStatisticsComponent,
    pub l1_data_gas: GasStatisticsComponent,
    pub l2_gas: GasStatisticsComponent,
}

impl fmt::Display for GasStatistics {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "(l1_gas: {}, l1_data_gas: {}, l2_gas: {})",
            self.l1_gas, self.l1_data_gas, self.l2_gas
        )
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct GasStatisticsComponent {
    pub min: u64,
    pub max: u64,
    pub mean: f64,
    pub std_deviation: f64,
}

impl GasStatisticsComponent {
    #[must_use]
    pub fn new(gas_usages: &[u64]) -> Self {
        let mean = GasStatistics::mean(gas_usages);
        Self {
            min: *gas_usages.iter().min().unwrap(),
            max: *gas_usages.iter().max().unwrap(),
            mean,
            std_deviation: GasStatistics::std_deviation(mean, gas_usages),
        }
    }
}

impl fmt::Display for GasStatisticsComponent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{{max: ~{}, min: ~{}, mean: ~{:.2}, std deviation: ~{:.2}}}",
            self.max, self.min, self.mean, self.std_deviation
        )
    }
}

impl GasStatistics {
    #[must_use]
    pub fn new(gas_usages: &[GasVector]) -> Self {
        let l1_gas_values: Vec<u64> = gas_usages.iter().map(|gv| gv.l1_gas.0).collect();
        let l1_data_gas_values: Vec<u64> = gas_usages.iter().map(|gv| gv.l1_data_gas.0).collect();
        let l2_gas_values: Vec<u64> = gas_usages.iter().map(|gv| gv.l2_gas.0).collect();

        GasStatistics {
            l1_gas: { GasStatisticsComponent::new(l1_gas_values.as_ref()) },
            l1_data_gas: { GasStatisticsComponent::new(l1_data_gas_values.as_ref()) },
            l2_gas: { GasStatisticsComponent::new(l2_gas_values.as_ref()) },
        }
    }

    #[allow(clippy::cast_precision_loss)]
    fn mean(gas_usages: &[u64]) -> f64 {
        let sum: f64 = gas_usages.iter().map(|&x| x as f64).sum();
        sum / gas_usages.len() as f64
    }

    #[allow(clippy::cast_precision_loss)]
    fn std_deviation(mean: f64, gas_usages: &[u64]) -> f64 {
        let sum_squared_diff = gas_usages
            .iter()
            .map(|&x| (x as f64 - mean).pow(2))
            .sum::<f64>();

        (sum_squared_diff / gas_usages.len() as f64).sqrt()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct FuzzingStatistics {
    pub runs: usize,
}

pub trait TestType {
    type GasInfo: std::fmt::Debug + Clone;
    type TestStatistics: std::fmt::Debug + Clone;
    type TraceData: std::fmt::Debug + Clone;
}

#[derive(Debug, PartialEq, Clone)]
pub struct Fuzzing;
impl TestType for Fuzzing {
    type GasInfo = GasStatistics;
    type TestStatistics = FuzzingStatistics;
    type TraceData = ();
}

#[derive(Debug, PartialEq, Clone)]
pub struct Single;
impl TestType for Single {
    type GasInfo = GasVector;
    type TestStatistics = ();
    type TraceData = VersionedProfilerCallTrace;
}

/// Summary of running a single test case
#[derive(Debug, Clone)]
pub enum TestCaseSummary<T: TestType> {
    /// Test case passed
    Passed {
        /// Name of the test case
        name: String,
        /// Message to be printed after the test case run
        msg: Option<String>,
        /// Arguments used in the test case run
        arguments: Vec<Felt>,
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
        /// Arguments used in the test case run
        arguments: Vec<Felt>,
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
    Skipped {},
}

#[derive(Debug)]
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
            TestCaseSummary::Skipped { .. } => None,
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
                arguments,
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
                        TestCaseSummary::Passed { gas_info, .. } => gas_info,
                        _ => unreachable!(),
                    })
                    .collect();

                TestCaseSummary::Passed {
                    name,
                    msg,
                    arguments,
                    gas_info: GasStatistics::new(gas_usages.as_ref()),
                    used_resources: UsedResources::default(),
                    test_statistics: FuzzingStatistics { runs },
                    trace_data: (),
                    debugging_trace,
                }
            }
            TestCaseSummary::Failed {
                name,
                msg,
                arguments,
                fuzzer_args,
                debugging_trace,
                test_statistics: (),
            } => TestCaseSummary::Failed {
                name,
                msg,
                arguments,
                fuzzer_args,
                test_statistics: FuzzingStatistics {
                    runs: results.len(),
                },
                debugging_trace,
            },
            TestCaseSummary::Ignored { name } => TestCaseSummary::Ignored { name: name.clone() },
            TestCaseSummary::Skipped {} => TestCaseSummary::Skipped {},
        }
    }
}

impl TestCaseSummary<Single> {
    #[must_use]
    #[expect(clippy::too_many_arguments)]
    pub(crate) fn from_run_result_and_info(
        run_result: RunResult,
        test_case: &TestCaseWithResolvedConfig,
        arguments: Vec<Felt>,
        fuzzer_args: Vec<String>,
        gas: GasVector,
        used_resources: UsedResources,
        call_trace: &Rc<RefCell<InternalCallTrace>>,
        encountered_errors: &EncounteredErrors,
        contracts_data: &ContractsData,
        versioned_program_path: &Utf8Path,
    ) -> Self {
        let name = test_case.name.clone();
        let msg = extract_result_data(&run_result, &test_case.config.expected_result)
            .map(|msg| add_backtrace_footer(msg, contracts_data, encountered_errors));

        let debugging_trace = cfg!(feature = "debugging")
            .then(|| debugging::Trace::new(&call_trace.borrow(), contracts_data, name.clone()));

        match run_result.value {
            RunResultValue::Success(_) => match &test_case.config.expected_result {
                ExpectedTestResult::Success => {
                    let summary = TestCaseSummary::Passed {
                        name,
                        msg,
                        arguments,
                        test_statistics: (),
                        gas_info: gas,
                        used_resources,
                        trace_data: VersionedProfilerCallTrace::V1(build_profiler_call_trace(
                            call_trace,
                            contracts_data,
                            versioned_program_path,
                        )),
                        debugging_trace,
                    };
                    check_available_gas(test_case.config.available_gas, summary)
                }
                ExpectedTestResult::Panics(_) => TestCaseSummary::Failed {
                    name,
                    msg,
                    arguments,
                    fuzzer_args,
                    test_statistics: (),
                    debugging_trace,
                },
            },
            RunResultValue::Panic(value) => match &test_case.config.expected_result {
                ExpectedTestResult::Success => TestCaseSummary::Failed {
                    name,
                    msg,
                    arguments,
                    fuzzer_args,
                    test_statistics: (),
                    debugging_trace,
                },
                ExpectedTestResult::Panics(panic_expectation) => match panic_expectation {
                    ExpectedPanicValue::Exact(expected) if !is_matching(&value, expected) => {
                        TestCaseSummary::Failed {
                            name,
                            msg,
                            arguments,
                            fuzzer_args,
                            test_statistics: (),
                            debugging_trace,
                        }
                    }
                    _ => TestCaseSummary::Passed {
                        name,
                        msg,
                        arguments,
                        test_statistics: (),
                        gas_info: gas,
                        used_resources,
                        trace_data: VersionedProfilerCallTrace::V1(build_profiler_call_trace(
                            call_trace,
                            contracts_data,
                            versioned_program_path,
                        )),
                        debugging_trace,
                    },
                },
            },
        }
    }
}

fn join_short_strings(data: &[Felt]) -> String {
    data.iter()
        .map(|felt| as_cairo_short_string(felt).unwrap_or_default())
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

/// Returns a string with the data that was produced by the test case.
/// If the test was expected to fail with specific data e.g. `#[should_panic(expected: ('data',))]`
/// and failed to do so, it returns a string comparing the panic data and the expected data.
#[must_use]
fn extract_result_data(run_result: &RunResult, expectation: &ExpectedTestResult) -> Option<String> {
    match &run_result.value {
        RunResultValue::Success(data) => match expectation {
            ExpectedTestResult::Panics(panic_expectation) => match panic_expectation {
                ExpectedPanicValue::Exact(panic_data) => {
                    let panic_string = join_short_strings(panic_data);

                    Some(format!(
                        "\n    Expected to panic but didn't\n    Expected panic data:  {panic_data:?} ({panic_string})\n"
                    ))
                }
                ExpectedPanicValue::Any => Some("\n    Expected to panic but didn't\n".into()),
            },
            ExpectedTestResult::Success => build_readable_text(data),
        },
        RunResultValue::Panic(panic_data) => {
            let expected_data = match expectation {
                ExpectedTestResult::Panics(panic_expectation) => match panic_expectation {
                    ExpectedPanicValue::Exact(data) => Some(data),
                    ExpectedPanicValue::Any => None,
                },
                ExpectedTestResult::Success => None,
            };

            match expected_data {
                Some(expected) if is_matching(panic_data, expected) => {
                    build_readable_text(panic_data)
                }
                Some(expected) => {
                    let panic_string = convert_felts_to_byte_array_string(panic_data)
                        .unwrap_or_else(|| join_short_strings(panic_data));
                    let expected_string = convert_felts_to_byte_array_string(expected)
                        .unwrap_or_else(|| join_short_strings(expected));

                    Some(format!(
                        "\n    Incorrect panic data\n    {}\n    {}\n",
                        format_args!("Actual:    {panic_data:?} ({panic_string})"),
                        format_args!("Expected:  {expected:?} ({expected_string})")
                    ))
                }
                None => build_readable_text(panic_data),
            }
        }
    }
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
    pub fn is_skipped(&self) -> bool {
        matches!(
            self,
            AnyTestCaseSummary::Single(TestCaseSummary::Skipped { .. })
                | AnyTestCaseSummary::Fuzzing(TestCaseSummary::Skipped { .. })
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
    use super::*;
    use starknet_api::execution_resources::GasAmount;

    const FLOAT_ERROR: f64 = 0.01;

    #[test]
    fn test_mean_basic() {
        let data = [1, 2, 3, 4, 5];
        let result = GasStatistics::mean(&data);
        assert!((result - 3.0).abs() < FLOAT_ERROR);
    }

    #[test]
    fn test_mean_single_element() {
        let data = [42];
        let result = GasStatistics::mean(&data);
        assert!((result - 42.0).abs() < FLOAT_ERROR);
    }

    #[test]
    fn test_std_deviation_basic() {
        let data = [1, 2, 3, 4, 5];
        let mean_value = GasStatistics::mean(&data);
        let result = GasStatistics::std_deviation(mean_value, &data);
        assert!((result - 1.414).abs() < FLOAT_ERROR);
    }

    #[test]
    fn test_std_deviation_single_element() {
        let data = [10];
        let mean_value = GasStatistics::mean(&data);
        let result = GasStatistics::std_deviation(mean_value, &data);
        assert!(result.abs() < FLOAT_ERROR);
    }

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

        let stats = GasStatistics::new(&gas_usages);

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
