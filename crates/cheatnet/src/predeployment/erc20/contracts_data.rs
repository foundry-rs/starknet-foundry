use std::{collections::HashMap, fs};

use crate::predeployment::erc20::{eth::ETH_CONTRACT_NAME, strk::STRK_CONTRACT_NAME};
use anyhow::Result;
use camino::Utf8PathBuf;
use scarb_api::StarknetContractArtifacts;

pub fn load_erc20_predeployed_contracts()
-> Result<HashMap<String, (StarknetContractArtifacts, Utf8PathBuf)>> {
    Ok(HashMap::from([
        (
            STRK_CONTRACT_NAME.to_string(),
            (
                StarknetContractArtifacts {
                    sierra: fs::read_to_string(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/src/data/predeployed_contracts/strk/sierra.json"
                    ))?,
                    casm: serde_json::from_str(&fs::read_to_string(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/src/data/predeployed_contracts/strk/casm.json"
                    ))?)?,
                    #[cfg(feature = "cairo-native")]
                    executor: None,
                },
                Utf8PathBuf::from(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/src/data/predeployed_contracts/strk/sierra.json"
                )),
            ),
        ),
        (
            ETH_CONTRACT_NAME.to_string(),
            (
                StarknetContractArtifacts {
                    sierra: fs::read_to_string(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/src/data/predeployed_contracts/eth/sierra.json"
                    ))?,
                    casm: serde_json::from_str(&fs::read_to_string(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/src/data/predeployed_contracts/eth/casm.json"
                    ))?)?,
                    #[cfg(feature = "cairo-native")]
                    executor: None,
                },
                Utf8PathBuf::from(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/src/data/predeployed_contracts/eth/sierra.json"
                )),
            ),
        ),
    ]))
}
