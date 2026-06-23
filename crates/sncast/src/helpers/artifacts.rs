use crate::{ErrorData, response::errors::StarknetCommandError};
use conversions::byte_array::ByteArray;
use shared::utils::contract_name_from_module_path;
use std::{collections::HashMap, hash::BuildHasher};

/// Contains compiled Starknet artifacts
#[derive(Debug, Clone)]
pub struct CastStarknetContractArtifacts {
    /// Compiled sierra code
    pub sierra: String,
    /// Compiled casm code
    pub casm: String,
}

pub fn resolve_contract_artifacts<'a, S: BuildHasher>(
    contract_identifier: &str,
    artifacts: &'a HashMap<String, CastStarknetContractArtifacts, S>,
) -> Result<&'a CastStarknetContractArtifacts, StarknetCommandError> {
    let mut matches: Vec<(&str, &CastStarknetContractArtifacts)> = artifacts
        .iter()
        .filter(|(module_path, _)| {
            contract_name_from_module_path(module_path) == contract_identifier
        })
        .map(|(module_path, artifact)| (module_path.as_str(), artifact))
        .collect();

    match matches.as_slice() {
        [] => Err(StarknetCommandError::ContractArtifactsNotFound(ErrorData {
            data: ByteArray::from(contract_identifier),
        })),
        [(_, artifact)] => Ok(artifact),
        _ => {
            matches.sort_unstable_by_key(|(module_path, _)| *module_path);
            let message = format!(
                "Found more than one contract named \"{contract_identifier}\" at: {}",
                matches
                    .iter()
                    .map(|(module_path, _)| *module_path)
                    .collect::<Vec<_>>()
                    .join(", ")
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
            "Found more than one contract named \"HelloStarknet\" at: pkg::a::HelloStarknet, pkg::b::HelloStarknet"
        );
    }

    #[test]
    fn errors_when_contract_artifacts_not_found() {
        let artifacts = sample_artifacts();

        let error = resolve_contract_artifacts("MissingContract", &artifacts).unwrap_err();

        assert!(matches!(
            error,
            StarknetCommandError::ContractArtifactsNotFound(_)
        ));
    }
}
