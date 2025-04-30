use anyhow::{Result, bail};
use camino::Utf8PathBuf;
use clap::Args;
use sncast::{
    helpers::constants::DEFAULT_MULTICALL_CONTENTS, response::structs::MulticallNewResponse,
};

#[derive(Args, Debug)]
#[command(about = "Generate a template for the multicall .toml file", long_about = None)]
pub struct New {
    /// Output path to the file where the template is going to be saved
    #[arg(required = true, num_args = 1)]
    pub output_path: Option<Utf8PathBuf>,

    /// If the file specified in output-path exists, this flag decides if it is going to be overwritten
    #[arg(short = 'o', long = "overwrite")]
    pub overwrite: bool,
}

pub fn write_empty_template(
    output_path: &Utf8PathBuf,
    overwrite: bool,
) -> Result<MulticallNewResponse> {
    if output_path.exists() {
        if !output_path.is_file() {
            bail!("Output file cannot be a directory");
        }

        if !overwrite {
            bail!(
                "Output file already exists, if you want to overwrite it, use the `--overwrite` flag"
            );
        }
    }

    std::fs::write(output_path.clone(), DEFAULT_MULTICALL_CONTENTS)?;

    Ok(MulticallNewResponse {
        path: output_path.clone(),
        content: DEFAULT_MULTICALL_CONTENTS.to_string(),
    })
}
