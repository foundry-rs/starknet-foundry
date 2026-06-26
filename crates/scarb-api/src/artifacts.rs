use anyhow::Result;

use crate::artifacts::representation::{ContractArtifact, StarknetArtifactsRepresentation};
use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
#[cfg(feature = "cairo-native")]
use cairo_native::executor::AotContractExecutor;
use camino::{Utf8Path, Utf8PathBuf};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::collections::HashMap;
use std::fs;
use universal_sierra_compiler_api::compile_contract_sierra_at_path;

pub mod deserialized;
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

#[derive(Debug, Clone, PartialEq)]
pub struct ContractData {
    pub name: String,
    pub artifacts: StarknetContractArtifacts,
    pub sierra_path: Utf8PathBuf,
}

/// A mapping of module tree paths to their corresponding contract data.
pub type ContractsData = HashMap<String, ContractData>;

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
    pub(crate) fn load_contracts_artifacts(self) -> Result<ContractsData> {
        // Gather contract artifacts across the base and all other representations.
        // The same contract may be emitted into both test targets (unittest and
        // integrationtest) under the same `module_path`, so we collapse identical
        // module paths here. Distinct module paths sharing a `name` are genuine
        // duplicates and are all kept.
        let mut all_artifacts: Vec<ContractArtifact> =
            StarknetArtifactsRepresentation::try_from_path(self.base.as_path())?.artifacts();

        for path in &self.other {
            let representation = StarknetArtifactsRepresentation::try_from_path(path.as_path())?;
            all_artifacts.extend(representation.artifacts());
        }

        self.compile_artifacts(deduplicate_by_module_path(all_artifacts))
    }

    #[tracing::instrument(skip_all, level = "debug")]
    fn compile_artifacts(&self, artifacts: Vec<ContractArtifact>) -> Result<ContractsData> {
        artifacts
            .into_par_iter()
            .map(|artifact| {
                let ContractArtifact {
                    name,
                    module_path,
                    sierra_path,
                } = artifact;
                let artifacts = self.compile_artifact_at_path(&sierra_path)?;
                Ok((
                    module_path,
                    ContractData {
                        name,
                        artifacts,
                        sierra_path,
                    },
                ))
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
            )?))
        } else {
            None
        })
    }
}

fn deduplicate_by_module_path(mut artifacts: Vec<ContractArtifact>) -> Vec<ContractArtifact> {
    artifacts.sort_by(|a, b| a.module_path.cmp(&b.module_path));
    artifacts.dedup_by(|a, b| a.module_path == b.module_path);
    artifacts
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ScarbCommand;
    use crate::tests::setup_package;
    use assert_fs::TempDir;
    use assert_fs::fixture::{FileWriteStr, PathChild};
    use camino::Utf8PathBuf;
    use indoc::indoc;

    #[test]
    fn test_deduplicate_by_module_path() {
        let artifacts = vec![
            ContractArtifact {
                name: "HelloStarknet".to_string(),
                module_path: "pkg::HelloStarknet".to_string(),
                sierra_path: Utf8PathBuf::from(
                    "pkg_unittest_HelloStarknet.test.contract_class.json",
                ),
            },
            ContractArtifact {
                name: "HelloStarknet".to_string(),
                module_path: "pkg::HelloStarknet".to_string(),
                sierra_path: Utf8PathBuf::from(
                    "pkg_integrationtest_HelloStarknet.test.contract_class.json",
                ),
            },
            ContractArtifact {
                name: "ERC20".to_string(),
                module_path: "pkg::ERC20".to_string(),
                sierra_path: Utf8PathBuf::from("pkg_unittest_ERC20.test.contract_class.json"),
            },
        ];

        let result = deduplicate_by_module_path(artifacts);

        let module_paths: Vec<&str> = result
            .iter()
            .map(|artifact| artifact.module_path.as_str())
            .collect();
        assert_eq!(module_paths, vec!["pkg::ERC20", "pkg::HelloStarknet",]);
    }

    fn setup_with_tests(tests_contents: &str) -> (TempDir, StarknetArtifactsFiles) {
        let temp = setup_package("basic_package");
        let tests_dir = temp.join("tests");
        fs::create_dir(&tests_dir).unwrap();

        temp.child(tests_dir.join("test.cairo"))
            .write_str(tests_contents)
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

    fn setup() -> (TempDir, StarknetArtifactsFiles) {
        setup_with_tests(indoc!(
            r"
                #[test]
                fn mock_test() {
                    assert!(true);
                }
            "
        ))
    }

    fn count_by_name(contracts: &HashMap<String, ContractData>, contract_name: &str) -> usize {
        contracts
            .values()
            .filter(|contract| contract.name == contract_name)
            .count()
    }

    #[test]
    fn test_load_contracts_artifacts() {
        let (_temp, artifacts_files) = setup();

        // Load the contracts
        let result = artifacts_files.load_contracts_artifacts().unwrap();

        // Both `src` contracts are unambiguous: each name resolves to a single contract, even
        // though they are emitted into both the unittest and integrationtest targets (identical
        // module paths are deduplicated).
        assert_eq!(count_by_name(&result, "ERC20"), 1);
        assert_eq!(count_by_name(&result, "HelloStarknet"), 1);
    }

    #[test]
    fn test_load_contracts_artifacts_keeps_duplicate_names() {
        // A second `HelloStarknet` defined in `tests/` collides by name with the one in `src/`,
        // but has a distinct fully qualified `module_path`, so both are kept as separate entries.
        let (_temp, artifacts_files) = setup_with_tests(indoc!(
            r"
                #[starknet::contract]
                mod HelloStarknet {
                    #[storage]
                    struct Storage {
                        counter: felt252,
                    }
                }

                #[test]
                fn mock_test() {
                    assert!(true);
                }
            "
        ));

        let result = artifacts_files.load_contracts_artifacts().unwrap();

        // The ambiguous name is kept twice, under two distinct module paths.
        assert_eq!(count_by_name(&result, "HelloStarknet"), 2);

        // A uniquely named contract stays unambiguous.
        assert_eq!(count_by_name(&result, "ERC20"), 1);
    }

    #[test]
    #[cfg(feature = "cairo-native")]
    fn test_load_contracts_artifacts_native() {
        let (_temp, artifacts_files) = setup();

        let artifacts_files = artifacts_files.compile_native(true);

        // Load the contracts
        let result = artifacts_files.load_contracts_artifacts().unwrap();

        // Assert the Contract Artifacts are loaded.
        assert_eq!(count_by_name(&result, "ERC20"), 1);
        assert_eq!(count_by_name(&result, "HelloStarknet"), 1);
        let erc20 = result
            .values()
            .find(|contract| contract.name == "ERC20")
            .unwrap();
        assert!(erc20.artifacts.executor.is_some());
    }
}
