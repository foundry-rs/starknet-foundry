use anyhow::{ensure, Context, Result};
use indoc::{formatdoc, indoc};
use scarb_api::metadata::Metadata;
use semver::Version;
use shared::command::CommandExt;
use std::ffi::OsString;
use std::process::Stdio;
use std::{env, fs, path::PathBuf, process::Command};
use toml_edit::{DocumentMut, Table};
use which::which;

pub const COVERAGE_DIR: &str = "coverage";
pub const OUTPUT_FILE_NAME: &str = "coverage.lcov";

const MINIMAL_SCARB_VERSION: Version = Version::new(2, 8, 0);

const CAIRO_COVERAGE_REQUIRED_ENTRIES: [(&str, &str); 3] = [
    ("unstable-add-statements-functions-debug-info", "true"),
    ("unstable-add-statements-code-locations-debug-info", "true"),
    ("inlining-strategy", "avoid"),
];

pub fn run_coverage(saved_trace_data_paths: &[PathBuf], coverage_args: &[OsString]) -> Result<()> {
    let coverage = env::var("CAIRO_COVERAGE")
        .map(PathBuf::from)
        .ok()
        .unwrap_or_else(|| PathBuf::from("cairo-coverage"));

    ensure!(
        which(coverage.as_os_str()).is_ok(),
        indoc! {
            r"The 'cairo-coverage' binary was not found in PATH. It may not have been installed.
            Please refer to the documentation for installation instructions:
            https://github.com/software-mansion/cairo-coverage/blob/main/README.md"
        }
    );

    let trace_files: Vec<&str> = saved_trace_data_paths
        .iter()
        .map(|trace_data_path| {
            trace_data_path
                .to_str()
                .expect("Failed to convert trace data path to string")
        })
        .collect();

    let mut command = Command::new(coverage);

    if coverage_args.iter().all(|arg| arg != "--output-path") {
        let dir_to_save_coverage = PathBuf::from(COVERAGE_DIR);
        fs::create_dir_all(&dir_to_save_coverage).context("Failed to create a coverage dir")?;
        let path_to_save_coverage = dir_to_save_coverage.join(OUTPUT_FILE_NAME);

        command.arg("--output-path").arg(&path_to_save_coverage);
    }

    command
        .args(trace_files)
        .args(coverage_args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output_checked()
        .with_context(|| {
            "cairo-coverage failed to generate coverage - inspect the errors above for more info"
        })?;

    Ok(())
}

pub fn can_coverage_be_generated(scarb_metadata: &Metadata) -> Result<()> {
    let manifest = fs::read_to_string(&scarb_metadata.runtime_manifest)?.parse::<DocumentMut>()?;

    ensure!(
        scarb_metadata.app_version_info.version >= MINIMAL_SCARB_VERSION,
        "Coverage generation requires scarb version >= {MINIMAL_SCARB_VERSION}",
    );

    let has_needed_entries = manifest
        .get("profile")
        .and_then(|profile| profile.get(&scarb_metadata.current_profile))
        .and_then(|profile| profile.get("cairo"))
        .and_then(|cairo| cairo.as_table())
        .is_some_and(|profile_cairo| {
            CAIRO_COVERAGE_REQUIRED_ENTRIES
                .iter()
                .all(|(key, value)| contains_entry_with_value(profile_cairo, key, value))
        });

    ensure!(
        has_needed_entries,
        formatdoc! {
            "Scarb.toml must have the following Cairo compiler configuration to run coverage:

            [profile.{profile}.cairo]
            unstable-add-statements-functions-debug-info = true
            unstable-add-statements-code-locations-debug-info = true
            inlining-strategy = \"avoid\"
            ... other entries ...
            ",
            profile = scarb_metadata.current_profile
        },
    );

    Ok(())
}

/// Check if the table contains an entry with the given key and value.
/// Accepts only bool and string values.
fn contains_entry_with_value(table: &Table, key: &str, value: &str) -> bool {
    table.get(key).is_some_and(|entry| {
        if let Some(entry) = entry.as_bool() {
            entry.to_string() == value
        } else if let Some(entry) = entry.as_str() {
            entry == value
        } else {
            false
        }
    })
}
