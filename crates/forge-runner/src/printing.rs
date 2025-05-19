use crate::forge_config::ForgeTrackedResource;
use crate::test_case_summary::{AnyTestCaseSummary, FuzzingStatistics, TestCaseSummary};
use cairo_vm::vm::runners::cairo_runner::ExecutionResources;
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::UsedResources;
use console::style;

pub fn print_test_result(
    any_test_result: &AnyTestCaseSummary,
    print_detailed_resources: bool,
    tracked_resource: ForgeTrackedResource,
) {
    if any_test_result.is_skipped() {
        return;
    }
    let result_header = result_header(any_test_result);
    let result_name = any_test_result.name().unwrap();

    let result_msg = result_message(any_test_result);
    let result_debug_trace = result_debug_trace(any_test_result);

    let mut fuzzer_report = None;
    if let AnyTestCaseSummary::Fuzzing(test_result) = any_test_result {
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

    let used_resources = match (print_detailed_resources, any_test_result) {
        (true, AnyTestCaseSummary::Single(TestCaseSummary::Passed { used_resources, .. })) => {
            format_detailed_resources(used_resources, tracked_resource)
        }
        _ => String::new(),
    };

    println!(
        "{result_header} {result_name}{fuzzer_report}{gas_usage}{used_resources}{result_msg}{result_debug_trace}"
    );
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

            if used_resources.execution_resources != ExecutionResources::default() {
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

fn result_message(any_test_result: &AnyTestCaseSummary) -> String {
    if let Some(msg) = any_test_result.msg() {
        if any_test_result.is_passed() {
            return format!("\n\n{msg}");
        }
        if any_test_result.is_failed() {
            return format!("\n\nFailure data:{msg}");
        }
    }
    String::new()
}

fn result_header(any_test_result: &AnyTestCaseSummary) -> String {
    if any_test_result.is_passed() {
        return format!("[{}]", style("PASS").green());
    }
    if any_test_result.is_failed() {
        return format!("[{}]", style("FAIL").red());
    }
    if any_test_result.is_ignored() {
        return format!("[{}]", style("IGNORE").yellow());
    }
    unreachable!()
}

fn result_debug_trace(any_test_result: &AnyTestCaseSummary) -> String {
    any_test_result
        .debugging_trace()
        .map(|trace| format!("\n{trace}"))
        .unwrap_or_default()
}
