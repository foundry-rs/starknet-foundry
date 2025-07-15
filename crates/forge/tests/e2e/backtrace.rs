use super::common::runner::{setup_package, test_runner};
use assert_fs::TempDir;
use assert_fs::fixture::{FileWriteStr, PathChild};
use indoc::indoc;
use shared::test_utils::output_assert::{AsOutput, assert_stdout_contains};
use std::fs;
use test_utils::use_snforge_std_compatibility;
use toml_edit::{DocumentMut, value};

#[test]
fn test_backtrace_missing_env() {
    let temp = setup_package("backtrace_vm_error");

    let output = test_runner(&temp).assert().failure();

    assert_stdout_contains(
        output,
        indoc! {
           "Failure data:
            Got an exception while executing a hint: Requested contract address 0x0000000000000000000000000000000000000000000000000000000000000123 is not deployed.
            note: run with `SNFORGE_BACKTRACE=1` environment variable to display a backtrace"
        },
    );
}

#[test]
fn test_backtrace() {
    let temp = setup_package("backtrace_vm_error");

    let output = test_runner(&temp)
        .env("SNFORGE_BACKTRACE", "1")
        .assert()
        .failure();

    assert_stdout_contains(
        output,
        indoc! {
           "
            [WARNING] To get accurate backtrace results, it is required to use the configuration available in the latest Cairo version. For more details, please visit https://foundry-rs.github.io/starknet-foundry/snforge-advanced-features/backtrace.html
            [FAIL] backtrace_vm_error::Test::test_unwrapped_call_contract_syscall
            
            Failure data:
            Got an exception while executing a hint: Requested contract address 0x0000000000000000000000000000000000000000000000000000000000000123 is not deployed.

            error occurred in contract 'InnerContract'
            stack backtrace:
               0: (inlined) backtrace_vm_error::InnerContract::inner_call
                   at [..]lib.cairo:48:9
               1: (inlined) backtrace_vm_error::InnerContract::InnerContract::inner
                   at [..]lib.cairo:38:13
               2: backtrace_vm_error::InnerContract::__wrapper__InnerContract__inner
                   at [..]lib.cairo:37:9

            error occurred in contract 'OuterContract'
            stack backtrace:
               0: (inlined) backtrace_vm_error::IInnerContractDispatcherImpl::inner
                   at [..]lib.cairo:22:1
               1: (inlined) backtrace_vm_error::OuterContract::OuterContract::outer
                   at [..]lib.cairo:17:13
               2: backtrace_vm_error::OuterContract::__wrapper__OuterContract__outer
                   at [..]lib.cairo:15:9
            
            [FAIL] backtrace_vm_error::Test::test_fork_unwrapped_call_contract_syscall
            
            Failure data:
            Got an exception while executing a hint: Requested contract address 0x0000000000000000000000000000000000000000000000000000000000000123 is not deployed.

            error occurred in forked contract with class hash: 0x1a92e0ec431585e5c19b98679e582ebc07d43681ba1cc9c55dcb5ba0ce721a1
            
            error occurred in contract 'OuterContract'
            stack backtrace:
               0: (inlined) backtrace_vm_error::IInnerContractDispatcherImpl::inner
                   at [..]lib.cairo:22:1
               1: (inlined) backtrace_vm_error::OuterContract::OuterContract::outer
                   at [..]lib.cairo:17:13
               2: backtrace_vm_error::OuterContract::__wrapper__OuterContract__outer
                   at [..]lib.cairo:15:9
            "
        },
    );
}

#[test]
fn test_backtrace_without_inlines() {
    let temp = setup_package("backtrace_vm_error");
    without_inlines(&temp);

    let output = test_runner(&temp)
        .env("SNFORGE_BACKTRACE", "1")
        .assert()
        .failure();

    assert_stdout_contains(
        output,
        indoc! {
           "[FAIL] backtrace_vm_error::Test::test_unwrapped_call_contract_syscall
            
            Failure data:
            Got an exception while executing a hint: Requested contract address 0x0000000000000000000000000000000000000000000000000000000000000123 is not deployed.

            error occurred in contract 'InnerContract'
            stack backtrace:
               0: backtrace_vm_error::InnerContract::inner_call
                   at [..]lib.cairo:48:9
               1: backtrace_vm_error::InnerContract::InnerContract::inner
                   at [..]lib.cairo:38:13
               2: backtrace_vm_error::InnerContract::__wrapper__InnerContract__inner
                   at [..]lib.cairo:37:9
            
            error occurred in contract 'OuterContract'
            stack backtrace:
               0: backtrace_vm_error::IInnerContractDispatcherImpl::inner
                   at [..]lib.cairo:22:1
               1: backtrace_vm_error::OuterContract::OuterContract::outer
                   at [..]lib.cairo:17:13
               2: backtrace_vm_error::OuterContract::__wrapper__OuterContract__outer
                   at [..]lib.cairo:15:9
            
            [FAIL] backtrace_vm_error::Test::test_fork_unwrapped_call_contract_syscall
            
            Failure data:
            Got an exception while executing a hint: Requested contract address 0x0000000000000000000000000000000000000000000000000000000000000123 is not deployed.

            error occurred in forked contract with class hash: 0x1a92e0ec431585e5c19b98679e582ebc07d43681ba1cc9c55dcb5ba0ce721a1
            
            error occurred in contract 'OuterContract'
            stack backtrace:
               0: backtrace_vm_error::IInnerContractDispatcherImpl::inner
                   at [..]lib.cairo:22:1
               1: backtrace_vm_error::OuterContract::OuterContract::outer
                   at [..]lib.cairo:17:13
               2: backtrace_vm_error::OuterContract::__wrapper__OuterContract__outer
                   at [..]lib.cairo:15:9
            "
        },
    );
}

