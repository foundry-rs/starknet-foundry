use anyhow::{Context, Result};
use shared::command::CommandExt;
use std::ffi::OsString;
use std::process::Stdio;
use std::{env, fs, path::PathBuf, process::Command};

pub const PROFILE_DIR: &str = "profile";

pub fn run_profiler(
    test_name: &str,
    trace_path: &PathBuf,
    profiler_args: &[OsString],
) -> Result<()> {
    let profiler = env::var("CAIRO_PROFILER")
        .map(PathBuf::from)
        .ok()
        .unwrap_or_else(|| PathBuf::from("cairo-profiler"));

    let mut command = Command::new(profiler);

    if profiler_args.iter().all(|arg| arg != "--output-path") {
        let dir_to_save_profile = PathBuf::from(PROFILE_DIR);
        fs::create_dir_all(&dir_to_save_profile).context("Failed to create a profile dir")?;
        let path_to_save_profile = dir_to_save_profile.join(format!("{test_name}.pb.gz"));

        command.arg("--output-path").arg(&path_to_save_profile);
    }

    command
        .arg(trace_path)
        .args(profiler_args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output_checked()
        .with_context(||format!("cairo-profiler failed to generate the profile for test {test_name} - inspect the errors above for more info"))?;

    Ok(())
}
