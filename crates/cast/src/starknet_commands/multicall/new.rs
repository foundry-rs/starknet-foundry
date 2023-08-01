use crate::helpers::constants::DEFAULT_MULTICALL_CONTENTS;
use anyhow::{bail, Result};
use camino::Utf8PathBuf;
use clap::Args;

#[derive(Args, Debug)]
#[command(about = "Declare a contract to starknet", long_about = None)]
pub struct New {
    /// output path to the file where the template is going to be saved
    #[clap(short = 'p', long = "output-path")]
    pub output_path: Option<Utf8PathBuf>,

    /// if the file specified in output-path exists, this flag decides if it is going to be overwritten
    #[clap(short = 'o', long = "overwrite")]
    pub overwrite: Option<bool>,
}

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
