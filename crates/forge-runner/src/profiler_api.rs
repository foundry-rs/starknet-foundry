use anyhow::{anyhow, Context, Result};
use std::process::Stdio;
use std::{env, fs, path::PathBuf, process::Command};

pub const PROFILE_DIR: &str = "profile";

pub fn run_profiler(test_name: &String, trace_path: &PathBuf) -> Result<()> {
    let profiler = env::var("CAIRO_PROFILER")
        .map(PathBuf::from)
        .ok()
        .unwrap_or_else(|| PathBuf::from("cairo-profiler"));
    let dir_to_save_profile = PathBuf::from(PROFILE_DIR);
    fs::create_dir_all(&dir_to_save_profile).context("Failed to create a profile dir")?;
    let path_to_save_profile = dir_to_save_profile.join(format!("{test_name}.pb.gz"));

    let mut cmd = Command::new(profiler);
    cmd.arg(trace_path);
    cmd.arg("--output-path");
    cmd.arg(&path_to_save_profile);
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());

    let exit_status = cmd.output().context("Failed to run cairo-profiler")?.status;

    if exit_status.success() {
        Ok(())
    } else {
        Err(anyhow!(
            "cairo-profiler failed to generate the profile for test {test_name} - inspect the errors above for more info"
        ))
    }
}
