use crate::e2e::common::runner::setup_package_at_path;
use camino::Utf8PathBuf;
use indoc::{formatdoc, indoc};
use scarb_api::ScarbCommand;
use shared::test_utils::output_assert::{assert_stdout_contains, case_assert_stdout_contains};
use snapbox::cmd::Command as SnapboxCommand;
use std::fs;

#[test]
#[cfg_attr(feature = "skip_plugin_checks", ignore = "Plugin checks skipped")]
#[allow(clippy::too_many_lines)]
fn syntax() {
    let temp = setup_package_at_path(Utf8PathBuf::from("diagnostics/syntax"));
    let output = SnapboxCommand::from_std(
        ScarbCommand::new()
            .current_dir(temp.path())
            .args(["build", "--test"])
            .command(),
    )
    .assert()
    .failure();

    assert_stdout_contains(
        output,
        indoc! {r#"
        error: Missing token ';'.
         --> [..]/tests/contract.cairo:14:70
            let (contract_address, _) = contract.deploy(constructor_calldata),unwrap();
                                                                             ^
        
        error: Skipped tokens. Expected: statement.
         --> [..]/tests/contract.cairo:14:70
            let (contract_address, _) = contract.deploy(constructor_calldata),unwrap();
                                                                             ^
        
        error: Missing token ';'.
         --> [..]/tests/contract.cairo:14:70
            let (contract_address, _) = contract.deploy(constructor_calldata),unwrap();
                                                                             ^
        note: this error originates in the attribute macro: `test`
        
        error: Skipped tokens. Expected: statement.
         --> [..]/tests/contract.cairo:14:70
            let (contract_address, _) = contract.deploy(constructor_calldata),unwrap();
                                                                             ^
        note: this error originates in the attribute macro: `test`
        
        error: Missing token ';'.
         --> [..]/tests/contract.cairo:14:70
            let (contract_address, _) = contract.deploy(constructor_calldata),unwrap();
                                                                             ^
        note: this error originates in the attribute macro: `test`
        note: this error originates in the attribute macro: `fuzzer`
        
        error: Skipped tokens. Expected: statement.
         --> [..]/tests/contract.cairo:14:70
            let (contract_address, _) = contract.deploy(constructor_calldata),unwrap();
                                                                             ^
        note: this error originates in the attribute macro: `test`
        note: this error originates in the attribute macro: `fuzzer`
        
        error: Missing token ';'.
         --> [..]/tests/contract.cairo:14:70
            let (contract_address, _) = contract.deploy(constructor_calldata),unwrap();
                                                                             ^
        note: this error originates in the attribute macro: `test`
        note: this error originates in the attribute macro: `fuzzer`
        note: this error originates in the attribute macro: `fork`
        
        error: Skipped tokens. Expected: statement.
         --> [..]/tests/contract.cairo:14:70
            let (contract_address, _) = contract.deploy(constructor_calldata),unwrap();
                                                                             ^
        note: this error originates in the attribute macro: `test`
        note: this error originates in the attribute macro: `fuzzer`
        note: this error originates in the attribute macro: `fork`
        
        error: Plugin diagnostic: Failed because of invalid syntax
         --> [..]/tests/contract.cairo:7:1
        #[test]
        ^^^^^^^
        
        error: Plugin diagnostic: Failed because of invalid syntax
         --> [..]/tests/contract.cairo:8:1
        #[fuzzer]
        ^^^^^^^^^
        note: this error originates in the attribute macro: `test`
        
        error: Plugin diagnostic: Failed because of invalid syntax
         --> [..]/tests/contract.cairo:9:1
        #[fork(url: "http://127.0.0.1:3030", block_tag: latest)]
        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        note: this error originates in the attribute macro: `test`
        note: this error originates in the attribute macro: `fuzzer`
        
        error: Plugin diagnostic: Failed because of invalid syntax
         --> [..]/tests/contract.cairo:10:1
        #[ignore]
        ^^^^^^^^^
        note: this error originates in the attribute macro: `test`
        note: this error originates in the attribute macro: `fuzzer`
        note: this error originates in the attribute macro: `fork`
        
        error: Unexpected type for tuple pattern. "core::result::Result::<(core::starknet::contract_address::ContractAddress, core::array::Span::<core::felt252>), core::array::Array::<core::felt252>>" is not a tuple.
         --> [..]/tests/contract.cairo:14:9
            let (contract_address, _) = contract.deploy(constructor_calldata),unwrap();
                ^^^^^^^^^^^^^^^^^^^^^
        note: this error originates in the attribute macro: `test`
        note: this error originates in the attribute macro: `fuzzer`
        note: this error originates in the attribute macro: `fork`
        
        error[E0006]: Function not found.
         --> [..]/tests/contract.cairo:14:71
            let (contract_address, _) = contract.deploy(constructor_calldata),unwrap();
                                                                              ^^^^^^
        note: this error originates in the attribute macro: `test`
        note: this error originates in the attribute macro: `fuzzer`
        note: this error originates in the attribute macro: `fork`
        
        error: could not compile `syntax_integrationtest` due to previous error
    "#},
    );
}

#[test]
#[cfg_attr(feature = "skip_plugin_checks", ignore = "Plugin checks skipped")]
fn semantic() {
    let temp = setup_package_at_path(Utf8PathBuf::from("diagnostics/semantic"));
    let output = SnapboxCommand::from_std(
        ScarbCommand::new()
            .current_dir(temp.path())
            .args(["build", "--test"])
            .command(),
    )
    .assert()
    .failure();

    assert_stdout_contains(
        output,
        indoc! {r"
        error[E0006]: Identifier not found.
         --> [..]/tests/contract.cairo:21:13
            let y = x;
                    ^
        note: this error originates in the attribute macro: `test`
        note: this error originates in the attribute macro: `fuzzer`
        note: this error originates in the attribute macro: `__fuzzer_config`
        note: this error originates in the attribute macro: `__fuzzer_wrapper`
        note: this error originates in the attribute macro: `fork`
        note: this error originates in the attribute macro: `ignore`
        note: this error originates in the attribute macro: `__internal_config_statement`
        
        warn[E0001]: Unused variable. Consider ignoring by prefixing with `_`.
         --> [..]/tests/contract.cairo:21:9
            let y = x;
                ^
        note: this error originates in the attribute macro: `test`
        note: this error originates in the attribute macro: `fuzzer`
        note: this error originates in the attribute macro: `__fuzzer_config`
        note: this error originates in the attribute macro: `__fuzzer_wrapper`
        note: this error originates in the attribute macro: `fork`
        note: this error originates in the attribute macro: `ignore`
        note: this error originates in the attribute macro: `__internal_config_statement`
        
        error: could not compile `semantic_integrationtest` due to previous error
        "},
    );
}

#[test]
#[cfg_attr(feature = "skip_plugin_checks", ignore = "Plugin checks skipped")]
fn parameters() {
    let temp = setup_package_at_path(Utf8PathBuf::from("diagnostics/parameters"));
    let output = SnapboxCommand::from_std(
        ScarbCommand::new()
            .current_dir(temp.path())
            .args(["build", "--test"])
            .command(),
    )
    .assert()
    .failure();

    assert_stdout_contains(
        output,
        indoc! {r#"
        error: Missing token ','.
         --> [..]/tests/contract.cairo:9:31
        fn call_and_invoke(_a: felt252; b: u256) {
                                      ^
        
        error: Skipped tokens. Expected: parameter.
         --> [..]/tests/contract.cairo:9:31
        fn call_and_invoke(_a: felt252; b: u256) {
                                      ^
        
        error: Missing token ','.
         --> [..]/tests/contract.cairo:9:31
        fn call_and_invoke(_a: felt252; b: u256) {
                                      ^
        note: this error originates in the attribute macro: `test`
        
        error: Skipped tokens. Expected: parameter.
         --> [..]/tests/contract.cairo:9:31
        fn call_and_invoke(_a: felt252; b: u256) {
                                      ^^
        note: this error originates in the attribute macro: `test`
        
        error: Plugin diagnostic: Failed because of invalid syntax
         --> [..]/tests/contract.cairo:7:1
        #[test]
        ^^^^^^^
        
        error: Plugin diagnostic: Failed because of invalid syntax
         --> [..]/tests/contract.cairo:8:1
        #[fork("TESTNET")]
        ^^^^^^^^^^^^^^^^^^
        note: this error originates in the attribute macro: `test`
        
        error: could not compile `parameters_integrationtest` due to previous error
    "#},
    );
}

#[test]
#[cfg_attr(feature = "skip_plugin_checks", ignore = "Plugin checks skipped")]
fn multiple() {
    let temp = setup_package_at_path(Utf8PathBuf::from("diagnostics/multiple"));
    let output = SnapboxCommand::from_std(
        ScarbCommand::new()
            .current_dir(temp.path())
            .args(["build", "--test"])
            .command(),
    )
    .assert()
    .failure();

    assert_stdout_contains(
        output,
        indoc! {r#"
        error: Missing tokens. Expected an expression.
         --> [..]/tests/contract.cairo:19:22
            assert(balance === 0, 'balance == 0');
                             ^
        
        error: Missing tokens. Expected an expression.
         --> [..]/tests/contract.cairo:19:22
            assert(balance === 0, 'balance == 0');
                             ^
        note: this error originates in the attribute macro: `test`
        
        error: Missing tokens. Expected an expression.
         --> [..]/tests/contract.cairo:19:22
            assert(balance === 0, 'balance == 0');
                             ^
        note: this error originates in the attribute macro: `test`
        note: this error originates in the attribute macro: `fuzzer`
        
        error: Missing tokens. Expected an expression.
         --> [..]/tests/contract.cairo:19:22
            assert(balance === 0, 'balance == 0');
                             ^
        note: this error originates in the attribute macro: `test`
        note: this error originates in the attribute macro: `fuzzer`
        note: this error originates in the attribute macro: `fork`
        
        error: Plugin diagnostic: Failed because of invalid syntax
         --> [..]/tests/contract.cairo:7:1
        #[test]
        ^^^^^^^
        
        error: Plugin diagnostic: Failed because of invalid syntax
         --> [..]/tests/contract.cairo:8:1
        #[fuzzer]
        ^^^^^^^^^
        note: this error originates in the attribute macro: `test`
        
        error: Plugin diagnostic: Failed because of invalid syntax
         --> [..]/tests/contract.cairo:9:1
        #[fork(url: "http://127.0.0.1:3030", block_tag: latest)]
        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        note: this error originates in the attribute macro: `test`
        note: this error originates in the attribute macro: `fuzzer`
        
        error: Plugin diagnostic: Failed because of invalid syntax
         --> [..]/tests/contract.cairo:10:1
        #[ignore]
        ^^^^^^^^^
        note: this error originates in the attribute macro: `test`
        note: this error originates in the attribute macro: `fuzzer`
        note: this error originates in the attribute macro: `fork`
        
        error: Unsupported feature.
         --> [..]/tests/contract.cairo:19:22
            assert(balance === 0, 'balance == 0');
                             ^
        note: this error originates in the attribute macro: `test`
        note: this error originates in the attribute macro: `fuzzer`
        note: this error originates in the attribute macro: `fork`
        
        error: Invalid left-hand side of assignment.
         --> [..]/tests/contract.cairo:19:12
            assert(balance === 0, 'balance == 0');
                   ^^^^^^^^^^
        note: this error originates in the attribute macro: `test`
        note: this error originates in the attribute macro: `fuzzer`
        note: this error originates in the attribute macro: `fork`
        
        error[E0006]: Function not found.
         --> [..]/tests/contract.cairo:57:30
            let balance = dispatcher/get_balance();
                                     ^^^^^^^^^^^
        note: this error originates in the attribute macro: `test`
        note: this error originates in the attribute macro: `fuzzer`
        note: this error originates in the attribute macro: `__fuzzer_config`
        note: this error originates in the attribute macro: `__fuzzer_wrapper`
        note: this error originates in the attribute macro: `fork`
        note: this error originates in the attribute macro: `ignore`
        note: this error originates in the attribute macro: `__internal_config_statement`
        
        error: could not compile `multiple_integrationtest` due to previous error
    "#},
    );
}

#[test]
#[cfg_attr(feature = "skip_plugin_checks", ignore = "Plugin checks skipped")]
fn generic() {
    let temp = setup_package_at_path(Utf8PathBuf::from("diagnostics/generic"));
    let output = SnapboxCommand::from_std(
        ScarbCommand::new()
            .current_dir(temp.path())
            .args(["build", "--test"])
            .command(),
    )
    .assert()
    .failure();

    assert_stdout_contains(
        output,
        indoc! {r"
        error: Trait has no implementation in context: core::traits::PartialOrd::<generic_integrationtest::contract::MyStruct>.
         --> [..]/tests/contract.cairo:29:13
            let s = smallest_element(@list);
                    ^^^^^^^^^^^^^^^^
        note: this error originates in the attribute macro: `test`
        note: this error originates in the attribute macro: `fuzzer`
        note: this error originates in the attribute macro: `__fuzzer_config`
        note: this error originates in the attribute macro: `__fuzzer_wrapper`
        note: this error originates in the attribute macro: `fork`
        note: this error originates in the attribute macro: `ignore`
        note: this error originates in the attribute macro: `__internal_config_statement`
        
        error: could not compile `generic_integrationtest` due to previous error
    "},
    );
}

#[test]
#[cfg_attr(feature = "skip_plugin_checks", ignore = "Plugin checks skipped")]
fn inline_macros() {
    let temp = setup_package_at_path(Utf8PathBuf::from("diagnostics/inline_macros"));
    let output = SnapboxCommand::from_std(
        ScarbCommand::new()
            .current_dir(temp.path())
            .args(["build", "--test"])
            .command(),
    )
    .assert()
    .failure();

    assert_stdout_contains(
        output,
        indoc! {r"
        error: Plugin diagnostic: Macro can not be parsed as legacy macro. Expected an argument list wrapped in either parentheses, brackets, or braces.
         --> [..]/tests/contract.cairo:23:5
            print!('balance {}'; balance);
            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        note: this error originates in the attribute macro: `test`
        note: this error originates in the attribute macro: `fuzzer`
        note: this error originates in the attribute macro: `__fuzzer_config`
        note: this error originates in the attribute macro: `__fuzzer_wrapper`
        note: this error originates in the attribute macro: `fork`
        note: this error originates in the attribute macro: `ignore`
        note: this error originates in the attribute macro: `__internal_config_statement`
        
        error: could not compile `inline_macros_integrationtest` due to previous error
    "},
    );
}

#[test]
#[cfg_attr(feature = "skip_plugin_checks", ignore = "Plugin checks skipped")]
fn different_attributes() {
    fn generate_attributes() -> impl Iterator<Item = String> {
        let attributes = vec![
            "#[fuzzer]".to_string(),
            r#"#[fork(url: "http://127.0.0.1:3030", block_tag: latest)]"#.to_string(),
            "#[ignore]".to_string(),
            "#[available_gas(l1_gas: 100, l2_gas: 200, l1_data_gas)]".to_string(),
            "#[should_panic(expected: 'panic message')]".to_string(),
        ];
        attributes.into_iter()
    }

    for attribute in generate_attributes() {
        let temp = setup_package_at_path(Utf8PathBuf::from("diagnostics/attributes"));
        let test_file = temp.join("tests/contract.cairo");

        let test_file_contents = fs::read_to_string(&test_file).unwrap();
        let test_file_contents = test_file_contents.replace("@attrs@", &attribute);
        fs::write(&test_file, test_file_contents).unwrap();

        let output = SnapboxCommand::new("scarb")
            .current_dir(&temp)
            .args(["build", "--test"])
            .assert()
            .failure();
        let expected_underline = "^".repeat(attribute.len());

        case_assert_stdout_contains(
            attribute.clone(),
            output,
            formatdoc! {r"
            error: Missing token ';'.
             --> [..]/tests/contract.cairo:10:81
                let (_contract_address1, _) = contract.deploy(constructor_calldata).unwrap()
                                                                                            ^
            
            error: Plugin diagnostic: Failed because of invalid syntax
             --> [..]/tests/contract.cairo:6:1
            {attribute}
            {expected_underline}
            
            error: could not compile `attributes_integrationtest` due to previous error
    "},
        );
    }
}
