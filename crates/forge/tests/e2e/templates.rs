use std::fs;

use super::common::runner::runner;
use assert_fs::prelude::PathChild;
use camino::Utf8PathBuf;
use forge::Template;
use packages_validation::check_and_lint;
use test_case::test_case;

#[test_case(&Template::CairoProgram; "cairo-program")]
#[test_case(&Template::BalanceContract; "balance-contract")]
#[test_case(&Template::Erc20Contract; "erc20-contract")]
fn validate_templates(template: &Template) {
    let temp_dir = assert_fs::TempDir::new().expect("Unable to create a temporary directory");
    let package_name = format!("{}_test", template.to_string().replace('-', "_"));

    runner(&temp_dir)
        .args([
            "new",
            "--template",
            template.to_string().as_str(),
            &package_name,
        ])
        .assert()
        .success();

    let package_path = temp_dir.child(package_name);
    let package_path = Utf8PathBuf::from_path_buf(package_path.to_path_buf())
        .expect("Failed to convert to Utf8PathBuf");

    // Overwrite Scarb.toml with `allow-warnings = false`
    let scarb_toml_path = package_path.join("Scarb.toml");
    let mut scarb_toml = fs::read_to_string(&scarb_toml_path).unwrap();
    scarb_toml.push_str("\n[cairo]\nallow-warnings = false\n");
    fs::write(&scarb_toml_path, &scarb_toml).expect("Failed to write to Scarb.toml");

    check_and_lint(&package_path);
}
