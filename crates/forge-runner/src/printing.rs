use crate::{
    test_case_summary::{AnyTestCaseSummary, FuzzingStatistics, TestCaseSummary},
    RunnerConfig,
};
use console::style;

pub(crate) fn print_test_result(
    any_test_result: &AnyTestCaseSummary,
    runner_config: &RunnerConfig,
) {
    if any_test_result.is_skipped() {
        return;
    }
    let result_header = result_header(any_test_result);
    let result_name = any_test_result.name().unwrap();

    let result_msg = result_message(any_test_result);

    let mut fuzzer_report = None;
    if let AnyTestCaseSummary::Fuzzing(test_result) = any_test_result {
        fuzzer_report = match test_result {
            TestCaseSummary::Passed {
                test_statistics: FuzzingStatistics { runs },
                gas_info,
                ..
            } => Some(format!(
                " (runs: {runs}, gas: {{max: ~{}, min: ~{}, mean: ~{:.2}, std deviation: ~{:.2}}})",
                gas_info.max, gas_info.min, gas_info.mean, gas_info.std_deviation
            )),
            TestCaseSummary::Failed {
                arguments,
                test_statistics: FuzzingStatistics { runs },
                ..
            } => Some(format!(" (runs: {runs}, arguments: {arguments:?})")),
            _ => None,
        };
    }
    let fuzzer_report = fuzzer_report.unwrap_or_else(String::new);

    let gas_usage = match any_test_result {
        AnyTestCaseSummary::Single(TestCaseSummary::Passed { gas_info, .. }) => {
            format!(" (gas: ~{gas_info})")
        }
        _ => String::new(),
    };

    let used_resources = match any_test_result {
        AnyTestCaseSummary::Single(TestCaseSummary::Passed { used_resources, .. }) => {
            fn sort_by_value<K, V>(map: &std::collections::HashMap<K, V>) -> Vec<(&K, &V)>
            where
                V: Ord,
            {
                let mut sorted: Vec<_> = map.iter().collect();
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

            let vm_res = &used_resources.execution_resources.vm_resources;

            let sorted_builtins = sort_by_value(&vm_res.builtin_instance_counter);
            let sorted_syscalls =
                sort_by_value(&used_resources.execution_resources.syscall_counter);

            let builtins = format_items(&sorted_builtins);
            let syscalls = format_items(&sorted_syscalls);

            format!(
                "
        steps: {}
        memory holes: {}
        builtins: ({})
        syscalls: ({})
            ",
                vm_res.n_steps, vm_res.n_memory_holes, builtins, syscalls,
            )
        }
        _ => String::new(),
    };

    println!(
        "{result_header} {result_name}{fuzzer_report}{gas_usage}{}{result_msg}",
        if runner_config.detailed_resources {
            &used_resources
        } else {
            ""
        }
    );
}

fn result_message(any_test_result: &AnyTestCaseSummary) -> String {
    if let Some(msg) = any_test_result.msg() {
        if any_test_result.is_passed() {
            return format!("\n\nSuccess data:{msg}");
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
