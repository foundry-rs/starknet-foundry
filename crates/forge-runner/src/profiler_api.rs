use anyhow::{Context, Result};
use std::{env, fs, path::PathBuf, process::Command};

pub const PROFILE_DIR: &str = "profile";

pub fn run_profiler(test_name: &String, trace_path: &PathBuf) -> Result<()> {
    let profiler = env::var("CAIRO_PROFILER")
        .map(PathBuf::from)
        .ok()
        .unwrap_or_else(|| PathBuf::from("cairo-profiler"));
    let dir_to_save_profile = PathBuf::from(PROFILE_DIR).join(format!("{test_name}.pb.gz"));
    fs::create_dir_all(&dir_to_save_profile).context("Failed to create a profile dir")?;
    let mut cmd = Command::new(profiler);
    cmd.arg(&trace_path);
    cmd.arg("--output-path");
    cmd.arg(&dir_to_save_profile);
    let _ = cmd.output().context("Failed to run cairo-profiler")?;
    Ok(())
}
