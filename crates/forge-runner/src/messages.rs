use crate::forge_config::ForgeTrackedResource;
use crate::gas::resources::GasCalculationResources;
use crate::test_case_summary::{AnyTestCaseSummary, FuzzingStatistics, TestCaseSummary};
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::UsedResources;
use console::style;
use foundry_ui::Message;
use serde::Serialize;
use serde_json::{Value, json};

#[derive(Serialize)]
enum TestResultStatus {
    Passed,
    Failed,
    Ignored,
    Interrupted,
    ExcludedFromPartition,
}

impl From<&AnyTestCaseSummary> for TestResultStatus {
    fn from(test_result: &AnyTestCaseSummary) -> Self {
        match test_result {
            AnyTestCaseSummary::Single(TestCaseSummary::Passed { .. })
            | AnyTestCaseSummary::Fuzzing(TestCaseSummary::Passed { .. }) => Self::Passed,
            AnyTestCaseSummary::Single(TestCaseSummary::Failed { .. })
            | AnyTestCaseSummary::Fuzzing(TestCaseSummary::Failed { .. }) => Self::Failed,
            AnyTestCaseSummary::Single(TestCaseSummary::Ignored { .. })
            | AnyTestCaseSummary::Fuzzing(TestCaseSummary::Ignored { .. }) => Self::Ignored,
            AnyTestCaseSummary::Single(TestCaseSummary::Interrupted { .. })
            | AnyTestCaseSummary::Fuzzing(TestCaseSummary::Interrupted { .. }) => Self::Interrupted,
            AnyTestCaseSummary::Single(TestCaseSummary::ExcludedFromPartition { .. })
            | AnyTestCaseSummary::Fuzzing(TestCaseSummary::ExcludedFromPartition { .. }) => {
                Self::ExcludedFromPartition
            }
        }
    }
}

#[derive(Serialize)]
pub struct TestResultMessage {
    status: TestResultStatus,
    name: String,
    msg: Option<String>,
    debugging_trace: String,
    fuzzer_report: String,
    gas_usage: String,
    used_resources: String,
    gas_report: String,
}

impl TestResultMessage {
    pub fn new(
        test_result: &AnyTestCaseSummary,
        show_detailed_resources: bool,
        tracked_resource: ForgeTrackedResource,
    ) -> Self {
        let name = test_result
            .name()
            .expect("Test result must have a name")
            .to_string();

        let debugging_trace = test_result
            .debugging_trace()
            .map(|trace| format!("\n{trace}"))
            .unwrap_or_default();

        let fuzzer_report = if let AnyTestCaseSummary::Fuzzing(test_result) = test_result {
            match test_result {
                TestCaseSummary::Passed {
                    test_statistics: FuzzingStatistics { runs },
                    gas_info,
                    ..
                } => format!(" (runs: {runs}, {gas_info})"),
                TestCaseSummary::Failed {
                    fuzzer_args,
                    test_statistics: FuzzingStatistics { runs },
                    ..
                } => format!(" (runs: {runs}, arguments: {fuzzer_args:?})"),
                _ => String::new(),
            }
        } else {
            String::new()
        };

        let (gas_usage, gas_report) = match test_result {
            AnyTestCaseSummary::Single(TestCaseSummary::Passed { gas_info, .. }) => {
                let gas_report = gas_info
                    .report_data
                    .as_ref()
                    .map(std::string::ToString::to_string)
                    .unwrap_or_default();

                (
                    format!(
                        " (l1_gas: ~{}, l1_data_gas: ~{}, l2_gas: ~{})",
                        gas_info.gas_used.l1_gas,
                        gas_info.gas_used.l1_data_gas,
                        gas_info.gas_used.l2_gas
                    ),
                    gas_report,
                )
            }
            _ => (String::new(), String::new()),
        };

        let used_resources = match (show_detailed_resources, &test_result) {
            (true, AnyTestCaseSummary::Single(TestCaseSummary::Passed { used_resources, .. })) => {
                format_detailed_resources(used_resources, tracked_resource)
            }
            _ => String::new(),
        };

        let msg = test_result.msg().map(std::string::ToString::to_string);
        let status = TestResultStatus::from(test_result);
        Self {
            status,
            name,
            msg,
            debugging_trace,
            fuzzer_report,
            gas_usage,
            used_resources,
            gas_report,
        }
    }

    fn result_message(&self) -> String {
        if let Some(msg) = &self.msg {
            match self.status {
                TestResultStatus::Passed => return format!("\n\n{msg}"),
                TestResultStatus::Failed => return format!("\n\nFailure data:{msg}"),
                TestResultStatus::Ignored
                | TestResultStatus::Interrupted
                | TestResultStatus::ExcludedFromPartition => return String::new(),
            }
        }
        String::new()
    }

    fn result_header(&self) -> String {
        match self.status {
            TestResultStatus::Passed => format!("[{}]", style("PASS").green()),
            TestResultStatus::Failed => format!("[{}]", style("FAIL").red()),
            TestResultStatus::Ignored => format!("[{}]", style("IGNORE").yellow()),
            TestResultStatus::Interrupted => {
                unreachable!("Interrupted tests should not have visible message representation")
            }
            TestResultStatus::ExcludedFromPartition => {
                unreachable!(
                    "Tests excluded from partition should not have visible message representation"
                )
            }
        }
    }
}

impl Message for TestResultMessage {
    fn text(&self) -> String {
        let result_name = &self.name;
        let result_header = self.result_header();

        let result_msg = self.result_message();
        let result_debug_trace = &self.debugging_trace;

        let fuzzer_report = &self.fuzzer_report;
        let gas_usage = &self.gas_usage;
        let gas_report = &self.gas_report;
        let used_resources = &self.used_resources;

        format!(
            "{result_header} {result_name}{fuzzer_report}{gas_usage}{used_resources}{result_msg}{result_debug_trace}{gas_report}"
        )
    }

    fn json(&self) -> Value {
        json!(self)
    }
}

fn format_detailed_resources(
    used_resources: &UsedResources,
    tracked_resource: ForgeTrackedResource,
) -> String {
    GasCalculationResources::from_used_resources(used_resources)
        .format_for_display(tracked_resource)
}
