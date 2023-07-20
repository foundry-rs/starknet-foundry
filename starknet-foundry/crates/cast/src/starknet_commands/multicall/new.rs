use crate::helpers::constants::DEFAULT_MULTICALL_CONTENTS;
use anyhow::{bail, Result};
use camino::Utf8PathBuf;

pub fn new(maybe_output_path: Option<Utf8PathBuf>, overwrite: bool) -> Result<String> {
    if let Some(output_path) = maybe_output_path {
        if output_path.exists() {
            if !output_path.is_file() {
                bail!("output file cannot be a directory");
            }
            if !overwrite {
                bail!(
                  "output file already exists, if you want to overwrite it, use the `overwrite` flag"
              );
            }
        }
        std::fs::write(output_path.clone(), DEFAULT_MULTICALL_CONTENTS)?;
        return Ok(format!(
            "Multicall template successfully saved in {output_path}"
        ));
    }
    Ok(DEFAULT_MULTICALL_CONTENTS.to_string())
}

pub fn print_new_result(new_result: &str) {
    println!("{new_result}");
}
