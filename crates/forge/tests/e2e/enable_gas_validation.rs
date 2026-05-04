use super::common::runner::{setup_package, test_runner};
use assert_fs::fixture::{FileWriteStr, PathChild};
use indoc::indoc;
use shared::test_utils::output_assert::assert_stdout_contains;
use std::fs;
use toml_edit::{DocumentMut, value};

#[test]
fn test_disabled_gas_in_scarb_toml() {
    let temp = setup_package("simple_package");

    let manifest_path = temp.child("Scarb.toml");
    let mut scarb_toml = fs::read_to_string(&manifest_path)
        .unwrap()
        .parse::<DocumentMut>()
        .unwrap();
    scarb_toml["cairo"]["enable-gas"] = value(false);
    manifest_path.write_str(&scarb_toml.to_string()).unwrap();

    let output = test_runner(&temp).assert().failure();

    assert_stdout_contains(
        output,
        indoc! {"
        [ERROR] snforge test does not support gas calculation being disabled
        help: enable gas calculation by adding the following entry to [..]Scarb.toml:

        [profile.dev.cairo]
        enable-gas = true
        ... other entries ...
        "},
    );
}
