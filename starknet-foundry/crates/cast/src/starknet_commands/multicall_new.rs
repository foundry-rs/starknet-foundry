use anyhow::Result;
use clap::Args;
use std::path::Path;

static DEFAULT_MULTICALL_CONTENTS: &str = r#"[[call]]
call_type = ""
class_hash = ""
inputs = []
id = ""
unique = false

[[call]]
call_type = ""
contract_address = ""
function = ""
inputs = []
"#;

#[derive(Args)]
#[command(about = "Call a contract instance on Starknet", long_about = None)]
pub struct MulticallNew {
    /// Path to a .toml file containing the multi call specification
    #[clap(short = 'p', long = "output-path")]
    pub output_path: Option<String>,

    #[clap(short = 'o', long = "overwrite")]
    pub overwrite: Option<bool>,
}

pub async fn multicall_new(maybe_output_path: Option<String>, overwrite: bool) -> Result<()> {
    if let Some(output_path) = maybe_output_path {
        let output_path = Path::new(output_path.as_str());
        if !output_path.is_file() {
            anyhow::bail!("invalid output file");
        }
        if output_path.exists() && !overwrite {
            anyhow::bail!("output file already exists, if you want to overwrite it, use the `overwrite` flag");
        }
        std::fs::write(output_path, DEFAULT_MULTICALL_CONTENTS)?;
    } else {
        println!("{DEFAULT_MULTICALL_CONTENTS}");
    }

    Ok(())
}
