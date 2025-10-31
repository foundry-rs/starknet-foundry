use crate::e2e::common::runner::{setup_package, test_runner};
use indoc::indoc;
use shared::test_utils::output_assert::assert_stdout_contains;

#[test]
fn basic() {
    let temp = setup_package("simple_package");
    let output = test_runner(&temp)
        .arg("--gas-report")
        .arg("call_and_invoke")
        .assert()
        .code(0);

    assert_stdout_contains(
        output,
        indoc! {r"
    [..]Compiling[..]
    [..]Finished[..]


    Collected 1 test(s) from simple_package package
    Running 0 test(s) from src/
    Running 1 test(s) from tests/
    [PASS] simple_package_integrationtest::contract::call_and_invoke (l1_gas: ~0, l1_data_gas: ~[..], l2_gas: ~[..])
    ╭------------------------+-------+-------+-------+---------+---------╮
    | HelloStarknet Contract |       |       |       |         |         |
    +====================================================================+
    | Function Name          | Min   | Max   | Avg   | Std Dev | # Calls |
    |------------------------+-------+-------+-------+---------+---------|
    | get_balance            | 13340 | 13340 | 13340 | 0       | 2       |
    |------------------------+-------+-------+-------+---------+---------|
    | increase_balance       | 24940 | 24940 | 24940 | 0       | 1       |
    ╰------------------------+-------+-------+-------+---------+---------╯

    Tests: 1 passed, 0 failed, 0 ignored, [..] filtered out
    "},
    );
}

#[test]
fn recursive_calls() {
    let temp = setup_package("debugging");

    let output = test_runner(&temp)
        .arg("test_debugging_trace_success")
        .arg("--gas-report")
        .assert()
        .code(0);

    assert_stdout_contains(
        output,
        indoc! {r"
    [..]Compiling[..]
    [..]Finished[..]


    Collected 1 test(s) from debugging package
    Running 0 test(s) from src/
    Running 1 test(s) from tests/
    [PASS] debugging_integrationtest::test_trace::test_debugging_trace_success (l1_gas: ~0, l1_data_gas: ~[..], l2_gas: ~[..])
    ╭-------------------------+-------+--------+--------+---------+---------╮
    | SimpleContract Contract |       |        |        |         |         |
    +=======================================================================+
    | Function Name           | Min   | Max    | Avg    | Std Dev | # Calls |
    |-------------------------+-------+--------+--------+---------+---------|
    | execute_calls           | 11680 | 609080 | 183924 | 235859  | 5       |
    |-------------------------+-------+--------+--------+---------+---------|
    | fail                    | 17950 | 17950  | 17950  | 0       | 1       |
    ╰-------------------------+-------+--------+--------+---------+---------╯

    Tests: 1 passed, 0 failed, 0 ignored, [..] filtered out
    "},
    );
}

#[test]
fn multiple_contracts_and_constructor() {
    let temp = setup_package("simple_package_with_cheats");
    let output = test_runner(&temp)
        .arg("--gas-report")
        .arg("call_and_invoke_proxy")
        .assert()
        .code(0);

    assert_stdout_contains(
        output,
        indoc! {r"
    [..]Compiling[..]
    [..]Finished[..]


    Collected 1 test(s) from simple_package_with_cheats package
    Running 0 test(s) from src/
    Running 1 test(s) from tests/
    [PASS] simple_package_with_cheats_integrationtest::contract::call_and_invoke_proxy (l1_gas: ~0, l1_data_gas: ~[..], l2_gas: ~[..])
    ╭------------------------+-------+-------+-------+---------+---------╮
    | HelloStarknet Contract |       |       |       |         |         |
    +====================================================================+
    | Function Name          | Min   | Max   | Avg   | Std Dev | # Calls |
    |------------------------+-------+-------+-------+---------+---------|
    | get_block_number       | 15780 | 15780 | 15780 | 0       | 2       |
    ╰------------------------+-------+-------+-------+---------+---------╯

    ╭-----------------------------+--------+--------+--------+---------+---------╮
    | HelloStarknetProxy Contract |        |        |        |         |         |
    +============================================================================+
    | Function Name               | Min    | Max    | Avg    | Std Dev | # Calls |
    |-----------------------------+--------+--------+--------+---------+---------|
    | constructor                 | 14650  | 14650  | 14650  | 0       | 1       |
    |-----------------------------+--------+--------+--------+---------+---------|
    | get_block_number            | 125280 | 125280 | 125280 | 0       | 2       |
    ╰-----------------------------+--------+--------+--------+---------+---------╯

    Tests: 1 passed, 0 failed, 0 ignored, [..] filtered out
    "},
    );
}

#[test]
fn no_transactions() {
    let temp = setup_package("simple_package");
    let output = test_runner(&temp)
        .arg("--gas-report")
        .arg("test_fib")
        .assert()
        .code(0);

    assert_stdout_contains(
        output,
        indoc! {r"
    [..]Compiling[..]
    [..]Finished[..]


    Collected 1 test(s) from simple_package package
    Running 1 test(s) from src/
    [PASS] simple_package::tests::test_fib (l1_gas: ~0, l1_data_gas: ~0, l2_gas: ~[..])
    No contract gas usage data to display. Make sure your test include transactions.

    Running 0 test(s) from tests/
    Tests: 1 passed, 0 failed, 0 ignored, [..] filtered out
    "},
    );
}
