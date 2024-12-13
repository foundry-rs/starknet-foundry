use super::common::runner::{setup_package, test_runner};
use assert_fs::fixture::{FileWriteStr, PathChild};
use indoc::indoc;
use shared::test_utils::output_assert::assert_stdout_contains;
use std::fs;
use toml_edit::{value, DocumentMut};

#[test]
#[cfg_attr(not(feature = "scarb_2_8_3"), ignore)]
fn test_backtrace_missing_env() {
    let temp = setup_package("backtrace_vm_error");

    let output = test_runner(&temp).assert().failure();

    assert_stdout_contains(
        output,
        indoc! {
           "Failure data:
                (0x454e545259504f494e545f4e4f545f464f554e44 ('ENTRYPOINT_NOT_FOUND'), 0x454e545259504f494e545f4641494c4544 ('ENTRYPOINT_FAILED'))
            note: run with `SNFORGE_BACKTRACE=1` environment variable to display a backtrace"
        },
    );
}

#[test]
#[cfg_attr(not(feature = "scarb_2_8_3"), ignore)]
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
                (0x454e545259504f494e545f4e4f545f464f554e44 ('ENTRYPOINT_NOT_FOUND'), 0x454e545259504f494e545f4641494c4544 ('ENTRYPOINT_FAILED'))
            error occurred in contract 'InnerContract' at pc: '72'
            stack backtrace:
               0: backtrace_vm_error::InnerContract::inner_call
                   at [..]/src/lib.cairo:47:9
               1: backtrace_vm_error::InnerContract::InnerContract::inner
                   at [..]/src/lib.cairo:38:13
               2: backtrace_vm_error::InnerContract::__wrapper__InnerContract__inner
                   at [..]/src/lib.cairo:37:9

            error occurred in contract 'OuterContract' at pc: '107'
            stack backtrace:
               0: backtrace_vm_error::IInnerContractDispatcherImpl::inner
                   at [..]/src/lib.cairo:22:1
               1: backtrace_vm_error::OuterContract::OuterContract::outer
                   at [..]/src/lib.cairo:17:13
               2: backtrace_vm_error::OuterContract::__wrapper__OuterContract__outer
                   at [..]/src/lib.cairo:15:9"
        },
    );
}

#[test]
#[cfg_attr(not(feature = "scarb_2_8_3"), ignore)]
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
                (0x454e545259504f494e545f4e4f545f464f554e44 ('ENTRYPOINT_NOT_FOUND'), 0x454e545259504f494e545f4641494c4544 ('ENTRYPOINT_FAILED'))
            failed to create backtrace: perhaps the contract was compiled without the following entry in Scarb.toml under [profile.dev.cairo]:
            unstable-add-statements-code-locations-debug-info = true

            or scarb version is less than 2.8.0"
        },
    );
}

#[test]
#[cfg_attr(not(feature = "scarb_2_8_3"), ignore)]
fn test_backtrace_panic() {
    let temp = setup_package("backtrace_panic");

    let output = test_runner(&temp)
        .env("SNFORGE_BACKTRACE", "1")
        .assert()
        .failure();

    assert_stdout_contains(
        output,
        indoc! {
           "Failure data:
                0x61616161 ('aaaa')
            error occurred in contract 'InnerContract' at pc: '70'
            stack backtrace:
               0: backtrace_panic::InnerContract::__wrapper__InnerContract__inner
                   at [..]/src/lib.cairo:34:9

            error occurred in contract 'OuterContract' at pc: '107'
            stack backtrace:
               0: backtrace_panic::IInnerContractDispatcherImpl::inner
                   at [..]/src/lib.cairo:22:1
               1: backtrace_panic::OuterContract::OuterContract::outer
                   at [..]/src/lib.cairo:17:13
               2: backtrace_panic::OuterContract::__wrapper__OuterContract__outer
                   at [..]/src/lib.cairo:15:9"
        },
    );
}
