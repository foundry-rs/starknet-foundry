use crate::{ErrorData, response::errors::StarknetCommandError};
use conversions::byte_array::ByteArray;
use std::{collections::HashMap, hash::BuildHasher};

/// Contains compiled Starknet artifacts
#[derive(Debug, Clone)]
pub struct CastStarknetContractArtifacts {
    /// Compiled sierra code
    pub sierra: String,
    /// Compiled casm code
    pub casm: String,
}

fn contract_name_from_module_path(module_path: &str) -> &str {
    module_path
        .rsplit("::")
        .next()
        .expect("Absolute module tree path should always contain at least one segment")
}

pub fn resolve_contract_artifacts<'a, S: BuildHasher>(
    contract_identifier: &str,
    artifacts: &'a HashMap<String, CastStarknetContractArtifacts, S>,
) -> Result<&'a CastStarknetContractArtifacts, StarknetCommandError> {
    let mut matching_module_paths: Vec<&str> = artifacts
        .keys()
        .filter(|module_path| contract_name_from_module_path(module_path) == contract_identifier)
        .map(String::as_str)
        .collect();
    matching_module_paths.sort_unstable();

    match matching_module_paths.as_slice() {
        [] => Err(StarknetCommandError::ContractArtifactsNotFound(ErrorData {
            data: ByteArray::from(contract_identifier),
        })),
        [module_path] => Ok(artifacts
            .get(*module_path)
            .expect("artifact should exist for resolved module path")),
        module_paths => {
            let message = format!(
                "Found more than one contract named \"{contract_identifier}\" in artifacts: {}",
                module_paths.join(", ")
            );
            Err(StarknetCommandError::ContractResolutionError(ErrorData {
                data: ByteArray::from(message.as_str()),
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_artifacts() -> HashMap<String, CastStarknetContractArtifacts> {
        HashMap::from([
            (
                "pkg::a::HelloStarknet".to_string(),
                CastStarknetContractArtifacts {
                    sierra: "a".to_string(),
                    casm: "a".to_string(),
                },
            ),
            (
                "pkg::b::HelloStarknet".to_string(),
                CastStarknetContractArtifacts {
                    sierra: "b".to_string(),
                    casm: "b".to_string(),
                },
            ),
            (
                "pkg::ERC20".to_string(),
                CastStarknetContractArtifacts {
                    sierra: "erc20".to_string(),
                    casm: "erc20".to_string(),
                },
            ),
        ])
    }

    #[test]
    fn resolves_unique_contract_by_name() {
        let artifacts = sample_artifacts();

        let artifact = resolve_contract_artifacts("ERC20", &artifacts).unwrap();

        assert_eq!(artifact.sierra, "erc20");
    }

    #[test]
    fn errors_on_ambiguous_contract_name() {
        let artifacts = sample_artifacts();

        let error = resolve_contract_artifacts("HelloStarknet", &artifacts).unwrap_err();

        assert_eq!(
            error.to_string(),
            "Found more than one contract named \"HelloStarknet\" in artifacts: pkg::a::HelloStarknet, pkg::b::HelloStarknet"
        );
    }
}
