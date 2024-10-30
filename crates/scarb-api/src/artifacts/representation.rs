use crate::artifacts::deserialized::{artifacts_for_package, StarknetArtifacts};
use anyhow::anyhow;
use camino::{Utf8Path, Utf8PathBuf};

pub struct StarknetArtifactsRepresentation {
    path: Utf8PathBuf,
    artifacts: StarknetArtifacts,
}

impl StarknetArtifactsRepresentation {
    pub fn try_from_path(path: &Utf8Path) -> anyhow::Result<Self> {
        let artifacts = artifacts_for_package(path)?;
        let path = path
            .parent()
            .ok_or_else(|| anyhow!("Failed to get parent for path = {}", &path))?
            .to_path_buf();

        Ok(Self { path, artifacts })
    }

    pub fn artifacts(self) -> Vec<(String, Utf8PathBuf)> {
        self.artifacts
            .contracts
            .into_iter()
            .map(|contract| {
                (
                    contract.contract_name,
                    self.path.join(contract.artifacts.sierra.as_path()),
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::ScarbCommand;
    use assert_fs::fixture::{FileTouch, FileWriteStr, PathChild, PathCopy};
    use assert_fs::TempDir;
    use camino::Utf8PathBuf;

    use super::*;

    #[test]
    fn parsing_starknet_artifacts() {
        let temp = crate::tests::setup_package("basic_package");

        ScarbCommand::new_with_stdio()
            .current_dir(temp.path())
            .arg("build")
            .run()
            .unwrap();

        let artifacts_path = temp
            .path()
            .join("target/dev/basic_package.starknet_artifacts.json");
        let artifacts_path = Utf8PathBuf::from_path_buf(artifacts_path).unwrap();

        let artifacts = artifacts_for_package(&artifacts_path).unwrap();

        assert!(!artifacts.contracts.is_empty());
    }

    #[test]
    fn parsing_starknet_artifacts_on_invalid_file() {
        let temp = TempDir::new().unwrap();
        temp.copy_from("../../", &[".tool-versions"]).unwrap();
        let path = temp.child("wrong.json");
        path.touch().unwrap();
        path.write_str("\"aa\": {}").unwrap();
        let artifacts_path = Utf8PathBuf::from_path_buf(path.to_path_buf()).unwrap();

        let result = artifacts_for_package(&artifacts_path);
        let err = result.unwrap_err();

        assert!(err.to_string().contains(&format!("Failed to parse {artifacts_path:?} contents. Make sure you have enabled sierra code generation in Scarb.toml")));
    }
}
