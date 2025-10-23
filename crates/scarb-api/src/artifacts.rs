use anyhow::Result;

use crate::artifacts::representation::StarknetArtifactsRepresentation;
use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
#[cfg(feature = "cairo-native")]
use cairo_native::executor::AotContractExecutor;
use camino::{Utf8Path, Utf8PathBuf};
use itertools::Itertools;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::collections::HashMap;
use std::fs;
use universal_sierra_compiler_api::compile_contract_sierra_at_path;

mod deserialized;
mod representation;

/// Contains compiled Starknet artifacts
#[derive(Debug, Clone)]
pub struct StarknetContractArtifacts {
    /// Compiled sierra code
    pub sierra: String,
    /// Compiled casm code
    pub casm: CasmContractClass,

    #[cfg(feature = "cairo-native")]
    /// Optional AOT compiled native executor
    pub executor: Option<AotContractExecutor>,
}

impl PartialEq for StarknetContractArtifacts {
    fn eq(&self, other: &Self) -> bool {
        let eq = self.sierra == other.sierra && self.casm == other.casm;

        #[cfg(feature = "cairo-native")]
        let eq = eq && self.executor.is_some() == other.executor.is_some();

        eq
    }
}

#[derive(PartialEq, Debug)]
pub(crate) struct StarknetArtifactsFiles {
    base: Utf8PathBuf,
    other: Vec<Utf8PathBuf>,
    #[cfg(feature = "cairo-native")]
    compile_native: bool,
}

impl StarknetArtifactsFiles {
    pub(crate) fn new(base_file: Utf8PathBuf, other_files: Vec<Utf8PathBuf>) -> Self {
        Self {
            base: base_file,
            other: other_files,
            #[cfg(feature = "cairo-native")]
            compile_native: false,
        }
    }

    #[cfg(feature = "cairo-native")]
    pub(crate) fn compile_native(mut self, compile_native: bool) -> Self {
        self.compile_native = compile_native;
        self
    }

    #[tracing::instrument(skip_all, level = "debug")]
    pub(crate) fn load_contracts_artifacts(
        self,
    ) -> Result<HashMap<String, (StarknetContractArtifacts, Utf8PathBuf)>> {
        // TODO(#2626) handle duplicates
        let mut base_artifacts: HashMap<String, (StarknetContractArtifacts, Utf8PathBuf)> = self
            .compile_artifacts(
                StarknetArtifactsRepresentation::try_from_path(self.base.as_path())?.artifacts(),
            )?;

        let other_artifact_representations: Vec<StarknetArtifactsRepresentation> = self
            .other
            .iter()
            .map(|path| StarknetArtifactsRepresentation::try_from_path(path.as_path()))
            .collect::<Result<_>>()?;

        let other_artifacts: Vec<(String, Utf8PathBuf)> =
            unique_artifacts(other_artifact_representations, &base_artifacts);

        let compiled_artifacts = self.compile_artifacts(other_artifacts)?;

        base_artifacts.extend(compiled_artifacts);

        Ok(base_artifacts)
    }

    #[tracing::instrument(skip_all, level = "debug")]
    fn compile_artifacts(
        &self,
        artifacts: Vec<(String, Utf8PathBuf)>,
    ) -> Result<HashMap<String, (StarknetContractArtifacts, Utf8PathBuf)>> {
        artifacts
            .into_par_iter()
            .map(|(name, path)| {
                self.compile_artifact_at_path(&path)
                    .map(|artifact| (name.to_string(), (artifact, path)))
            })
            .collect::<Result<_>>()
    }

    #[tracing::instrument(skip_all, level = "debug")]
    #[cfg_attr(not(feature = "cairo-native"), expect(clippy::unused_self))]
    fn compile_artifact_at_path(&self, path: &Utf8Path) -> Result<StarknetContractArtifacts> {
        let sierra = fs::read_to_string(path)?;

        let casm = compile_contract_sierra_at_path(path.as_std_path())?;

        #[cfg(feature = "cairo-native")]
        let executor = self.compile_to_native(&sierra)?;

        Ok(StarknetContractArtifacts {
            sierra,
            casm,
            #[cfg(feature = "cairo-native")]
            executor,
        })
    }

    #[cfg(feature = "cairo-native")]
    #[tracing::instrument(skip_all, level = "debug")]
    fn compile_to_native(&self, sierra: &str) -> Result<Option<AotContractExecutor>> {
        Ok(if self.compile_native {
            Some(native_api::compile_contract_class(&serde_json::from_str(
                sierra,
            )?)?)
        } else {
            None
        })
    }
}

