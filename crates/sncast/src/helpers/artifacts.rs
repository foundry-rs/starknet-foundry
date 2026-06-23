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

pub type ContractModulePath = String;
pub type ContractArtifactsMap<S = std::collections::hash_map::RandomState> =
    HashMap<ContractModulePath, CastStarknetContractArtifacts, S>;

pub fn resolve_contract_artifacts<'a, S: BuildHasher>(
    contract_identifier: &str,
    artifacts: &'a ContractArtifactsMap<S>,
) -> Result<&'a CastStarknetContractArtifacts, StarknetCommandError> {
    let contract_identifier = contract_identifier
        .strip_prefix("::")
        .unwrap_or(contract_identifier);
    let module_path_suffix = format!("::{contract_identifier}");

    let mut matches: Vec<(&str, &CastStarknetContractArtifacts)> = artifacts
        .iter()
        .filter(|(module_path, _)| {
            *module_path == contract_identifier || module_path.ends_with(&module_path_suffix)
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
            let module_paths = matches
                .iter()
                .map(|(module_path, _)| format!(" - {module_path}"))
                .collect::<Vec<_>>()
                .join("\n");
            let message = format!(
                "Found more than one contract matching \"{contract_identifier}\". Pass one of these module paths to `--contract-name`:\n{module_paths}",
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

    fn sample_artifacts() -> ContractArtifactsMap {
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

    fn sample_artifacts_with_nested_module_paths() -> ContractArtifactsMap {
        let mut artifacts = sample_artifacts();
        artifacts.insert(
            "pkg::nested::a::HelloStarknet".to_string(),
            CastStarknetContractArtifacts {
                sierra: "nested-a".to_string(),
                casm: "nested-a".to_string(),
            },
        );
        artifacts
    }

    #[test]
    fn resolves_unique_contract_by_name() {
        let artifacts = sample_artifacts();

        let artifact = resolve_contract_artifacts("ERC20", &artifacts).unwrap();

        assert_eq!(artifact.sierra, "erc20");
    }

    #[test]
    fn resolves_contract_by_partial_module_path() {
        let artifacts = sample_artifacts();

        let artifact = resolve_contract_artifacts("a::HelloStarknet", &artifacts).unwrap();

        assert_eq!(artifact.sierra, "a");
    }

    #[test]
    fn resolves_contract_by_partial_module_path_with_leading_colons() {
        let artifacts = sample_artifacts();

        let artifact = resolve_contract_artifacts("::a::HelloStarknet", &artifacts).unwrap();

        assert_eq!(artifact.sierra, "a");
    }

    #[test]
    fn resolves_contract_by_deeper_partial_module_path() {
        let artifacts = sample_artifacts_with_nested_module_paths();

        let artifact = resolve_contract_artifacts("nested::a::HelloStarknet", &artifacts).unwrap();

        assert_eq!(artifact.sierra, "nested-a");
    }

    #[test]
    fn resolves_contract_by_full_module_path() {
        let artifacts = sample_artifacts();

        let artifact = resolve_contract_artifacts("pkg::a::HelloStarknet", &artifacts).unwrap();

        assert_eq!(artifact.sierra, "a");
    }

    #[test]
    fn errors_on_ambiguous_contract_name() {
        let artifacts = sample_artifacts();

        let error = resolve_contract_artifacts("HelloStarknet", &artifacts).unwrap_err();

        assert_eq!(
            error.to_string(),
            "Found more than one contract matching \"HelloStarknet\". Pass one of these module paths to `--contract-name`:\n - pkg::a::HelloStarknet\n - pkg::b::HelloStarknet"
        );
    }

    #[test]
    fn errors_on_ambiguous_partial_module_path() {
        let artifacts = sample_artifacts_with_nested_module_paths();

        let error = resolve_contract_artifacts("a::HelloStarknet", &artifacts).unwrap_err();

        assert_eq!(
            error.to_string(),
            "Found more than one contract matching \"a::HelloStarknet\". Pass one of these module paths to `--contract-name`:\n - pkg::a::HelloStarknet\n - pkg::nested::a::HelloStarknet"
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
