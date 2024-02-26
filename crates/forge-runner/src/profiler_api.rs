use anyhow::Result;
use std::{env, path::PathBuf, process::Command};




// TODO improve the error
fn assert_profiler_available() -> Result<()>  {
    let profiler = env::var("CAIRO_PROFILER").map(PathBuf::from).ok()
        .unwrap_or_else(|| PathBuf::from("cairo-profiler"));
    let mut cmd = Command::new(profiler);
    cmd.arg("-V");
    let _ = cmd.output()?;
    Ok(())
}

fn run_profiler(trace: PathBuf, test_name: String) {
    let profiler = env::var("CAIRO_PROFILER").map(PathBuf::from).ok()
    .unwrap_or_else(|| PathBuf::from("cairo-profiler"));
    let mut cmd = Command::new(profiler);
}
