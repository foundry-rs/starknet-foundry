use super::common::runner::runner;
use assert_fs::prelude::PathChild;
use camino::Utf8PathBuf;
use forge::Template;
use packages_validation::check_and_lint;
use scarb_api::ScarbCommand;
use std::fs;
use std::process::Stdio;
use test_case::test_case;
use toml_edit::DocumentMut;

#[test_case(&Template::CairoProgram; "cairo-program")]
#[test_case(&Template::BalanceContract; "balance-contract")]
#[test_case(&Template::Erc20Contract; "erc20-contract")]
fn validate_templates(template: &Template) {
    let temp_dir = assert_fs::TempDir::new().expect("Unable to create a temporary directory");
    let package_name = format!("{}_test", template.to_string().replace('-', "_"));
    let snforge_std = Utf8PathBuf::from("../../snforge_std")
        .canonicalize_utf8()
        .unwrap();

    runner(&temp_dir)
        .env("DEV_DISABLE_SNFORGE_STD_DEPENDENCY", "true")
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

    let scarb_add = ScarbCommand::new()
        .current_dir(&package_path)
        .args([
            "add",
            "snforge_std",
            "--dev",
            "--path",
            snforge_std.as_str(),
        ])
        .command()
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Failed to run scarb add")
        .wait()
        .expect("Failed to wait for scarb add");
    assert!(scarb_add.success(), "Failed to add snforge_std to package");

    // Overwrite Scarb.toml with `allow-warnings = false`
    let scarb_toml_path = package_path.join("Scarb.toml");
    let mut scarb_toml = fs::read_to_string(&scarb_toml_path)
        .unwrap()
        .parse::<DocumentMut>()
        .unwrap();
    scarb_toml["cairo"]["allow-warnings"] = toml_edit::value(false);
    fs::write(&scarb_toml_path, scarb_toml.to_string()).expect("Failed to write to Scarb.toml");

    check_and_lint(&package_path);
}
