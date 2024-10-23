use crate::contracts::StarknetArtifactsRepresentation;
use crate::StarknetContractArtifacts;
use camino::Utf8PathBuf;
use itertools::Itertools;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::collections::HashMap;

#[derive(PartialEq, Debug)]
pub(crate) struct StarknetArtifactsFiles {
    base_file: Utf8PathBuf,
    other_files: Vec<Utf8PathBuf>,
}

impl StarknetArtifactsFiles {
    pub(crate) fn new(base_file: Utf8PathBuf, other_files: Vec<Utf8PathBuf>) -> Self {
        Self {
            base_file,
            other_files,
        }
    }

    pub(crate) fn load_contracts_artifacts(
        self,
    ) -> anyhow::Result<HashMap<String, (StarknetContractArtifacts, Utf8PathBuf)>> {
        let mut base_artifacts: HashMap<String, (StarknetContractArtifacts, Utf8PathBuf)> =
            compile_artifacts(
                StarknetArtifactsRepresentation::try_from_path(self.base_file.as_path())?
                    .artifacts(),
            )?;

        let other_artifact_representations: Vec<StarknetArtifactsRepresentation> = self
            .other_files
            .iter()
            .map(|path| StarknetArtifactsRepresentation::try_from_path(path.as_path()))
            .collect::<anyhow::Result<_>>()?;

        let other_artifacts: Vec<(String, Utf8PathBuf)> =
            unique_artifacts(other_artifact_representations, &base_artifacts);

        let compiled_artifacts = compile_artifacts(other_artifacts)?;

        base_artifacts.extend(compiled_artifacts);

        Ok(base_artifacts)
    }
}

fn unique_artifacts(
    artifact_representations: Vec<StarknetArtifactsRepresentation>,
    current_artifacts: &HashMap<String, (StarknetContractArtifacts, Utf8PathBuf)>,
) -> Vec<(String, Utf8PathBuf)> {
    artifact_representations
        .into_iter()
        .flat_map(StarknetArtifactsRepresentation::artifacts)
        .unique()
        .filter(|(name, _)| !current_artifacts.contains_key(name))
        .collect()
}

fn compile_artifacts(
    artifacts: Vec<(String, Utf8PathBuf)>,
) -> anyhow::Result<HashMap<String, (StarknetContractArtifacts, Utf8PathBuf)>> {
    artifacts
        .into_par_iter()
        .map(|(name, path)| {
            StarknetContractArtifacts::try_compile_at_path(path.as_path())
                .map(|artifact| (name.to_string(), (artifact, path)))
        })
        .collect::<anyhow::Result<_>>()
}
