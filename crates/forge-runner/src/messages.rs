use std::fmt::Display;

use crate::forge_config::ForgeTrackedResource;
use crate::gas::resources::GasCalculationResources;
use crate::test_case_summary::{
    AnyTestCaseSummary, FuzzingStatistics, GasFuzzingInfo, TestCaseSummary,
};
use console::style;
use debugging::Trace;
use foundry_ui::Message;
use serde::Serialize;
use serde_json::{Value, json};
use starknet_api::execution_resources::GasVector;

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    msg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    debugging_trace: Option<Trace>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fuzzer_report: Option<FuzzerReport>,
    gas_usage: GasVector,
    #[serde(skip_serializing_if = "Option::is_none")]
    used_resources: Option<GasCalculationResources>,
    #[serde(skip_serializing_if = "Option::is_none")]
    gas_report: Option<String>,

    #[serde(skip)]
    tracked_resource: ForgeTrackedResource,
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

        let debugging_trace = test_result.debugging_trace().cloned();

        let fuzzer_report = if let AnyTestCaseSummary::Fuzzing(test_result) = test_result {
            match test_result {
                TestCaseSummary::Passed {
                    test_statistics: FuzzingStatistics { runs },
                    gas_info,
                    ..
                } => Some(FuzzerReport::Passed {
                    runs: *runs,
                    gas_info: gas_info.clone(),
                }),
                TestCaseSummary::Failed {
                    fuzzer_args,
                    test_statistics: FuzzingStatistics { runs },
                    ..
                } => Some(FuzzerReport::Failed {
                    runs: *runs,
                    args: fuzzer_args.clone(),
                }),
                _ => None,
            }
        } else {
            None
        };

        let (gas_usage, gas_report) = match test_result {
            AnyTestCaseSummary::Single(TestCaseSummary::Passed { gas_info, .. }) => {
                let gas_report = gas_info
                    .report_data
                    .as_ref()
                    .map(std::string::ToString::to_string);

                (gas_info.gas_used, gas_report)
            }
            _ => (GasVector::ZERO, None),
        };

        let used_resources = match (show_detailed_resources, &test_result) {
            (true, AnyTestCaseSummary::Single(TestCaseSummary::Passed { used_resources, .. })) => {
                Some(GasCalculationResources::from_used_resources(used_resources))
            }
            _ => None,
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
            tracked_resource,
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
        let result_debug_trace = self
            .debugging_trace
            .as_ref()
            .map_or_default(|trace| format!("\n{trace}"));

        let fuzzer_report = self
            .fuzzer_report
            .as_ref()
            .map_or_default(std::string::ToString::to_string);

        let gas_usage = format!(
            " (l1_gas: ~{}, l1_data_gas: ~{}, l2_gas: ~{})",
            self.gas_usage.l1_gas, self.gas_usage.l1_data_gas, self.gas_usage.l2_gas
        );
        let gas_report = self.gas_report.as_deref().unwrap_or_default();
        let used_resources = self
            .used_resources
            .as_ref()
            .map_or_default(|res| res.format_for_display(self.tracked_resource));

        format!(
            "{result_header} {result_name}{fuzzer_report}{gas_usage}{used_resources}{result_msg}{result_debug_trace}{gas_report}"
        )
    }

    fn json(&self) -> Value {
        json!(self)
    }
}

#[derive(Serialize)]
#[serde(untagged)]
enum FuzzerReport {
    Passed {
        runs: usize,
        gas_info: GasFuzzingInfo,
    },
    Failed {
        runs: usize,
        args: Vec<String>,
    },
}

impl Display for FuzzerReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FuzzerReport::Passed { runs, gas_info } => write!(f, " (runs: {runs}, {gas_info})"),
            FuzzerReport::Failed { runs, args } => {
                write!(f, " (runs: {runs}, arguments: {args:?})")
            }
        }
    }
}
