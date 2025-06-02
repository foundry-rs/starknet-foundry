use crate::forge_config::ForgeTrackedResource;
use crate::test_case_summary::{AnyTestCaseSummary, FuzzingStatistics, TestCaseSummary};
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::UsedResources;
use console::style;
use foundry_ui::Message;
use serde::Serialize;

#[derive(Serialize)]
pub enum TestResultStatus {
    Passed,
    Failed,
    Ignored,
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
                format_detailed_resources(used_resources, tracked_resource)
            }
            _ => String::new(),
        };

        Self {
            name,
            msg: test_result.msg().map(std::string::ToString::to_string),
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
                TestResultStatus::Failed => return format!("\n\nFailure data: {msg}"),
                TestResultStatus::Ignored => return String::new(),
            }
        }
        String::new()
    }

    fn result_header(&self) -> String {
        match self.status {
            TestResultStatus::Passed => format!("[{}]", style("PASS").green()),
            TestResultStatus::Failed => format!("[{}]", style("FAIL").red()),
            TestResultStatus::Ignored => format!("[{}]", style("IGNORE").yellow()),
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
}

fn format_detailed_resources(
    used_resources: &UsedResources,
    tracked_resource: ForgeTrackedResource,
) -> String {
    // Sort syscalls by call count
    let mut syscall_usage: Vec<_> = used_resources
        .syscall_usage
        .iter()
        .map(|(selector, usage)| (selector, usage.call_count))
        .collect();
    syscall_usage.sort_by(|a, b| b.1.cmp(&a.1));

    let syscalls = format_items(&syscall_usage);

    match tracked_resource {
        ForgeTrackedResource::CairoSteps => {
            let vm_resources = &used_resources.execution_resources;
            let sorted_builtins = sort_by_value(&vm_resources.builtin_instance_counter);
            let builtins = format_items(&sorted_builtins);

            format!(
                "
        steps: {}
        memory holes: {}
        builtins: ({})
        syscalls: ({})
            ",
                vm_resources.n_steps, vm_resources.n_memory_holes, builtins, syscalls,
            )
        }
        ForgeTrackedResource::SierraGas => {
            format!(
                "
        sierra_gas_consumed: ({})
        syscalls: ({})
            ",
                used_resources.gas_consumed.0, syscalls,
            )
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
