use anyhow::{ensure, Context, Result};
use shared::command::CommandExt;
use std::process::Stdio;
use std::{env, fs, path::PathBuf, process::Command};
use which::which;

pub const COVERAGE_DIR: &str = "coverage";
pub const OUTPUT_FILE_NAME: &str = "coverage.lcov";

pub fn run_coverage(saved_trace_data_paths: &[PathBuf]) -> Result<()> {
    let coverage = env::var("CAIRO_COVERAGE")
        .map(PathBuf::from)
        .ok()
        .unwrap_or_else(|| PathBuf::from("cairo-coverage"));

    ensure!(
        which(coverage.as_os_str()).is_ok(),
        "The 'cairo-coverage' binary was not found in PATH. Perhaps you didn't install it? Check out docs for more info"
    );

    let dir_to_save_coverage = PathBuf::from(COVERAGE_DIR);
    fs::create_dir_all(&dir_to_save_coverage).context("Failed to create a coverage dir")?;
    let path_to_save_coverage = dir_to_save_coverage.join(OUTPUT_FILE_NAME);

    let trace_files: Vec<&str> = saved_trace_data_paths
        .iter()
        .map(|trace_data_path| {
            trace_data_path
                .to_str()
                .expect("Failed to convert trace data path to string")
        })
        .collect();

    Command::new(coverage)
        .arg("--output-path")
        .arg(&path_to_save_coverage)
        .args(trace_files)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output_checked()
        .with_context(|| {
            "cairo-coverage failed to generate coverage - inspect the errors above for more info"
        })?;

    Ok(())
}
