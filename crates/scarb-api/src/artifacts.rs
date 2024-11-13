use anyhow::Result;

use crate::artifacts::representation::StarknetArtifactsRepresentation;
use camino::{Utf8Path, Utf8PathBuf};
use itertools::Itertools;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::collections::HashMap;
use std::fs;
use universal_sierra_compiler_api::{compile_sierra_at_path, SierraType};

mod deserialized;
mod representation;

/// Contains compiled Starknet artifacts
#[derive(Debug, PartialEq, Clone)]
pub struct StarknetContractArtifacts {
    /// Compiled sierra code
    pub sierra: String,
    /// Compiled casm code
    pub casm: String,
}

#[derive(PartialEq, Debug)]
pub(crate) struct StarknetArtifactsFiles {
    base: Utf8PathBuf,
    other: Vec<Utf8PathBuf>,
}

impl StarknetArtifactsFiles {
    pub(crate) fn new(base_file: Utf8PathBuf, other_files: Vec<Utf8PathBuf>) -> Self {
        Self {
            base: base_file,
            other: other_files,
        }
    }

    // TODO(#2625) add unit tests
    pub(crate) fn load_contracts_artifacts(
        self,
    ) -> Result<HashMap<String, (StarknetContractArtifacts, Utf8PathBuf)>> {
        // TODO(#2626) handle duplicates
        let mut base_artifacts: HashMap<String, (StarknetContractArtifacts, Utf8PathBuf)> =
            compile_artifacts(
                StarknetArtifactsRepresentation::try_from_path(self.base.as_path())?.artifacts(),
            )?;

        let other_artifact_representations: Vec<StarknetArtifactsRepresentation> = self
            .other
            .iter()
            .map(|path| StarknetArtifactsRepresentation::try_from_path(path.as_path()))
            .collect::<Result<_>>()?;

        let other_artifacts: Vec<(String, Utf8PathBuf)> =
            unique_artifacts(other_artifact_representations, &base_artifacts);

        let compiled_artifacts = compile_artifacts(other_artifacts)?;

        base_artifacts.extend(compiled_artifacts);

        Ok(base_artifacts)
    }
}

// TODO(#2625) add unit tests
fn unique_artifacts(
    artifact_representations: Vec<StarknetArtifactsRepresentation>,
    current_artifacts: &HashMap<String, (StarknetContractArtifacts, Utf8PathBuf)>,
) -> Vec<(String, Utf8PathBuf)> {
    artifact_representations
        .into_iter()
        .flat_map(StarknetArtifactsRepresentation::artifacts)
        .unique_by(|(name, _)| name.to_string())
        .filter(|(name, _)| !current_artifacts.contains_key(name))
        .collect()
}

fn compile_artifacts(
    artifacts: Vec<(String, Utf8PathBuf)>,
) -> Result<HashMap<String, (StarknetContractArtifacts, Utf8PathBuf)>> {
    artifacts
        .into_par_iter()
        .map(|(name, path)| {
            compile_artifact_at_path(&path).map(|artifact| (name.to_string(), (artifact, path)))
        })
        .collect::<Result<_>>()
}

fn compile_artifact_at_path(path: &Utf8Path) -> Result<StarknetContractArtifacts> {
    let sierra = fs::read_to_string(path)?;

    let casm = compile_sierra_at_path(path.as_str(), None, &SierraType::Contract)?;

    Ok(StarknetContractArtifacts { sierra, casm })
}
