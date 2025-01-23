use anyhow::Result;
use camino::Utf8Path;
use shared::spinner::Spinner;
use std::env;

pub fn spawn_usc_spinner(sierra_file_path: &Utf8Path) -> Result<Spinner> {
    // Skip printing path when compiling unsaved sierra
    // which occurs during test execution for some cheatcodes e.g. `replace_bytecode`
    let message = if is_temp_file(sierra_file_path)? {
        "Compiling Sierra to Casm".to_string()
    } else {
        format!(
            "Compiling Sierra to Casm ({})",
            sierra_file_path.canonicalize_utf8()?
        )
    };
    let spinner = Spinner::create_with_message(message);

    Ok(spinner)
}

fn is_temp_file(file_path: &Utf8Path) -> Result<bool> {
    let temp_dir = env::temp_dir().canonicalize()?;
    Ok(file_path.canonicalize()?.starts_with(temp_dir))
}
