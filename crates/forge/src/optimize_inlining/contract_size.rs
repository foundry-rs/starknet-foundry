use anyhow::{Context, Result};
use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
use cairo_lang_starknet_classes::contract_class::ContractClass;
use camino::{Utf8Path, Utf8PathBuf};
use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct StarknetArtifacts {
    pub version: u32,
    pub contracts: Vec<StarknetContract>,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct StarknetContract {
    pub id: String,
    pub package_name: String,
    pub contract_name: String,
    pub artifacts: StarknetContractArtifactPaths,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct StarknetContractArtifactPaths {
    pub sierra: Option<Utf8PathBuf>,
    pub casm: Option<Utf8PathBuf>,
}

fn parse_starknet_artifacts(path: &Utf8Path) -> Result<StarknetArtifacts> {
    let content = fs::read_to_string(path).with_context(|| format!("Failed to read {path}"))?;
    serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse starknet artifacts from {path}"))
}

#[derive(Debug)]
pub struct ContractSizeInfo {
    pub name: String,
    pub path: Utf8PathBuf,
    pub size: u64,
    pub felts_count: u64,
}

pub fn check_contract_sizes(
    starknet_artifacts_paths: &[Utf8PathBuf],
    max_size: u64,
    max_felts_count: u64,
) -> Result<(bool, Vec<ContractSizeInfo>)> {
    let mut sizes = Vec::new();
    let mut all_valid = true;

    let get_contract_size = |contract_path: &Utf8PathBuf| -> Result<u64> {
        let size = fs::metadata(&contract_path)
            .with_context(|| format!("Failed to read {contract_path}"))?
            .len();
        Ok(size)
    };

    for starknet_artifacts_path in starknet_artifacts_paths {
        let artifacts = parse_starknet_artifacts(starknet_artifacts_path)?;
        let artifacts_dir = starknet_artifacts_path
            .parent()
            .expect("Starknet artifacts path must have a parent");

        for contract in &artifacts.contracts {
            if let Some(sierra_path) = &contract.artifacts.sierra {
                let sierra_path = artifacts_dir.join(sierra_path);
                let size = get_contract_size(&sierra_path)?;
                if size > max_size {
                    all_valid = false;
                }
                let class: ContractClass =
                    serde_json::from_str(&fs::read_to_string(&sierra_path)?)?;
                let sierra_felts: u64 = class.sierra_program.len() as u64;
                if sierra_felts > max_felts_count {
                    all_valid = false;
                }
                sizes.push(ContractSizeInfo {
                    name: contract.contract_name.clone(),
                    path: sierra_path,
                    size,
                    felts_count: sierra_felts,
                });
            };
            if let Some(casm_path) = &contract.artifacts.casm {
                let casm_path = artifacts_dir.join(casm_path);
                let size = get_contract_size(&casm_path)?;
                if size > max_size {
                    all_valid = false;
                }
                let class: CasmContractClass =
                    serde_json::from_str(&fs::read_to_string(&casm_path)?)?;
                let casm_felts: u64 = class.bytecode.len() as u64;
                if casm_felts > max_felts_count {
                    all_valid = false;
                }
                sizes.push(ContractSizeInfo {
                    name: contract.contract_name.clone(),
                    path: casm_path,
                    size,
                    felts_count: casm_felts,
                });
            };
        }
    }

    Ok((all_valid, sizes))
}
