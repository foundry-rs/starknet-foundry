use super::common::runner::{setup_package, test_runner};
use assert_fs::TempDir;
use assert_fs::fixture::{FileWriteStr, PathChild};
use indoc::indoc;
use shared::test_utils::output_assert::assert_stdout_contains;
use std::fs;
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
           "Failure data:
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
                   at [..]lib.cairo:15:9"
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
           "Failure data:
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
                   at [..]lib.cairo:15:9"
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

    if cfg!(feature = "supports-panic-backtrace") {
        assert_stdout_contains(
            output,
            indoc! {
               "Failure data:
                    0x417373657274206661696c6564 ('Assert failed')
                error occurred in contract 'InnerContract'
                stack backtrace:
                   0: (inlined) core::array::ArrayImpl::append
                       at [..]array.cairo:135:9
                   1: core::array_inline_macro
                       at [..]lib.cairo:364:11
                   2: (inlined) core::Felt252PartialEq::eq
                       at [..]lib.cairo:231:9
                   3: (inlined) backtrace_panic::InnerContract::inner_call
                       at [..]traits.cairo:442:10
                   4: (inlined) backtrace_panic::InnerContract::InnerContract::inner
                       at [..]lib.cairo:40:16
                   5: backtrace_panic::InnerContract::__wrapper__InnerContract__inner
                       at [..]lib.cairo:35:13

                error occurred in contract 'OuterContract'
                stack backtrace:
                   0: (inlined) backtrace_panic::IInnerContractDispatcherImpl::inner
                       at [..]lib.cairo:22:1
                   1: (inlined) backtrace_panic::OuterContract::OuterContract::outer
                       at [..]lib.cairo:17:13
                   2: backtrace_panic::OuterContract::__wrapper__OuterContract__outer
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
                   0: backtrace_panic::InnerContract::__wrapper__InnerContract__inner
                       at [..]lib.cairo:34:9

                error occurred in contract 'OuterContract'
                stack backtrace:
                   0: (inlined) backtrace_panic::IInnerContractDispatcherImpl::inner
                       at [..]lib.cairo:22:1
                   1: (inlined) backtrace_panic::OuterContract::OuterContract::outer
                       at [..]lib.cairo:17:13
                   2: backtrace_panic::OuterContract::__wrapper__OuterContract__outer
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

    if cfg!(feature = "supports-panic-backtrace") {
        assert_stdout_contains(
            output,
            indoc! {
               "Failure data:
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
                   0: backtrace_panic::IInnerContractDispatcherImpl::inner
                       at [..]lib.cairo:22:1
                   1: backtrace_panic::OuterContract::OuterContract::outer
                       at [..]lib.cairo:17:13
                   2: backtrace_panic::OuterContract::__wrapper__OuterContract__outer
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
                   0: backtrace_panic::InnerContract::__wrapper__InnerContract__inner
                       at [..]lib.cairo:34:9

                error occurred in contract 'OuterContract'
                stack backtrace:
                   0: backtrace_panic::IInnerContractDispatcherImpl::inner
                       at [..]lib.cairo:22:1
                   1: backtrace_panic::OuterContract::OuterContract::outer
                       at [..]lib.cairo:17:13
                   2: backtrace_panic::OuterContract::__wrapper__OuterContract__outer
                       at [..]lib.cairo:15:9"
            },
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
