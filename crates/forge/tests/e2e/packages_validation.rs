use std::{fs, process::Stdio};

use super::common::runner::runner;
use assert_fs::prelude::{FileWriteStr, PathChild};
use clap::ValueEnum;
use forge::Template;
use scarb_api::ScarbCommand;

fn find_all_matches_recursively(root_dir_path: &str, file_name: &str) -> Vec<String> {
    let mut matches = Vec::new();
    let paths = std::fs::read_dir(root_dir_path).unwrap();

    for path in paths {
        let path = path.unwrap().path();
        if path.is_dir() {
            matches.append(&mut find_all_matches_recursively(
                &path.to_string_lossy(),
                file_name,
            ));
        } else if path.to_string_lossy().contains(file_name) {
            matches.push(path.to_string_lossy().to_string());
        }
    }

    matches
}

fn find_all_cairo_packages_paths() -> Vec<String> {
    let _ = std::env::set_current_dir("../..");
    let manifests_paths = find_all_matches_recursively("crates", "Scarb.toml");
    let cairo_packages_paths = manifests_paths
        .iter()
        .map(|manifest_path| {
            manifest_path.split("/Scarb.toml").collect::<Vec<&str>>()[0].to_string()
        })
        .collect::<Vec<String>>();

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

// // TODO(#3149): Lint checking can be done together with formatting, once minimal supported version of scarb is >= 2.10.0
// #[test]
// #[cfg_attr(not(feature = "scarb_since_2_10"), ignore)]
// fn check_packages_linting() {
//     let skipped_packages = [
//         "backtrace_panic", // This package has code which results in compiler (not lint) warnings
//         "build_fails",     // This package fails to compile on purpose
//         "missing_field",   // This package fails to compile on purpose
//         "trace_resources", // This package has code which results in compiler (not lint) warnings
//         "coverage_project", // This package has code which results in compiler (not lint) warnings
//         "snforge-scarb-plugin",
//     ];

//     let cairo_packages_paths = find_all_cairo_packages_paths();
//     let cairo_packages_paths = cairo_packages_paths
//         .iter()
//         .filter(|path| !skipped_packages.contains(&path.split("/").last().unwrap()))
//         .map(|path| path.to_string())
//         .collect::<Vec<String>>();

//     for path in cairo_packages_paths {
//         println!("Running `scarb lint` in directory: {}", path);
//         let output = ScarbCommand::new()
//             .current_dir(path)
//             .arg("lint")
//             .command()
//             .output()
//             .expect("Failed to run `scarb lint`");

//         assert_lint_output(output);
//     }
// }

#[test]
fn check_forge_templates() {
    let temp_dir = assert_fs::TempDir::new().expect("Unable to create a temporary directory");

    let templates = Template::value_variants();

    for template in templates {
        let package_name = format!("{}_test", template.to_string().replace("`-`", "`_`"));

        runner(&temp_dir)
            .args([
                "new",
                "--template",
                template.to_string().as_str(),
                &package_name,
            ])
            .assert()
            .success();

        // Overwrite Scarb.toml with `allow-warnings = true`
        let scarb_toml_path = temp_dir.child(&package_name).child("Scarb.toml");
        let mut scarb_toml = fs::read_to_string(&scarb_toml_path).unwrap();
        scarb_toml.push_str("\n[cairo]\nallow-warnings = true\n");
        scarb_toml_path.write_str(&scarb_toml).unwrap();

        println!("Running `scarb check` in directory {package_name}");
        let check_output = ScarbCommand::new()
            .current_dir(temp_dir.child(&package_name).path())
            .arg("check")
            .command()
            .output()
            .expect("Failed to run `scarb check`");
        assert!(check_output.status.success(), "`scarb check` failed");

        // TODO(#3149)
        if cfg!(feature = "scarb_since_2_10") {
            println!("Running `scarb lint` in directory {package_name}");
            let lint_output = ScarbCommand::new()
                .current_dir(temp_dir.child(package_name).path())
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
}
