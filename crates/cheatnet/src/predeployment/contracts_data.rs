use std::{collections::HashMap, fs};

use crate::{
    predeployment::erc20::{
        eth::{ERC20MINTABLE_SIERRA_CLASS_HASH, ETH_CONTRACT_NAME},
        strk::{ERC20LOCKABLE_SIERRA_CLASS_HASH, STRK_CONTRACT_NAME},
    },
    runtime_extensions::forge_runtime_extension::contracts_data::ContractsData,
};
use anyhow::{Context, Result, anyhow};
use camino::Utf8PathBuf;
use conversions::string::TryFromHexStr;
use scarb_api::StarknetContractArtifacts;

macro_rules! load_contract {
    ($name:expr, $path_part:expr) => {{
        let sierra = include_str!(concat!(
            "../data/predeployed_contracts/",
            $path_part,
            "/sierra.json"
        ));
        let casm = include_str!(concat!(
            "../data/predeployed_contracts/",
            $path_part,
            "/casm.json"
        ));

        let artifacts = StarknetContractArtifacts {
            sierra: sierra.to_string(),
            casm: serde_json::from_str(casm)?,
            #[cfg(feature = "cairo-native")]
            executor: None,
        };

        (
            format!("{} (predeployed contract)", $name),
            (
                artifacts,
                persist_embedded_sierra($path_part.to_uppercase().as_str(), sierra)?,
            ),
        )
    }};
}

pub fn load_predeployed_contracts() -> Result<ContractsData> {
    let contracts = HashMap::from([
        load_contract!(STRK_CONTRACT_NAME, "ERC20Lockable"),
        load_contract!(ETH_CONTRACT_NAME, "ERC20Mintable"),
    ]);

    let mut contracts_data = ContractsData::try_from(contracts)?;

    // Additional settings for backtrace and debug info (in Scarb.toml) impact generated sierra.
    // Predeployed contracts are compiled with these settings, because we need support for
    // debugging features (backtrace and traces).
    // These settings affect generated sierra, which means that class hashes of sierra with and without these
    // settings will differ. Contracts on network are compiled without these settings, because they
    // don't need to support mentioned debugging features, and because of that they have different class hashes than predeployed contracts.
    // Considering this, we need to manually override class hashes of predeployed contracts with class hashes of contracts on network.
    let class_hashes_to_change = vec![
        (
            STRK_CONTRACT_NAME.to_string(),
            ERC20LOCKABLE_SIERRA_CLASS_HASH.to_string(),
        ),
        (
            ETH_CONTRACT_NAME.to_string(),
            ERC20MINTABLE_SIERRA_CLASS_HASH.to_string(),
        ),
    ];

    for (contract_name, class_hash) in class_hashes_to_change.into_iter() {
        let class_hash = TryFromHexStr::try_from_hex_str(&class_hash)?;

        let contract_data = contracts_data
            .contracts
            .get_mut(&contract_name)
            .ok_or_else(|| anyhow!("contract data for {contract_name} should exist"))?;
        contract_data.class_hash = class_hash;
    }

    Ok(contracts_data)
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
