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
use std::{collections::HashMap, fs};

pub const CACHE_DIR: &str = ".snfoundry_cache";

/// Load data of predeployed contract from its artifacts
macro_rules! load_contract {
    ($name:expr, $contract_dir:expr) => {{
        let sierra = include_str!(concat!(
            "../data/predeployed_contracts/",
            $contract_dir,
            "/sierra.json"
        ));
        let casm = include_str!(concat!(
            "../data/predeployed_contracts/",
            $contract_dir,
            "/casm.json"
        ));

        let artifacts = StarknetContractArtifacts {
            sierra: sierra.to_string(),
            casm: serde_json::from_str(casm)?,
            #[cfg(feature = "cairo-native")]
            executor: None,
        };

        (
            $name.to_string(),
            (
                artifacts,
                maybe_cache_contract_sierra($contract_dir, sierra)?,
            ),
        )
    }};
}

/// Loads data of predeployed contracts from their artifacts, and prepares it for usage in cheatnet.
pub fn load_predeployed_contracts() -> Result<ContractsData> {
    let contracts = HashMap::from([
        load_contract!(STRK_CONTRACT_NAME, "ERC20Lockable"),
        load_contract!(ETH_CONTRACT_NAME, "ERC20Mintable"),
    ]);

    let mut contracts_data = ContractsData::try_from(contracts)?;

    // Local predeployed contracts are compiled with debug features (backtrace and traces) enabled
    // in Scarb.toml to work with debugging features. Since these settings modify the
    // generated Sierra code, the resulting class hashes differ from those of contracts
    // deployed on-chain, which are compiled without mentioned compiler settings.
    // To ensure consistency with the network, we manually override the local class hashes
    // with their official on-chain equivalents.
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

    for (contract_name, class_hash) in class_hashes_to_change {
        let class_hash = TryFromHexStr::try_from_hex_str(&class_hash)?;

        let predeployed_contract = contracts_data
            .contracts
            .get_mut(&contract_name)
            .ok_or_else(|| anyhow!("contract data for {contract_name} should exist"))?;
        predeployed_contract.class_hash = class_hash;
    }

    Ok(contracts_data)
}

/// Saves sierra file of predeployed contract to cache, and returns path to it.
/// If the file already exists in the cache, it skips the write operation.
fn maybe_cache_contract_sierra(contract_name: &str, sierra: &str) -> Result<Utf8PathBuf> {
    let path = Utf8PathBuf::from(CACHE_DIR)
        .join("predeployed_contracts")
        .join(env!("CARGO_PKG_VERSION"))
        .join(format!("{contract_name}.sierra.json"));

    if path.exists() {
        return Ok(path);
    }

    let parent = path
        .parent()
        .with_context(|| format!("Failed to get parent directory for {path}"))?;
    fs::create_dir_all(parent).with_context(|| {
        format!("Failed to create directory for sierra of {contract_name} at {parent}")
    })?;
    fs::write(&path, sierra)
        .with_context(|| format!("Failed to write Sierra for {contract_name} to {path}"))?;

    Ok(path)
}
