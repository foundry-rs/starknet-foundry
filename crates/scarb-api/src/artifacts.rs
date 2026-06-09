use anyhow::Result;

use crate::artifacts::representation::StarknetArtifactsRepresentation;
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

/// A single compiled contract together with its artifacts and Sierra path.
#[derive(Debug, Clone, PartialEq)]
pub struct ContractData {
    pub contract_name: String,
    pub artifacts: StarknetContractArtifacts,
    pub sierra_path: Utf8PathBuf,
}

/// A mapping of module paths to their corresponding contract data.
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
        // Gather `(contract_name, module_path, sierra_path)` across the base and all other
        // representations. The same contract may be emitted into several targets (e.g. unittest
        // and integrationtest) with an identical `module_path`; those are not real duplicates,
        // so we deduplicate by `module_path`. Distinct module_paths sharing a `contract_name`
        // are genuine duplicates and are all kept here, keyed by their unique module_path.
        let mut all_artifacts: Vec<(String, String, Utf8PathBuf)> =
            StarknetArtifactsRepresentation::try_from_path(self.base.as_path())?.artifacts();

        for path in &self.other {
            let representation = StarknetArtifactsRepresentation::try_from_path(path.as_path())?;
            all_artifacts.extend(representation.artifacts());
        }

        self.compile_artifacts(deduplicate_by_module_path(all_artifacts))
    }

    #[tracing::instrument(skip_all, level = "debug")]
    fn compile_artifacts(
        &self,
        artifacts: Vec<(String, String, Utf8PathBuf)>,
    ) -> Result<HashMap<String, ContractData>> {
        artifacts
            .into_par_iter()
            .map(|(contract_name, module_path, sierra_path)| {
                let artifacts = self.compile_artifact_at_path(&sierra_path)?;
                Ok((
                    module_path,
                    ContractData {
                        contract_name,
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

/// Deduplicates `(contract_name, module_path, sierra_path)` entries by `module_path`.
///
/// The same contract may be emitted into several targets (e.g. unittest and integrationtest)
/// with an identical `module_path`; those are not real duplicates and are collapsed into one.
/// Distinct module_paths sharing a `contract_name` are genuine duplicates and are all kept.
/// Sorting by `module_path` makes both the deduplication and the resulting order deterministic.
fn deduplicate_by_module_path(
    mut artifacts: Vec<(String, String, Utf8PathBuf)>,
) -> Vec<(String, String, Utf8PathBuf)> {
    artifacts.sort_by(|(_, a, _), (_, b, _)| a.cmp(b));
    artifacts.dedup_by(|(_, a, _), (_, b, _)| a == b);
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
            // Same contract emitted into two targets with an identical module_path -> collapsed.
            (
                "HelloStarknet".to_string(),
                "pkg::HelloStarknet".to_string(),
                Utf8PathBuf::from("unittest/HelloStarknet.sierra"),
            ),
            (
                "HelloStarknet".to_string(),
                "pkg::HelloStarknet".to_string(),
                Utf8PathBuf::from("integrationtest/HelloStarknet.sierra"),
            ),
            // Same name, different module_path -> a genuine duplicate, kept.
            (
                "HelloStarknet".to_string(),
                "pkg_integrationtest::tests::HelloStarknet".to_string(),
                Utf8PathBuf::from("integrationtest/tests_HelloStarknet.sierra"),
            ),
            (
                "ERC20".to_string(),
                "pkg::ERC20".to_string(),
                Utf8PathBuf::from("unittest/ERC20.sierra"),
            ),
        ];

        let result = deduplicate_by_module_path(artifacts);

        // Identical module_paths are collapsed; distinct ones are kept, ordered by module_path.
        let module_paths: Vec<&str> = result.iter().map(|(_, path, _)| path.as_str()).collect();
        assert_eq!(
            module_paths,
            vec![
                "pkg::ERC20",
                "pkg::HelloStarknet",
                "pkg_integrationtest::tests::HelloStarknet",
            ]
        );

        // The ambiguous name survives twice (two distinct module paths).
        let hello_starknet_count = result
            .iter()
            .filter(|(name, _, _)| name == "HelloStarknet")
            .count();
        assert_eq!(hello_starknet_count, 2);
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
            .filter(|contract| contract.contract_name == contract_name)
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
        // but has a distinct fully qualified module_path, so both are kept as separate entries.
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
            .find(|contract| contract.contract_name == "ERC20")
            .unwrap();
        assert!(erc20.artifacts.executor.is_some());
    }
}
