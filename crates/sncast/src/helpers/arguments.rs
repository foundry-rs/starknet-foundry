use anyhow::{Context, Result, bail};
use data_transformer::transform;
use std::fs;

use camino::Utf8PathBuf;
use starknet::core::types::ContractClass;
use starknet::core::types::contract::AbiEntry;
use starknet_types_core::felt::Felt;

#[derive(Debug, Clone, clap::Args)]
#[group(multiple = false)]
pub struct Arguments {
    /// Arguments of the called function serialized as a series of felts
    #[arg(short, long, value_delimiter = ' ', num_args = 1..)]
    pub calldata: Option<Vec<String>>,

    // Arguments of the called function as a comma-separated string of Cairo expressions
    #[arg(long, allow_hyphen_values = true)]
    pub arguments: Option<String>,
}

impl Arguments {
    pub fn try_into_calldata(
        self,
        contract_class: Option<ContractClass>,
        selector: &Felt,
        abi_file: Option<Utf8PathBuf>,
    ) -> Result<Vec<Felt>> {
        if let Some(calldata) = self.calldata {
            return calldata
                .iter()
                .map(|data| {
                    Felt::from_dec_str(data)
                        .or_else(|_| Felt::from_hex(data))
                        .context("Failed to parse to felt")
                })
                .collect();
        }

        let abi: Vec<AbiEntry> = match (contract_class, abi_file) {
            (Some(_), Some(_)) => {
                bail!("`contract_class` and `abi_file` params are mutually exclusive");
            }
            (Some(ContractClass::Sierra(sierra_class)), None) => {
                serde_json::from_str(sierra_class.abi.as_str())
                    .context("Couldn't deserialize ABI received from network")?
            }
            (Some(_), None) => {
                bail!("Transformation of arguments is not available for Cairo Zero contracts");
            }
            (None, Some(path)) => {
                let abi_str = fs::read_to_string(path).context("Failed to read ABI file")?;
                serde_json::from_str(&abi_str).context("Failed to deserialize ABI from file")?
            }
            (None, None) => {
                bail!("Either `contract_class` or `abi_file` must be provided");
            }
        };

        let args = self.arguments.unwrap_or_default();
        transform(&args, &abi, selector).context("Failed to transform arguments into calldata")
    }
}
