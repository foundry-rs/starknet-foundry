use std::{fs, process::Stdio};

use super::common::runner::runner;
use assert_fs::prelude::PathChild;
use camino::Utf8PathBuf;
use clap::ValueEnum;
use forge::Template;
use project_root::get_project_root;
use scarb_api::ScarbCommand;
use test_case::test_case;

fn find_all_matches_recursively(root_dir_path: Utf8PathBuf, file_name: &str) -> Vec<Utf8PathBuf> {
    let mut matches = Vec::new();
    let paths = std::fs::read_dir(root_dir_path).unwrap();

    for path in paths {
        let path = path.unwrap().path();
        if path.is_dir() {
            matches.append(&mut find_all_matches_recursively(
                Utf8PathBuf::from(path.to_string_lossy().to_string()),
                file_name,
            ));
        } else if path.to_string_lossy().contains(file_name) {
            matches.push(Utf8PathBuf::from(path.to_string_lossy().to_string()));
        }
    }

    matches
}

fn find_all_cairo_packages_paths() -> Vec<Utf8PathBuf> {
    let project_root = get_project_root().expect("Failed to get project root");
    let project_root =
        Utf8PathBuf::from_path_buf(project_root).expect("Failed to convert to Utf8PathBuf");
    let manifests_paths = find_all_matches_recursively(project_root, "Scarb.toml");
    let cairo_packages_paths = manifests_paths
        .iter()
        .map(|manifest_path| {
            let manifest_path = Utf8PathBuf::from(manifest_path);
            manifest_path
                .parent()
                .map_or_else(|| manifest_path.clone(), camino::Utf8Path::to_path_buf)
        })
        .collect::<Vec<_>>();

    cairo_packages_paths
}

#[test]
fn check_all_packages_formatting() {
    let cairo_packages_paths = find_all_cairo_packages_paths();

    for path in cairo_packages_paths {
        println!("Running `scarb fmt --check` in directory: {path}");
        let output = ScarbCommand::new()
            .current_dir(path)
            .arg("fmt")
            .arg("--check")
            .command()
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .expect("Failed to run `scarb fmt --check`");

        assert!(output.status.success(), "`scarb fmt --check` failed");
    }
}

fn check_and_lint(package_path: Utf8PathBuf) {
    println!("Running `scarb check` in directory {package_path}");
    let check_output = ScarbCommand::new()
        .current_dir(&package_path)
        .arg("check")
        .command()
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .expect("Failed to run `scarb check`");
    assert!(check_output.status.success(), "`scarb check` failed");

    // TODO(#3149)
    if cfg!(feature = "scarb_since_2_10") {
        println!("Running `scarb lint` in directory {package_path}");
        let lint_output = ScarbCommand::new()
            .current_dir(package_path)
            .arg("lint")
            .command()
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .expect("Failed to run `scarb lint`");
        assert!(lint_output.status.success(), "`scarb lint` failed");

        // TODO(#3148): Once `scarb lint` can change warning to error, we should check status instead of checking if stdout is not empty
        // ATM `scarb lint` returns 0 even if there are warnings
        assert!(
            lint_output.stdout.is_empty(),
            "`scarb lint` output should be empty"
        );
    }
}

#[test]
fn validate_forge_templates() {
    let temp_dir = assert_fs::TempDir::new().expect("Unable to create a temporary directory");

    let templates = Template::value_variants();

    for template in templates {
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

        // Overwrite Scarb.toml with `allow-warnings = true`
        let scarb_toml_path = package_path.join("Scarb.toml");
        let mut scarb_toml = fs::read_to_string(&scarb_toml_path).unwrap();
        scarb_toml.push_str("\n[cairo]\nallow-warnings = true\n");
        fs::write(&scarb_toml_path, &scarb_toml).expect("Failed to write to Scarb.toml");

        check_and_lint(package_path);
    }
}

#[test_case("snforge_std")]
#[test_case("sncast_std")]
fn validate_libs(package_name: &str) {
    let package_path = get_project_root()
        .expect("Failed to get project root")
        .join(package_name);
    let package_path = package_path
        .canonicalize()
        .expect("Failed to canonicalize path");
    let package_path =
        Utf8PathBuf::from_path_buf(package_path).expect("Failed to convert to Utf8PathBuf");

    check_and_lint(package_path);
}
