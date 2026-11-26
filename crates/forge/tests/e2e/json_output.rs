use super::common::runner::{setup_package, test_runner};
use indoc::indoc;
use serde_json::Value;
use shared::test_utils::output_assert::assert_stdout_contains;

fn json_messages(stdout: &[u8]) -> Vec<Value> {
    stdout
        .split(|&byte| byte == b'\n')
        .filter_map(|line| serde_json::from_slice(line).ok())
        .collect()
}

#[track_caller]
fn test_result<'a>(messages: &'a [Value], name: &str) -> &'a Value {
    messages
        .iter()
        .find(|message| message["name"] == name)
        .unwrap_or_else(|| panic!("Missing JSON test result for {name:?}"))
}

#[test]
fn collection_and_summary() {
    let temp = setup_package("simple_package");
    let output = test_runner(&temp)
        .args([
            "--json",
            "simple_package_integrationtest::test_simple::test_failing",
        ])
        .assert()
        .code(1);
    assert_stdout_contains(
        output,
        indoc! {r#"
            {"package_name":"simple_package","tests_num":1}
            {"test_target_location":"Lib","tests_num":0}
            {"test_target_location":"Tests","tests_num":1}
            {"summary":{"failed":1,"filtered":12,"ignored":0,"interrupted":0,"passed":0}}
            {"failed_test_names":["simple_package_integrationtest::test_simple::test_failing"]}
        "#},
    );
}

#[test]
fn passed_test_result() {
    let temp = setup_package("simple_package");
    let name = "simple_package::tests::test_fib";
    let output = test_runner(&temp)
        .args(["--json", "--exact", name])
        .assert()
        .success();
    let messages = json_messages(&output.get_output().stdout);
    let result = test_result(&messages, name);

    assert_eq!(result["status"], "passed");
    assert!(
        result["gas_usage"]["l2_gas"]
            .as_u64()
            .is_some_and(|gas| gas > 0)
    );
}

#[test]
fn ignored_test_result() {
    let temp = setup_package("simple_package");
    let name = "simple_package::tests::ignored_test";
    let output = test_runner(&temp)
        .args(["--json", "--exact", name])
        .assert()
        .success();
    let messages = json_messages(&output.get_output().stdout);

    assert_eq!(test_result(&messages, name)["status"], "ignored");
}

#[test]
fn failed_test_result() {
    let temp = setup_package("simple_package");
    let name = "simple_package_integrationtest::test_simple::test_failing";
    let output = test_runner(&temp)
        .args(["--json", "--exact", name])
        .assert()
        .code(1);
    let messages = json_messages(&output.get_output().stdout);
    let result = test_result(&messages, name);

    assert_eq!(result["status"], "failed");
    assert!(
        result["msg"]
            .as_str()
            .unwrap()
            .contains("0x6661696c696e6720636865636b ('failing check')")
    );
}

#[test]
fn detailed_test_result() {
    let temp = setup_package("simple_package");
    let name = "simple_package_integrationtest::contract::call_and_invoke";
    let output = test_runner(&temp)
        .args([
            "--json",
            "--exact",
            name,
            "--detailed-resources",
            "--gas-report",
            "--trace-verbosity",
            "minimal",
        ])
        .assert()
        .success();
    let messages = json_messages(&output.get_output().stdout);
    let result = test_result(&messages, name);

    assert_eq!(result["status"], "passed");
    assert!(
        result["gas_report"]
            .as_str()
            .unwrap()
            .contains("increase_balance")
    );
    assert!(result["used_resources"]["sierra_gas"].is_u64());
    assert!(result["used_resources"]["vm_resources"].is_object());
    assert!(result["used_resources"]["syscalls"].is_object());
    let trace = &result["debugging_trace"];
    assert_eq!(trace["test_name"], name);
    let nested_calls = serde_json::to_string_pretty(&trace["nested_calls"]).unwrap();
    insta::assert_snapshot!(nested_calls);
}

#[test]
fn passed_fuzzer_report() {
    let temp = setup_package("fuzzing");
    let name = "fuzzing::tests::fuzzed_argument";
    let output = test_runner(&temp)
        .args([
            "--json",
            "--exact",
            name,
            "--fuzzer-runs",
            "3",
            "--fuzzer-seed",
            "1234",
        ])
        .env("CLICOLOR_FORCE", "0")
        .assert()
        .success();
    let messages = json_messages(&output.get_output().stdout);
    let passed = test_result(&messages, name);

    assert_eq!(passed["status"], "passed");
    assert_eq!(passed["fuzzer_report"]["runs"], 3);
    let gas_info = &passed["fuzzer_report"]["gas_info"];
    for resource in ["l1_gas", "l1_data_gas", "l2_gas"] {
        let stats = &gas_info[resource];
        assert!(stats["min"].is_u64());
        assert!(stats["max"].is_u64());
        assert!(stats["mean"].is_number());
        assert!(stats["std_deviation"].is_number());
    }
    assert_stdout_contains(
        output,
        r#"{"label":"Fuzzer seed","message":{"message":"1234"},"message_type":"labeled"}"#,
    );
}

#[test]
fn failed_fuzzer_report() {
    let temp = setup_package("fuzzing");
    let name = "fuzzing::tests::failing_fuzz";
    let output = test_runner(&temp)
        .args(["--json", "--exact", name, "--fuzzer-seed", "1234"])
        .assert()
        .code(1);
    let messages = json_messages(&output.get_output().stdout);
    let failed = test_result(&messages, name);

    assert_eq!(failed["status"], "failed");
    assert_eq!(failed["fuzzer_report"]["runs"], 1);
    let args = failed["fuzzer_report"]["args"].as_array().unwrap();
    assert_eq!(args.len(), 2);
    assert!(args.iter().all(Value::is_string));
}

#[test]
fn workspace_partition_summary() {
    let temp = setup_package("partitioning");
    let output = test_runner(&temp)
        .args([
            "--json",
            "--partition",
            "1/2",
            "--workspace",
            "package_a::tests::test_a",
        ])
        .assert()
        .success();
    assert_stdout_contains(
        output,
        indoc! {r#"
            {"partition":{"index":1,"total":2}}
            {"summary":{"failed":0,"filtered":6,"ignored":0,"interrupted":0,"passed":1}}
            {"included_tests_count":7,"partition":{"index":1,"total":2},"total_tests_count":13}
        "#},
    );
}

#[test]
fn command_error() {
    let temp = setup_package("empty");
    let output = test_runner(&temp)
        .args(["--json", "--package", "missing_package"])
        .assert()
        .code(2);
    assert_stdout_contains(
        output,
        r#"{"message":{"message":"Failed to find any packages matching the specified filter[..]"},"message_type":"error"}"#,
    );
}
