use anyhow::Result;
use camino::Utf8Path;
use indicatif::{ProgressBar, ProgressStyle};
use std::env;
use std::time::Duration;

pub fn spawn_spinner_message(sierra_file_path: &Utf8Path) -> Result<ProgressBar> {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(ProgressStyle::with_template("\n{spinner} {msg}\n")?);
    spinner.enable_steady_tick(Duration::from_millis(100));

    // Skip printing path when compiling unsaved sierra
    // which occurs during test execution for some cheatcodes e.g. `replace_bytecode`
    let message = if is_temp_file(sierra_file_path)? {
        "Compiling SIERRA to CASM".to_string()
    } else {
        format!(
            "Compiling SIERRA to CASM ({})",
            sierra_file_path.canonicalize_utf8()?
        )
    };
    spinner.set_message(message);

    Ok(spinner)
}

fn is_temp_file(file_path: &Utf8Path) -> Result<bool> {
    let temp_dir = env::temp_dir().canonicalize()?;
    Ok(file_path.canonicalize()?.starts_with(temp_dir))
}
