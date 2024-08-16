use crate::build_trace_data::TRACE_DIR;
use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use shared::command::CommandExt;
use std::process::Stdio;
use std::{env, fs, path::PathBuf, process::Command};

pub const COVERAGE_DIR: &str = "coverage";
pub const OUTPUT_FILE_NAME: &str = "coverage.lcov";

pub fn run_coverage() -> Result<()> {
    let coverage = env::var("CAIRO_COVERAGE")
        .map(PathBuf::from)
        .ok()
        .unwrap_or_else(|| PathBuf::from("cairo-coverage"));
    let dir_to_save_coverage = PathBuf::from(COVERAGE_DIR);
    fs::create_dir_all(&dir_to_save_coverage).context("Failed to create a coverage dir")?;

    let path_to_save_coverage = dir_to_save_coverage.join(OUTPUT_FILE_NAME);
    let trace_files = get_all_json_files_from_trace_dir()?;

    Command::new(coverage)
        .arg("--output-path")
        .arg(&path_to_save_coverage)
        .args(trace_files)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output_checked()
        .with_context(|| {
            "cairo-coverage failed to generate coverage - inspect the errors above for more info"
                .to_string()
        })?;

    Ok(())
}

fn get_all_json_files_from_trace_dir() -> Result<Vec<String>> {
    let trace_dir = Utf8PathBuf::from(TRACE_DIR);

    let mut trace_files = Vec::new();
    for entry in fs::read_dir(&trace_dir)? {
        let trace_file_path = Utf8PathBuf::from_path_buf(entry?.path())
            .expect("Failed to convert trace file path to Utf8PathBuf");

        if trace_file_path.extension() == Some("json") {
            trace_files.push(trace_file_path.to_string());
        }
    }

    Ok(trace_files)
}