#[test]
fn test_wrong_scarb_toml_configuration() {
    let temp = setup_package("backtrace_vm_error");

    let manifest_path = temp.child("Scarb.toml");

    let mut scarb_toml = fs::read_to_string(&manifest_path)
        .unwrap()
        .parse::<DocumentMut>()
        .unwrap();

    scarb_toml["profile"]["dev"]["cairo"]["unstable-add-statements-code-locations-debug-info"] =
        value(false);

    manifest_path.write_str(&scarb_toml.to_string()).unwrap();

    let output = test_runner(&temp)
        .env("SNFORGE_BACKTRACE", "1")
        .assert()
        .failure();

    assert_stdout_contains(
        output,
        indoc! {
           "Failure data:
            Got an exception while executing a hint: Requested contract address 0x0000000000000000000000000000000000000000000000000000000000000123 is not deployed.
            failed to create backtrace: perhaps the contract was compiled without the following entry in Scarb.toml under [profile.dev.cairo]:
            unstable-add-statements-code-locations-debug-info = true

            or scarb version is less than 2.8.0"
        },
    );
}

#[test]
fn test_backtrace_panic() {
    let temp = setup_package("backtrace_panic");

    let output = test_runner(&temp)
        .env("SNFORGE_BACKTRACE", "1")
        .assert()
        .failure();

    if use_snforge_std_compatibility() {
        assert_stdout_contains(
            output,
            indoc! {
               "[FAIL] backtrace_panic::Test::test_contract_panics

                Failure data:
                    0x417373657274206661696c6564 ('Assert failed')

                error occurred in contract 'InnerContract'
                stack backtrace:
                   0: backtrace_panic::InnerContract::__wrapper__InnerContract__inner
                       at [..]lib.cairo:34:9

                error occurred in contract 'OuterContract'
                stack backtrace:
                   0: backtrace_panic::OuterContract::__wrapper__OuterContract__outer
                       at [..]lib.cairo:15:9

                [FAIL] backtrace_panic::Test::test_fork_contract_panics

                Failure data:
                    0x417373657274206661696c6564 ('Assert failed')

                error occurred in forked contract with class hash: 0x554cb276fb5eb0788344f5431b9a166e2f445d8a91c7aef79d8c77e7eede956

                error occurred in contract 'OuterContract'
                stack backtrace:
                   0: backtrace_panic::OuterContract::__wrapper__OuterContract__outer
                       at [..]lib.cairo:15:9"
            },
        );
    } else {
        assert_stdout_contains(
            output,
            indoc! {
               "Failure data:
                    0x417373657274206661696c6564 ('Assert failed')
                error occurred in contract 'InnerContract'
                stack backtrace:
                   0: core::panic_with_const_felt252
                       at [..]lib.cairo:364:5
                   1: core::panic_with_const_felt252
                       at [..]lib.cairo:364:5
                   2: (inlined) core::Felt252PartialEq::eq
                       at [..]lib.cairo:231:9
                   3: (inlined) backtrace_panic::InnerContract::inner_call
                       at [..]traits.cairo:441:10
                   4: (inlined) backtrace_panic::InnerContract::InnerContract::inner
                       at [..]lib.cairo:40:16
                   5: backtrace_panic::InnerContract::__wrapper__InnerContract__inner
                       at [..]lib.cairo:35:13

                error occurred in contract 'OuterContract'
                stack backtrace:
                   0: backtrace_panic::OuterContract::__wrapper__OuterContract__outer
                       at [..]lib.cairo:15:9"
            },
        );
    }
}

