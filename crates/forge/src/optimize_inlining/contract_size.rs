use anyhow::{Context, Result, bail};
use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
use cairo_lang_starknet_classes::contract_class::ContractClass;
use camino::Utf8PathBuf;
use scarb_api::artifacts::deserialized::artifacts_for_package;
use std::collections::HashSet;
use std::fs;

const MODULE_PATH_SEPARATOR: &str = "::";

#[derive(Debug)]
pub struct ContractSizeInfo {
    pub contract_id: String,
    pub artifact_type: ContractArtifactType,
    pub size: u64,
    pub felts_count: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractArtifactType {
    Sierra,
    Casm,
}

pub fn check_and_validate_contract_sizes(
    starknet_artifacts_paths: &[Utf8PathBuf],
    max_size: u64,
    max_felts_count: u64,
    contracts_filter: &[String],
) -> Result<(bool, Vec<ContractSizeInfo>)> {
    let mut sizes = Vec::new();
    let mut all_valid = true;
    let mut matched_filters: HashSet<&str> = HashSet::new();
    let mut available_contracts: Vec<String> = Vec::new();

    for starknet_artifacts_path in starknet_artifacts_paths {
        let artifacts = artifacts_for_package(starknet_artifacts_path.as_path())?;
        let artifacts_dir = starknet_artifacts_path
            .parent()
            .expect("Starknet artifacts path must have a parent");

        for contract in &artifacts.contracts {
            available_contracts.push(contract.contract_name.clone());

            let matching_filter = contracts_filter.iter().find(|f| {
                if contract.contract_name == **f {
                    return true;
                }
                if !f.contains(MODULE_PATH_SEPARATOR) {
                    return false;
                }
                contract.module_path.ends_with(f.as_str())
            });
            let Some(filter) = matching_filter else {
                continue;
            };
            matched_filters.insert(filter.as_str());

            let sierra_path = artifacts_dir.join(&contract.artifacts.sierra);
            let size = get_contract_size(&sierra_path)?;
            if size > max_size {
                all_valid = false;
            }
            let class: ContractClass = serde_json::from_str(&fs::read_to_string(&sierra_path)?)?;
            let sierra_felts: u64 = class.sierra_program.len() as u64;
            if sierra_felts > max_felts_count {
                all_valid = false;
            }
            sizes.push(ContractSizeInfo {
                contract_id: contract.id.clone(),
                artifact_type: ContractArtifactType::Sierra,
                size,
                felts_count: sierra_felts,
            });

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
                    contract_id: contract.id.clone(),
                    artifact_type: ContractArtifactType::Casm,
                    size,
                    felts_count: casm_felts,
                });
            }
        }
    }

    let unmatched: Vec<&str> = contracts_filter
        .iter()
        .filter(|f| !matched_filters.contains(f.as_str()))
        .map(String::as_str)
        .collect();

    if !unmatched.is_empty() {
        bail!(
            "The following contracts were not found in starknet artifacts: {}. Available contracts: {}",
            unmatched.join(", "),
            available_contracts.join(", ")
        );
    }

    Ok((all_valid, sizes))
}

fn get_contract_size(contract_path: &Utf8PathBuf) -> Result<u64> {
    let size = fs::metadata(contract_path)
        .with_context(|| format!("Failed to read {contract_path}"))?
        .len();
    Ok(size)
}
