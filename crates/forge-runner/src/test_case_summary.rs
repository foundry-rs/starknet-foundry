use crate::running::ForkInfo;
use crate::TestCaseRunnable;
use cairo_felt::Felt252;
use cairo_lang_runner::short_string::as_cairo_short_string;
use cairo_lang_runner::{RunResult, RunResultValue};
use starknet_api::block::BlockNumber;
use std::option::Option;
use test_collector::{ExpectedPanicValue, ExpectedTestResult};

#[derive(Debug, PartialEq, Clone)]
pub struct GasStatistics {
    pub min: f64,
    pub max: f64,
}
#[derive(Debug, PartialEq, Clone)]
pub struct FuzzingStatistics {
    pub runs: usize,
}

pub trait TestType: GasInfo + TestStatistics {}

#[derive(Debug, PartialEq, Clone)]
pub struct Fuzzing;
impl TestType for Fuzzing {}

#[derive(Debug, PartialEq, Clone)]
pub struct Single;
impl TestType for Single {}

pub trait GasInfo {
    type Type: std::fmt::Debug + Clone;
}
impl GasInfo for Fuzzing {
    type Type = GasStatistics;
}
impl GasInfo for Single {
    type Type = f64;
}

pub trait TestStatistics {
    type Type: std::fmt::Debug + Clone;
}
impl TestStatistics for Fuzzing {
    type Type = FuzzingStatistics;
}
impl TestStatistics for Single {
    type Type = ();
}


/// Summary of running a single test case
#[derive(Debug, Clone)]
pub enum TestCaseSummary<T : TestType> {
    /// Test case passed
    Passed {
        /// Name of the test case
        name: String,
        /// Message to be printed after the test case run
        msg: Option<String>,
        /// Arguments used in the test case run
        arguments: Vec<Felt252>,
        /// Information on used gas
        gas_info: <T as GasInfo>::Type,
        /// Statistics of the test run
        test_statistics: <T as TestStatistics>::Type,
        /// Number of block used if BlockId::Tag(Latest) was specified
        latest_block_number: Option<BlockNumber>,

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
        test_statistics: <T as TestStatistics>::Type,
        /// Number of block used if BlockId::Tag(Latest) was specified
        latest_block_number: Option<BlockNumber>,
    },
    /// Test case ignored due to `#[ignored]` attribute or `--ignored` flag
    Ignored {
        /// Name of the test case
        name: String,
    },
    /// Test case skipped due to exit first or execution interrupted, test result is ignored.
    Skipped {},
}

impl<T: TestType> TestCaseSummary<T> {
    #[must_use]
    pub fn arguments(&self) -> Vec<Felt252> {
        match self {
            TestCaseSummary::Failed { arguments, .. }
            | TestCaseSummary::Passed { arguments, .. } => arguments.clone(),
            TestCaseSummary::Ignored { .. } | TestCaseSummary::Skipped { .. } => vec![],
        }
    }


    pub(crate) fn latest_block_number(&self) -> &Option<BlockNumber> {
        match self {
            TestCaseSummary::Failed {
                latest_block_number,
                ..
            }
            | TestCaseSummary::Passed {
                latest_block_number,
                ..
            } => latest_block_number,
            TestCaseSummary::Ignored { .. } | TestCaseSummary::Skipped { .. } => &None,
        }
    }
}

impl TestCaseSummary<Fuzzing> {
    #[must_use]
    pub fn gas_usage(&self) -> Option<String> {
        match self {
            TestCaseSummary::Passed {
                gas_info,
                ..
            } => Some(format!("(max: ~{}, min: ~{})", gas_info.max, gas_info.min)),
            _ => None,
        }
    }

    #[must_use]
    pub fn runs(&self) -> Option<usize> {
        match self {
            TestCaseSummary::Passed { test_statistics: FuzzingStatistics { runs }, .. } => Some(runs.clone()),
            TestCaseSummary::Failed { test_statistics: FuzzingStatistics { runs }, .. } =>  Some(runs.clone()),
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
        fork_info: &ForkInfo,
        gas: f64,
    ) -> Self {
        let name = test_case.name.to_string();
        let msg = extract_result_data(&run_result, &test_case.expected_result);
        let latest_block_number = fork_info.latest_block_number;
        match run_result.value {
            RunResultValue::Success(_) => match &test_case.expected_result {
                ExpectedTestResult::Success => TestCaseSummary::Passed {
                    name,
                    msg,
                    arguments,
                    test_statistics: (),
                    latest_block_number,
                    gas_info: gas,
                },
                ExpectedTestResult::Panics(_) => TestCaseSummary::Failed {
                    name,
                    msg,
                    arguments,
                    test_statistics: (),
                    latest_block_number,
                },
            },
            RunResultValue::Panic(value) => match &test_case.expected_result {
                ExpectedTestResult::Success => TestCaseSummary::Failed {
                    name,
                    msg,
                    arguments,
                    test_statistics: (),
                    latest_block_number,
                },
                ExpectedTestResult::Panics(panic_expectation) => match panic_expectation {
                    ExpectedPanicValue::Exact(expected) if &value != expected => {
                        TestCaseSummary::Failed {
                            name,
                            msg,
                            arguments,
                            test_statistics: (),
                            latest_block_number,
                        }
                    }
                    _ => TestCaseSummary::Passed {
                        name,
                        msg,
                        arguments,
                        test_statistics: (),
                        latest_block_number,
                        gas_info: gas,
                    },
                },
            },
        }
    }

    #[must_use]
    pub fn gas_usage(&self) -> Option<String> {
        match self {
            TestCaseSummary::Passed {
                gas_info,
                ..
            } => Some(format!("~{gas_info}")),
            _ => None,
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