#[tracing::instrument(skip_all, level = "debug")]
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ScarbCommand;
    use crate::tests::setup_package;
    use assert_fs::TempDir;
    use assert_fs::fixture::{FileWriteStr, PathChild};
    use cairo_lang_starknet_classes::casm_contract_class::CasmContractEntryPoints;
    use camino::Utf8PathBuf;
    use deserialized::{StarknetArtifacts, StarknetContract, StarknetContractArtifactPaths};
    use indoc::indoc;
    use num_bigint::BigUint;

    #[test]
    fn test_unique_artifacts() {
        // Mock StarknetArtifactsRepresentation
        let mock_base_artifacts = HashMap::from([(
            "contract1".to_string(),
            (
                StarknetContractArtifacts {
                    sierra: "sierra1".to_string(),
                    casm: CasmContractClass {
                        prime: BigUint::default(),
                        compiler_version: String::default(),
                        bytecode: vec![],
                        bytecode_segment_lengths: None,
                        hints: vec![],
                        pythonic_hints: None,
                        entry_points_by_type: CasmContractEntryPoints::default(),
                    },
                    #[cfg(feature = "cairo-native")]
                    executor: None,
                },
                Utf8PathBuf::from("path1"),
            ),
        )]);

        let mock_representation_1 = StarknetArtifactsRepresentation {
            base_path: Utf8PathBuf::from("mock/path1"),
            artifacts: StarknetArtifacts {
                version: 1,
                contracts: vec![StarknetContract {
                    id: "1".to_string(),
                    package_name: "package1".to_string(),
                    contract_name: "contract1".to_string(),
                    artifacts: StarknetContractArtifactPaths {
                        sierra: Utf8PathBuf::from("mock/path1/contract1.sierra"),
                    },
                }],
            },
        };

        let mock_representation_2 = StarknetArtifactsRepresentation {
            base_path: Utf8PathBuf::from("mock/path2"),
            artifacts: StarknetArtifacts {
                version: 1,
                contracts: vec![StarknetContract {
                    id: "2".to_string(),
                    package_name: "package2".to_string(),
                    contract_name: "contract2".to_string(),
                    artifacts: StarknetContractArtifactPaths {
                        sierra: Utf8PathBuf::from("mock/path2/contract2.sierra"),
                    },
                }],
            },
        };

        let representations = vec![mock_representation_1, mock_representation_2];

        let result = unique_artifacts(representations, &mock_base_artifacts);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "contract2");
    }

    fn setup() -> (TempDir, StarknetArtifactsFiles) {
        let temp = setup_package("basic_package");
        let tests_dir = temp.join("tests");
        fs::create_dir(&tests_dir).unwrap();

        temp.child(tests_dir.join("test.cairo"))
            .write_str(indoc!(
                r"
                    #[test]
                    fn mock_test() {
                        assert!(true);
                    }
                "
            ))
            .unwrap();

        ScarbCommand::new_with_stdio()
            .current_dir(temp.path())
            .arg("build")
            .arg("--test")
            .run()
            .unwrap();

        // Define path to the generated artifacts
        let base_artifacts_path = temp.to_path_buf().join("target").join("dev");

        // Get the base artifact
        let base_file = Utf8PathBuf::from_path_buf(
            base_artifacts_path.join("basic_package_integrationtest.test.starknet_artifacts.json"),
        )
        .unwrap();

        // Load other artifact files and add them to the temporary directory
        let other_files = vec![
            Utf8PathBuf::from_path_buf(
                base_artifacts_path.join("basic_package_unittest.test.starknet_artifacts.json"),
            )
            .unwrap(),
        ];

        // Create `StarknetArtifactsFiles`
        let artifacts_files = StarknetArtifactsFiles::new(base_file, other_files);

        (temp, artifacts_files)
    }

    #[test]
    fn test_load_contracts_artifacts() {
        let (_temp, artifacts_files) = setup();

        // Load the contracts
        let result = artifacts_files.load_contracts_artifacts().unwrap();

        // Assert the Contract Artifacts are loaded.
        assert!(result.contains_key("ERC20"));
        assert!(result.contains_key("HelloStarknet"));
    }

    #[test]
    #[cfg(feature = "cairo-native")]
    fn test_load_contracts_artifacts_native() {
        let (_temp, artifacts_files) = setup();

        let artifacts_files = artifacts_files.compile_native(true);

        // Load the contracts
        let result = artifacts_files.load_contracts_artifacts().unwrap();

        // Assert the Contract Artifacts are loaded.
        assert!(result.contains_key("ERC20"));
        assert!(result.contains_key("HelloStarknet"));
        assert!(result.get("ERC20").unwrap().0.executor.is_some());
    }
}
