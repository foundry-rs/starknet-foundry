// TODO(#3022): Rename `printing.rs` to `messages.rs``
use crate::forge_config::ForgeTrackedResource;
use crate::test_case_summary::{AnyTestCaseSummary, FuzzingStatistics, TestCaseSummary};
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::UsedResources;
use console::style;
use foundry_ui::Message;
use serde::Serialize;

#[derive(Serialize)]
pub struct TestResultMessage {
    is_passed: bool,
    is_failed: bool,
    is_ignored: bool,
    name: String,
    msg: Option<String>,
    debugging_trace: String,
    fuzzer_report: String,
    gas_usage: String,
    used_resources: String,
}

impl TestResultMessage {
    pub fn new(
        any_test_result: &AnyTestCaseSummary,
        show_detailed_resources: bool,
        tracked_resource: ForgeTrackedResource,
    ) -> Self {
        let name = any_test_result
            .name()
            .expect("Test result must have a name")
            .to_string();

        let debugging_trace = any_test_result
            .debugging_trace()
            .map(|trace| format!("\n{trace}"))
            .unwrap_or_default();

        let mut fuzzer_report = None;
        if let AnyTestCaseSummary::Fuzzing(test_result) = &any_test_result {
            fuzzer_report = match test_result {
                TestCaseSummary::Passed {
                    test_statistics: FuzzingStatistics { runs },
                    gas_info,
                    ..
                } => Some(format!(" (runs: {runs}, {gas_info})",)),
                TestCaseSummary::Failed {
                    fuzzer_args,
                    test_statistics: FuzzingStatistics { runs },
                    ..
                } => Some(format!(" (runs: {runs}, arguments: {fuzzer_args:?})")),
                _ => None,
            };
        }
        let fuzzer_report = fuzzer_report.unwrap_or_else(String::new);

        let gas_usage = match any_test_result {
            AnyTestCaseSummary::Single(TestCaseSummary::Passed { gas_info, .. }) => {
                format!(
                    " (l1_gas: ~{}, l1_data_gas: ~{}, l2_gas: ~{})",
                    gas_info.l1_gas, gas_info.l1_data_gas, gas_info.l2_gas
                )
            }
            _ => String::new(),
        };

        let used_resources = match (show_detailed_resources, &any_test_result) {
            (true, AnyTestCaseSummary::Single(TestCaseSummary::Passed { used_resources, .. })) => {
                format_detailed_resources(used_resources, tracked_resource)
            }
            _ => String::new(),
        };

        Self {
            name,
            is_passed: any_test_result.is_passed(),
            is_failed: any_test_result.is_failed(),
            is_ignored: any_test_result.is_ignored(),
            msg: any_test_result.msg().map(std::string::ToString::to_string),
            debugging_trace,
            fuzzer_report,
            gas_usage,
            used_resources,
        }
    }

    fn result_message(&self) -> String {
        if let Some(msg) = &self.msg {
            if self.is_passed {
                return format!("\n\n{msg}");
            }
            if self.is_failed {
                return format!("\n\nFailure data:{msg}");
            }
        }
        String::new()
    }

    fn result_header(&self) -> String {
        if self.is_passed {
            return format!("[{}]", style("PASS").green());
        }
        if self.is_failed {
            return format!("[{}]", style("FAIL").red());
        }
        if self.is_ignored {
            return format!("[{}]", style("IGNORE").yellow());
        }
        unreachable!()
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
