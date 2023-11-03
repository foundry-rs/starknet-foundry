use crate::CrateLocation;
use anyhow::Error;
use console::style;
use forge_runner::test_case_summary::TestCaseSummary;
use forge_runner::test_crate_summary::TestCrateSummary;
use forge_runner::TestResultPrinter;

pub trait TestPrinter {
    fn print_error_message(&self, error: &Error);
    fn print_collected_tests_count(&self, tests_num: usize, package_name: &str);
    fn print_running_tests(&self, test_crate_file: CrateLocation, tests_num: usize);
    fn print_test_summary(&self, summaries: &[TestCrateSummary]);
    fn print_test_seed(&self, seed: u64);
    fn print_failures(&self, all_failed_tests: &[TestCaseSummary]);
}

#[derive(Debug, Clone, Default)]
pub struct PrettyPrinter;

impl TestResultPrinter for PrettyPrinter {
    fn print_test_result(&self, test_result: &TestCaseSummary) {
        let result_header = match test_result {
            TestCaseSummary::Passed { .. } => format!("[{}]", style("PASS").green()),
            TestCaseSummary::Failed { .. } => format!("[{}]", style("FAIL").red()),
            TestCaseSummary::Ignored { .. } => format!("[{}]", style("IGNORE").yellow()),
            TestCaseSummary::Skipped { .. } => format!("[{}]", style("SKIP").color256(11)),
            TestCaseSummary::Interrupted {} => {
                unreachable!()
            }
        };

        let result_name = match test_result {
            TestCaseSummary::Skipped { name }
            | TestCaseSummary::Ignored { name }
            | TestCaseSummary::Failed { name, .. }
            | TestCaseSummary::Passed { name, .. } => name,
            TestCaseSummary::Interrupted {} => {
                unreachable!()
            }
        };

        let result_message = match test_result {
            TestCaseSummary::Passed { msg: Some(msg), .. } => format!("\n\nSuccess data:{msg}"),
            TestCaseSummary::Failed { msg: Some(msg), .. } => format!("\n\nFailure data:{msg}"),
            _ => String::new(),
        };

        let fuzzer_report = match test_result.runs() {
            None => String::new(),
            Some(runs) => {
                if matches!(test_result, TestCaseSummary::Failed { .. }) {
                    let arguments = test_result.arguments();
                    format!(" (fuzzer runs = {runs}, arguments = {arguments:?})")
                } else {
                    format!(" (fuzzer runs = {runs})")
                }
            }
        };

        println!("{result_header} {result_name}{fuzzer_report}{result_message}");
    }
}

impl PrettyPrinter {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl TestPrinter for PrettyPrinter {
    fn print_error_message(&self, error: &Error) {
        let error_tag = style("ERROR").red();
        println!("[{error_tag}] {error}");
    }
    fn print_collected_tests_count(&self, tests_num: usize, package_name: &str) {
        let plain_text = format!("\n\nCollected {tests_num} test(s) from {package_name} package");
        println!("{}", style(plain_text).bold());
    }
    fn print_running_tests(&self, test_crate_file: CrateLocation, tests_num: usize) {
        let dir_name = match test_crate_file {
            CrateLocation::Lib => "src",
            CrateLocation::Tests => "tests",
        };
        let plain_text = format!("Running {tests_num} test(s) from {dir_name}/");

        println!("{}", style(plain_text).bold());
    }
    fn print_test_summary(&self, summaries: &[TestCrateSummary]) {
        let passed: usize = summaries.iter().map(TestCrateSummary::count_passed).sum();
        let skipped: usize = summaries.iter().map(TestCrateSummary::count_skipped).sum();
        let failed: usize = summaries.iter().map(TestCrateSummary::count_failed).sum();

        println!(
            "{}: {} passed, {} failed, {} skipped",
            style("Tests").bold(),
            passed,
            failed,
            skipped,
        );
    }
    fn print_test_seed(&self, seed: u64) {
        println!("{}: {seed}", style("Fuzzer seed").bold());
    }
    fn print_failures(&self, all_failed_tests: &[TestCaseSummary]) {
        if all_failed_tests.is_empty() {
            return;
        }

        let failed_tests_names: Vec<&String> = all_failed_tests
            .iter()
            .map(|test_case_summary| match test_case_summary {
                TestCaseSummary::Failed { name, .. } => name,
                TestCaseSummary::Passed { .. }
                | TestCaseSummary::Ignored { .. }
                | TestCaseSummary::Skipped { .. }
                | TestCaseSummary::Interrupted {} => unreachable!(),
            })
            .collect();

        println!("\nFailures:");
        for name in failed_tests_names {
            println!("    {name}");
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct NullPrinter;

impl NullPrinter {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl TestPrinter for NullPrinter {
    fn print_error_message(&self, _error: &Error) {}

    fn print_collected_tests_count(&self, _tests_num: usize, _package_name: &str) {}

    fn print_running_tests(&self, _test_crate_file: CrateLocation, _tests_num: usize) {}

    fn print_test_summary(&self, _summaries: &[TestCrateSummary]) {}

    fn print_test_seed(&self, _seed: u64) {}

    fn print_failures(&self, _all_failed_tests: &[TestCaseSummary]) {}
}

impl TestResultPrinter for NullPrinter {
    fn print_test_result(&self, _test_result: &TestCaseSummary) {}
}
