use anyhow::Result;
use std::{env, path::PathBuf, process::Command};

pub const PROFILE_DIR: &str = "profile";


// TODO improve the error
pub fn assert_profiler_available() -> Result<()>  {
    let profiler = env::var("CAIRO_PROFILER").map(PathBuf::from).ok()
        .unwrap_or_else(|| PathBuf::from("cairo-profiler"));
    let mut cmd = Command::new(profiler);
    cmd.arg("-V");
    let _ = cmd.output()?;
    Ok(())
}

pub fn run_profiler(test_name: &String, trace_path: PathBuf) {
    println!("{}", 123);
    let profiler = env::var("CAIRO_PROFILER").map(PathBuf::from).ok()
        .unwrap_or_else(|| PathBuf::from("cairo-profiler"));
    let dir_to_save_profile = PathBuf::from(PROFILE_DIR).join(format!("{test_name}.pb.gz"));
    let mut cmd = Command::new(profiler);
    cmd.arg(trace_path);
    cmd.arg("--output-path");
    cmd.arg(dir_to_save_profile);
}
