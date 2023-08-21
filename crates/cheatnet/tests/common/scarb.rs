use anyhow::{anyhow, Context, Result};
use camino::Utf8PathBuf;
use cheatnet::cheatcodes::ContractArtifacts;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

// TODO(#41) TAKEN FROM FORGE, REMOVE AFTER SCARB HAS BEEN MOVED TO A SEPARATE PACKAGE
#[allow(dead_code)]
#[derive(Deserialize, Debug, PartialEq, Clone)]
struct StarknetContract {
    id: String,
    package_name: String,
    contract_name: String,
    artifacts: StarknetContractArtifactPaths,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, PartialEq, Clone)]
struct StarknetContractArtifactPaths {
    sierra: Utf8PathBuf,
    casm: Utf8PathBuf,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
struct StarknetArtifacts {
    version: u32,
    contracts: Vec<StarknetContract>,
}

fn artifacts_for_package(path: &Utf8PathBuf) -> Result<StarknetArtifacts> {
    let starknet_artifacts =
        fs::read_to_string(path).with_context(|| format!("Failed to read {path:?} contents"))?;
    let starknet_artifacts: StarknetArtifacts =
        serde_json::from_str(starknet_artifacts.as_str())
            .with_context(|| format!("Failed to parse {path:?} contents. Make sure you have enabled sierra and casm code generation in Scarb.toml"))?;
    Ok(starknet_artifacts)
}

pub fn try_get_starknet_artifacts_path(
    path: &Utf8PathBuf,
    target_name: &str,
) -> Result<Option<Utf8PathBuf>> {
    let path = path.join("target/dev");
    let paths = fs::read_dir(path);
    let Ok(mut paths) = paths else {
        return Ok(None);
    };
    let starknet_artifacts = paths.find_map(|path| match path {
        Ok(path) => {
            let name = path.file_name().into_string().ok()?;
            (name == format!("{target_name}.starknet_artifacts.json")).then_some(path.path())
        }
        Err(_) => None,
    });
    let starknet_artifacts: Option<Result<Utf8PathBuf>> = starknet_artifacts.map(|artifacts| {
        Utf8PathBuf::try_from(artifacts.clone())
            .with_context(|| format!("Failed to convert path = {artifacts:?} to Utf8PathBuf"))
    });
    starknet_artifacts.transpose()
}

pub fn get_contracts_map(path: &Utf8PathBuf) -> HashMap<String, ContractArtifacts> {
    let base_path = path
        .parent()
        .ok_or_else(|| anyhow!("Failed to get parent for path = {}", path))
        .unwrap();
    let artifacts = artifacts_for_package(path).unwrap();
    let mut map = HashMap::new();
    for contract in artifacts.contracts {
        let name = contract.contract_name;
        let sierra_path = base_path.join(contract.artifacts.sierra);
        let casm_path = base_path.join(contract.artifacts.casm);
        let sierra = fs::read_to_string(sierra_path).unwrap();
        let casm = fs::read_to_string(casm_path).unwrap();
        map.insert(name, ContractArtifacts { sierra, casm });
    }
    map
}
