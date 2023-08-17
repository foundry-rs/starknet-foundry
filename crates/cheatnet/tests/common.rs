use anyhow::{anyhow, Context, Result};
use cairo_felt::Felt252;
use camino::Utf8PathBuf;
use cheatnet::{cheatcodes::ContractArtifacts, conversions::felt_from_short_string, CheatnetState};
use include_dir::{include_dir, Dir};
use serde::Deserialize;
use starknet_api::core::ContractAddress;
use std::fs;
use std::str::FromStr;
use std::{collections::HashMap, path::PathBuf};
use tempfile::{tempdir, TempDir};

static PREDEPLOYED_CONTRACTS: Dir = include_dir!("crates/cheatnet/predeployed-contracts");
static TARGET_NAME: &str = "cheatnet_testing_contracts";

fn load_predeployed_contracts() -> TempDir {
    let tmp_dir = tempdir().expect("Failed to create a temporary directory");
    PREDEPLOYED_CONTRACTS
        .extract(&tmp_dir)
        .expect("Failed to copy corelib to temporary directory");
    tmp_dir
}

pub fn create_cheatnet_state() -> CheatnetState {
    let predeployed_contracts_dir = load_predeployed_contracts();
    let predeployed_contracts: PathBuf = predeployed_contracts_dir.path().into();
    let predeployed_contracts = Utf8PathBuf::try_from(predeployed_contracts)
        .expect("Failed to convert path to predeployed contracts to Utf8PathBuf");

    CheatnetState::new(&predeployed_contracts)
}

// TAKEN FROM FORGE
// REMOVE AFTER SCARB HAS BEEN MOVED TO A SEPARATE PACKAGE
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

fn try_get_starknet_artifacts_path(
    path: &Utf8PathBuf,
    target_name: &str,
) -> Result<Option<Utf8PathBuf>> {
    let path = path.join("target/dev");
    dbg!(&path);
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

fn get_contracts_map(path: &Utf8PathBuf) -> HashMap<String, ContractArtifacts> {
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

pub fn deploy_contract(
    state: &mut CheatnetState,
    contract_name: &str,
    calldata: &[Felt252],
) -> ContractAddress {
    let contract = felt_from_short_string(&contract_name);

    let contracts_path = Utf8PathBuf::from_str("tests/contracts")
        .unwrap()
        .canonicalize_utf8()
        .unwrap();
    dbg!(&contracts_path);
    let artifacts_path = try_get_starknet_artifacts_path(&contracts_path, TARGET_NAME)
        .unwrap()
        .unwrap();
    let contracts = get_contracts_map(&artifacts_path);

    let class_hash = state.declare(&contract, &contracts).unwrap();
    state.deploy(&class_hash, calldata).unwrap()
}