#[test]
fn test_backtrace_panic_without_inlines() {
    let temp = setup_package("backtrace_panic");
    without_inlines(&temp);

    let output = test_runner(&temp)
        .env("SNFORGE_BACKTRACE", "1")
        .assert()
        .failure();

    if use_snforge_std_compatibility() {
        assert_stdout_contains(
            output,
            indoc! {
               "[FAIL] backtrace_panic::Test::test_contract_panics

                Failure data:
                    0x417373657274206661696c6564 ('Assert failed')
                
                error occurred in contract 'InnerContract'
                stack backtrace:
                   0: backtrace_panic::InnerContract::__wrapper__InnerContract__inner
                       at [..]lib.cairo:34:9
                
                error occurred in contract 'OuterContract'
                stack backtrace:
                   0: backtrace_panic::OuterContract::__wrapper__OuterContract__outer
                       at [..]lib.cairo:15:9
                
                [FAIL] backtrace_panic::Test::test_fork_contract_panics
                
                Failure data:
                    0x417373657274206661696c6564 ('Assert failed')
                
                error occurred in forked contract with class hash: 0x554cb276fb5eb0788344f5431b9a166e2f445d8a91c7aef79d8c77e7eede956
                
                error occurred in contract 'OuterContract'
                stack backtrace:
                   0: backtrace_panic::OuterContract::__wrapper__OuterContract__outer
                       at [..]lib.cairo:15:9"
            },
        );
    } else {
        assert_stdout_contains(
            output,
            indoc! {
               "[FAIL] backtrace_panic::Test::test_contract_panics

                Failure data:
                    0x417373657274206661696c6564 ('Assert failed')
                
                error occurred in contract 'InnerContract'
                stack backtrace:
                   0: core::array_inline_macro
                       at [..]lib.cairo:350:11
                   1: core::assert
                       at [..]lib.cairo:377:9
                   2: backtrace_panic::InnerContract::inner_call
                       at [..]lib.cairo:40:9
                   3: backtrace_panic::InnerContract::unsafe_new_contract_state
                       at [..]lib.cairo:29:5
                   4: backtrace_panic::InnerContract::__wrapper__InnerContract__inner
                       at [..]lib.cairo:34:9

                error occurred in contract 'OuterContract'
                stack backtrace:
                   0: core::starknet::SyscallResultTraitImpl::unwrap_syscall
                       at [..]starknet.cairo:135:52
                   1: backtrace_panic::IInnerContractDispatcherImpl::inner
                       at [..]lib.cairo:22:1
                   2: backtrace_panic::OuterContract::OuterContract::outer
                       at [..]lib.cairo:17:13
                   3: backtrace_panic::OuterContract::__wrapper__OuterContract__outer
                       at [..]lib.cairo:15:9
                
                [FAIL] backtrace_panic::Test::test_fork_contract_panics
                
                Failure data:
                    0x417373657274206661696c6564 ('Assert failed')
                
                error occurred in forked contract with class hash: 0x554cb276fb5eb0788344f5431b9a166e2f445d8a91c7aef79d8c77e7eede956
                
                error occurred in contract 'OuterContract'
                stack backtrace:
                   0: core::starknet::SyscallResultTraitImpl::unwrap_syscall
                       at [..]starknet.cairo:135:52
                   1: backtrace_panic::IInnerContractDispatcherImpl::inner
                       at [..]lib.cairo:22:1
                   2: backtrace_panic::OuterContract::OuterContract::outer
                       at [..]lib.cairo:17:13
                   3: backtrace_panic::OuterContract::__wrapper__OuterContract__outer
                       at [..]lib.cairo:15:9"
            },
        );
    }
}

#[test]
fn test_handled_error_not_display() {
    let temp = setup_package("dispatchers");

    let output = test_runner(&temp)
        .arg("test_handle_and_panic")
        .env("SNFORGE_BACKTRACE", "1")
        .assert()
        .success();

    // Error from the `FailableContract` should not appear in the output
    assert!(
        !output
            .as_stdout()
            .contains("error occurred in contract 'FailableContract'")
    );

    if use_snforge_std_compatibility() {
        assert_stdout_contains(
            output,
            indoc! {"
            error occurred in contract 'ErrorHandler'
            stack backtrace:
               0: dispatchers::error_handler::ErrorHandler::__wrapper__ErrorHandler__catch_panic_and_fail
        "},
        );
    } else {
        assert_stdout_contains(
            output,
            indoc! {"
            error occurred in contract 'ErrorHandler'
            stack backtrace:
               0: core::panic_with_const_felt252
                   at [..]lib.cairo:364:5
               1: core::panic_with_const_felt252
                   at [..]lib.cairo:364:5
               2: dispatchers::error_handler::ErrorHandler::ErrorHandler::catch_panic_and_fail
                   at [..]error_handler.cairo:50:21
               3: dispatchers::error_handler::ErrorHandler::__wrapper__ErrorHandler__catch_panic_and_fail
                   at [..]error_handler.cairo:42:9
        "},
        );
    }
}

fn without_inlines(temp_dir: &TempDir) {
    let manifest_path = temp_dir.child("Scarb.toml");

    let mut scarb_toml = fs::read_to_string(&manifest_path)
        .unwrap()
        .parse::<DocumentMut>()
        .unwrap();

    scarb_toml["profile"]["dev"]["cairo"]["inlining-strategy"] = value("avoid");

    manifest_path.write_str(&scarb_toml.to_string()).unwrap();
}
