use crate::compiled_runnable::TestCaseRunnable;
use crate::expected_result::{ExpectedPanicValue, ExpectedTestResult};
use cairo_felt::Felt252;
use cairo_lang_runner::short_string::as_cairo_short_string;
use cairo_lang_runner::{RunResult, RunResultValue};
use num_integer::Roots;
use std::option::Option;

#[derive(Debug, PartialEq, Clone, Default)]
pub struct GasStatistics {
    pub min: u128,
    pub max: u128,
    pub mean: Option<u128>,
    pub std_deviation: Option<u128>,
}

impl GasStatistics {
    #[must_use]
    pub fn new(gas_usages: &[u128]) -> Self {
        let mean = GasStatistics::mean(gas_usages);
        let std_deviation = GasStatistics::std_deviation(mean, gas_usages);

        GasStatistics {
            min: gas_usages.iter().min().copied().unwrap(),
            max: gas_usages.iter().max().copied().unwrap(),
            mean,
            std_deviation,
        }
    }

    fn mean(gas_usages: &[u128]) -> Option<u128> {
        let n = gas_usages.len() as u128;
        if n > 1 {
            Some(gas_usages.iter().sum::<u128>() / n)
        } else {
            None
        }
    }

    fn std_deviation(mean: Option<u128>, gas_usages: &[u128]) -> Option<u128> {
        if let Some(mean) = mean {
            let sum_squared_diff = gas_usages
                .iter()
                .map(|&x| {
                    let abs_difference = if x > mean { x - mean } else { mean - x };
                    abs_difference.pow(2)
                })
                .sum::<u128>();

            Some((sum_squared_diff / gas_usages.len() as u128).sqrt())
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct FuzzingStatistics {
    pub runs: usize,
}

pub trait TestType {
    type GasInfo: std::fmt::Debug + Clone;
    type TestStatistics: std::fmt::Debug + Clone;
}

#[derive(Debug, PartialEq, Clone)]
pub struct Fuzzing;
impl TestType for Fuzzing {
    type GasInfo = GasStatistics;
    type TestStatistics = FuzzingStatistics;
}

#[derive(Debug, PartialEq, Clone)]
pub struct Single;
impl TestType for Single {
    type GasInfo = u128;
    type TestStatistics = ();
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
        arguments: Vec<Felt252>,
        /// Information on used gas
        gas_info: <T as TestType>::GasInfo,
        /// Statistics of the test run
        test_statistics: <T as TestType>::TestStatistics,
    },
    /// Test case failed
    Failed {
        /// Name of the test case
        name: String,
        /// Message returned by the test case run
        msg: Option<String>,
        /// Arguments used in the test case run
        arguments: Vec<Felt252>,
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
pub enum AnyTestCaseSummary {
    Fuzzing(TestCaseSummary<Fuzzing>),
    Single(TestCaseSummary<Single>),
}

impl<T: TestType> TestCaseSummary<T> {
    #[must_use]
    pub fn name(&self) -> Option<&String> {
        match self {
            TestCaseSummary::Failed { name, .. }
            | TestCaseSummary::Passed { name, .. }
            | TestCaseSummary::Ignored { name, .. } => Some(name),
            TestCaseSummary::Skipped { .. } => None,
        }
    }

    #[must_use]
    pub fn msg(&self) -> Option<&String> {
        match self {
            TestCaseSummary::Failed { msg: Some(msg), .. }
            | TestCaseSummary::Passed { msg: Some(msg), .. } => Some(msg),
            _ => None,
        }
    }

    #[must_use]
    pub fn arguments(&self) -> Vec<Felt252> {
        match self {
            TestCaseSummary::Failed { arguments, .. }
            | TestCaseSummary::Passed { arguments, .. } => arguments.clone(),
            TestCaseSummary::Ignored { .. } | TestCaseSummary::Skipped { .. } => vec![],
        }
    }
}

impl TestCaseSummary<Fuzzing> {
    #[must_use]
    pub fn runs(&self) -> Option<usize> {
        match self {
            TestCaseSummary::Passed {
                test_statistics: FuzzingStatistics { runs },
                ..
            }
            | TestCaseSummary::Failed {
                test_statistics: FuzzingStatistics { runs },
                ..
            } => Some(*runs),
            _ => None,
        }
    }
}

impl TestCaseSummary<Single> {
    #[must_use]
    pub(crate) fn from_run_result_and_info(
        run_result: RunResult,
        test_case: &TestCaseRunnable,
        arguments: Vec<Felt252>,
        gas: u128,
    ) -> Self {
        let name = test_case.name.to_string();
        let msg = extract_result_data(&run_result, &test_case.expected_result);
        match run_result.value {
            RunResultValue::Success(_) => match &test_case.expected_result {
                ExpectedTestResult::Success => TestCaseSummary::Passed {
                    name,
                    msg,
                    arguments,
                    test_statistics: (),
                    gas_info: gas,
                },
                ExpectedTestResult::Panics(_) => TestCaseSummary::Failed {
                    name,
                    msg,
                    arguments,
                    test_statistics: (),
                },
            },
            RunResultValue::Panic(value) => match &test_case.expected_result {
                ExpectedTestResult::Success => TestCaseSummary::Failed {
                    name,
                    msg,
                    arguments,
                    test_statistics: (),
                },
                ExpectedTestResult::Panics(panic_expectation) => match panic_expectation {
                    ExpectedPanicValue::Exact(expected) if &value != expected => {
                        TestCaseSummary::Failed {
                            name,
                            msg,
                            arguments,
                            test_statistics: (),
                        }
                    }
                    _ => TestCaseSummary::Passed {
                        name,
                        msg,
                        arguments,
                        test_statistics: (),
                        gas_info: gas,
                    },
                },
            },
        }
    }
}

/// Helper function to build `readable_text` from a run data.
fn build_readable_text(data: &Vec<Felt252>) -> Option<String> {
    let mut readable_text = String::new();

    for felt in data {
        readable_text.push_str(&format!("\n    original value: [{felt}]"));
        if let Some(short_string) = as_cairo_short_string(felt) {
            readable_text.push_str(&format!(", converted to a string: [{short_string}]"));
        }
    }

    if readable_text.is_empty() {
        None
    } else {
        readable_text.push('\n');
        Some(readable_text)
    }
}

/// Returns a string with the data that was produced by the test case.
/// If the test was expected to fail with specific data e.g. `#[should_panic(expected: ('data',))]`
/// and failed to do so, it returns a string comparing the panic data and the expected data.
#[must_use]
fn extract_result_data(run_result: &RunResult, expectation: &ExpectedTestResult) -> Option<String> {
    match &run_result.value {
        RunResultValue::Success(data) => build_readable_text(data),
        RunResultValue::Panic(panic_data) => {
            let expected_data = match expectation {
                ExpectedTestResult::Panics(panic_expectation) => match panic_expectation {
                    ExpectedPanicValue::Exact(data) => Some(data),
                    ExpectedPanicValue::Any => None,
                },
                ExpectedTestResult::Success => None,
            };

            let panic_string: String = panic_data
                .iter()
                .map(|felt| as_cairo_short_string(felt).unwrap_or_default())
                .collect::<Vec<String>>()
                .join(", ");

            match expected_data {
                Some(expected) if expected == panic_data => None,
                Some(expected) => {
                    let expected_string = expected
                        .iter()
                        .map(|felt| as_cairo_short_string(felt).unwrap_or_default())
                        .collect::<Vec<String>>()
                        .join(", ");

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
    pub fn name(&self) -> Option<&String> {
        match self {
            AnyTestCaseSummary::Fuzzing(case) => case.name(),
            AnyTestCaseSummary::Single(case) => case.name(),
        }
    }

    #[must_use]
    pub fn msg(&self) -> Option<&String> {
        match self {
            AnyTestCaseSummary::Fuzzing(case) => case.msg(),
            AnyTestCaseSummary::Single(case) => case.msg(),
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
