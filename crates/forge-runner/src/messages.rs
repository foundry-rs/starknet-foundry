use crate::forge_config::ForgeTrackedResource;
use crate::test_case_summary::{AnyTestCaseSummary, FuzzingStatistics, TestCaseSummary};
use cairo_vm::vm::runners::cairo_runner::ExecutionResources;
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::UsedResources;
use console::style;
use foundry_ui::components::warning::WarningMessage;
use foundry_ui::{Message, UI};
use serde::Serialize;
use serde_json::{Value, json};

#[derive(Serialize)]
enum TestResultStatus {
    Passed,
    Failed,
    Ignored,
    Interrupted,
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
}

impl TestResultMessage {
    pub fn new(
        test_result: &AnyTestCaseSummary,
        show_detailed_resources: bool,
        tracked_resource: ForgeTrackedResource,
        ui: &UI,
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

        let gas_usage = match test_result {
            AnyTestCaseSummary::Single(TestCaseSummary::Passed { gas_info, .. }) => {
                format!(
                    " (l1_gas: ~{}, l1_data_gas: ~{}, l2_gas: ~{})",
                    gas_info.l1_gas, gas_info.l1_data_gas, gas_info.l2_gas
                )
            }
            _ => String::new(),
        };

        let used_resources = match (show_detailed_resources, &test_result) {
            (true, AnyTestCaseSummary::Single(TestCaseSummary::Passed { used_resources, .. })) => {
                format_detailed_resources(used_resources, tracked_resource, ui)
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
        }
    }

    fn result_message(&self) -> String {
        if let Some(msg) = &self.msg {
            match self.status {
                TestResultStatus::Passed => return format!("\n\n{msg}"),
                TestResultStatus::Failed => return format!("\n\nFailure data:{msg}"),
                TestResultStatus::Ignored | TestResultStatus::Interrupted => return String::new(),
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
        let used_resources = &self.used_resources;

        format!(
            "{result_header} {result_name}{fuzzer_report}{gas_usage}{used_resources}{result_msg}{result_debug_trace}"
        )
    }

    fn json(&self) -> Value {
        json!(self)
    }
}

fn format_detailed_resources(
    used_resources: &UsedResources,
    tracked_resource: ForgeTrackedResource,
    ui: &UI,
) -> String {
    // Sort syscalls by call count
    let mut syscall_usage: Vec<_> = used_resources
        .syscall_usage
        .iter()
        .map(|(selector, usage)| (selector, usage.call_count))
        .collect();
    syscall_usage.sort_by(|a, b| b.1.cmp(&a.1));

    let format_vm_resources = |vm_resources: &ExecutionResources| -> String {
        let sorted_builtins = sort_by_value(&vm_resources.builtin_instance_counter);
        let builtins = format_items(&sorted_builtins);

        format!(
            "
        steps: {}
        memory holes: {}
        builtins: ({})",
            vm_resources.n_steps, vm_resources.n_memory_holes, builtins
        )
    };

    let syscalls = format_items(&syscall_usage);
    let vm_resources_output = format_vm_resources(&used_resources.execution_resources);

    match tracked_resource {
        ForgeTrackedResource::CairoSteps => {
            format!(
                "{vm_resources_output}
        syscalls: ({syscalls})
        "
            )
        }
        ForgeTrackedResource::SierraGas => {
            let mut output = format!(
                "
        sierra_gas_consumed: {}
        syscalls: ({syscalls})",
                used_resources.gas_consumed.0
            );

            // TODO(#3399): Remove this warning and `ui` from parameter list
            if used_resources.execution_resources != ExecutionResources::default() {
                ui.println(&WarningMessage::new(
                    "When tracking sierra gas and executing contracts with a Sierra version older than 1.7.0, \
                    syscall related resources may be incorrectly reported to the wrong resource type \
                    in the output of `--detailed-resources` flag."
                ));
                output.push_str(&vm_resources_output);
            }
            output.push('\n');

            output
        }
    }
}

fn sort_by_value<'a, K, V, M>(map: M) -> Vec<(&'a K, &'a V)>
where
    M: IntoIterator<Item = (&'a K, &'a V)>,
    V: Ord,
{
    let mut sorted: Vec<_> = map.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));
    sorted
}

fn format_items<K, V>(items: &[(K, V)]) -> String
where
    K: std::fmt::Debug,
    V: std::fmt::Display,
{
    items
        .iter()
        .map(|(key, value)| format!("{key:?}: {value}"))
        .collect::<Vec<String>>()
        .join(", ")
}
