use std::{collections::HashMap, fs};

use crate::predeployment::erc20::{eth::ETH_CONTRACT_NAME, strk::STRK_CONTRACT_NAME};
use anyhow::{Context, Result, anyhow};
use camino::Utf8PathBuf;
use scarb_api::StarknetContractArtifacts;

pub fn load_erc20_predeployed_contracts()
-> Result<HashMap<String, (StarknetContractArtifacts, Utf8PathBuf)>> {
    let strk_sierra = include_str!("../../data/predeployed_contracts/STRK/sierra.json");
    let eth_sierra = include_str!("../../data/predeployed_contracts/ETH/sierra.json");

    Ok(HashMap::from([
        (
            STRK_CONTRACT_NAME.to_string(),
            (
                StarknetContractArtifacts {
                    sierra: strk_sierra.to_string(),
                    casm: serde_json::from_str(include_str!(
                        "../../data/predeployed_contracts/STRK/casm.json"
                    ))?,
                    #[cfg(feature = "cairo-native")]
                    executor: None,
                },
                persist_embedded_sierra("STRK", strk_sierra)?,
            ),
        ),
        (
            ETH_CONTRACT_NAME.to_string(),
            (
                StarknetContractArtifacts {
                    sierra: eth_sierra.to_string(),
                    casm: serde_json::from_str(include_str!(
                        "../../data/predeployed_contracts/ETH/casm.json"
                    ))?,
                    #[cfg(feature = "cairo-native")]
                    executor: None,
                },
                persist_embedded_sierra("ETH", eth_sierra)?,
            ),
        ),
    ]))
}

fn persist_embedded_sierra(contract_name: &str, sierra: &str) -> Result<Utf8PathBuf> {
    let path = std::env::temp_dir()
        .join("snfoundry-predeployed-contracts")
        .join(env!("CARGO_PKG_VERSION"))
        .join(format!("{contract_name}.sierra.json"));

    let parent = path
        .parent()
        .with_context(|| format!("Failed to get parent directory for {}", path.display()))?;
    fs::create_dir_all(parent).with_context(|| {
        format!(
            "Failed to create directory for predeployed Sierra at {}",
            parent.display()
        )
    })?;
    fs::write(&path, sierra).with_context(|| {
        format!(
            "Failed to materialize embedded Sierra for {contract_name} at {}",
            path.display()
        )
    })?;

    Utf8PathBuf::from_path_buf(path).map_err(|path| {
        anyhow!(
            "Materialized Sierra path is not valid UTF-8: {}",
            path.display()
        )
    })
}
