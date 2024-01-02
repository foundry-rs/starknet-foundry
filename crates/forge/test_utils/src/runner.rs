use anyhow::{anyhow, bail, Context, Result};
use assert_fs::fixture::{FileTouch, FileWriteStr, PathChild};
use assert_fs::TempDir;
use camino::Utf8PathBuf;
use forge_runner::test_crate_summary::TestCrateSummary;
use indoc::formatdoc;
use scarb_api::{get_contracts_map, StarknetContractArtifacts};
use scarb_metadata::MetadataCommand;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::str::FromStr;

use crate::tempdir_with_tool_versions;

/// Represents a dependency of a Cairo project
#[derive(Debug, Clone)]
pub struct LinkedLibrary {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct Contract {
    name: String,
    code: String,
}

impl Contract {
    #[must_use]
    pub fn new(name: &str, code: &str) -> Self {
        Self {
            name: name.to_string(),
            code: code.to_string(),
        }
    }

    pub fn from_code_path(name: String, path: &Path) -> Result<Self> {
        let code = fs::read_to_string(path)?;
        Ok(Self { name, code })
    }

    fn generate_sierra_and_casm(self) -> Result<(String, String)> {
        let dir = tempdir_with_tool_versions()?;

        let contract_path = dir.child("src/lib.cairo");
        contract_path.touch()?;
        contract_path.write_str(&self.code)?;

        let scarb_toml_path = dir.child("Scarb.toml");
        scarb_toml_path
            .write_str(&formatdoc!(
                r#"
                [package]
                name = "contract"
                version = "0.1.0"

                [[target.starknet-contract]]
                sierra = true

                [dependencies]
                starknet = "2.4.0"
                "#,
            ))
            .unwrap();

        let build_output = Command::new("scarb")
            .current_dir(&dir)
            .arg("build")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .context("Failed to build contracts with Scarb")?;
        if !build_output.status.success() {
            bail!("scarb build did not succeed")
        }

        let scarb_metadata = MetadataCommand::new()
            .current_dir(dir.path())
            .inherit_stderr()
            .exec()?;
        let package = scarb_metadata
            .packages
            .iter()
            .find(|package| package.name == "contract")
            .unwrap();

        Ok(get_contracts_map(&scarb_metadata, &package.id)
            .unwrap()
            .into_values()
            .map(|x| (x.sierra, x.casm))
            .next()
            .unwrap())
    }
}

#[derive(Debug)]
pub struct TestCase {
    dir: TempDir,
    contracts: Vec<Contract>,
    environment_variables: HashMap<String, String>,
}

impl<'a> TestCase {
    pub const TEST_PATH: &'a str = "tests/test_case.cairo";
    const PACKAGE_NAME: &'a str = "my_package";

    pub fn from(test_code: &str, contracts: Vec<Contract>) -> Result<Self> {
        let dir = tempdir_with_tool_versions()?;
        let test_file = dir.child(Self::TEST_PATH);
        test_file.touch()?;
        test_file.write_str(test_code)?;

        dir.child("src/lib.cairo").touch().unwrap();

        let snforge_std_path = Utf8PathBuf::from_str("../../snforge_std")
            .unwrap()
            .canonicalize_utf8()
            .unwrap()
            .to_string()
            .replace('\\', "/");

        let scarb_toml_path = dir.child("Scarb.toml");
        scarb_toml_path
            .write_str(&formatdoc!(
                r#"
                [package]
                name = "test_package"
                version = "0.1.0"

                [[target.starknet-contract]]
                sierra = true
                casm = true

                [dependencies]
                starknet = "2.4.0"
                snforge_std = {{ path = "{}" }}
                "#,
                snforge_std_path
            ))
            .unwrap();

        Ok(Self {
            dir,
            contracts,
            environment_variables: HashMap::new(),
        })
    }

    pub fn set_env(&mut self, key: &str, value: &str) {
        self.environment_variables.insert(key.into(), value.into());
    }

    #[must_use]
    pub fn env(&self) -> &HashMap<String, String> {
        &self.environment_variables
    }

    pub fn path(&self) -> Result<Utf8PathBuf> {
        Utf8PathBuf::from_path_buf(self.dir.path().to_path_buf())
            .map_err(|_| anyhow!("Failed to convert TestCase path to Utf8PathBuf"))
    }

    #[must_use]
    pub fn linked_libraries(&self) -> Vec<LinkedLibrary> {
        let snforge_std_path = PathBuf::from_str("../../snforge_std")
            .unwrap()
            .canonicalize()
            .unwrap();
        vec![
            LinkedLibrary {
                name: Self::PACKAGE_NAME.to_string(),
                path: self.dir.path().join("src"),
            },
            LinkedLibrary {
                name: "snforge_std".to_string(),
                path: snforge_std_path.join("src"),
            },
        ]
    }

    pub fn contracts(&self) -> Result<HashMap<String, StarknetContractArtifacts>> {
        self.contracts
            .clone()
            .into_iter()
            .map(|contract| {
                let name = contract.name.clone();
                let (sierra, casm) = contract.generate_sierra_and_casm()?;

                Ok((name, StarknetContractArtifacts { sierra, casm }))
            })
            .collect()
    }

    #[must_use]
    pub fn find_test_result(results: &[TestCrateSummary]) -> &TestCrateSummary {
        results
            .iter()
            .find(|tc| !tc.test_case_summaries.is_empty())
            .unwrap()
    }
}

#[macro_export]
macro_rules! test_case {
    ( $test_code:expr ) => ({
        use $crate::runner::TestCase;
        TestCase::from($test_code, vec![]).unwrap()
    });
    ( $test_code:expr, $( $contract:expr ),*) => ({
        use $crate::runner::TestCase;

        let contracts = vec![$($contract,)*];
        TestCase::from($test_code, contracts).unwrap()
    });
}

#[macro_export]
macro_rules! assert_passed {
    ($result:expr) => {{
        use forge_runner::test_case_summary::{AnyTestCaseSummary, TestCaseSummary};
        use $crate::runner::TestCase;

        let result = TestCase::find_test_result(&$result);
        assert!(
            !result.test_case_summaries.is_empty(),
            "No test results found"
        );
        assert!(
            result.test_case_summaries.iter().all(|t| t.is_passed()),
            "Some tests didn't pass"
        );
    }};
}

#[macro_export]
macro_rules! assert_failed {
    ($result:expr) => {{
        use forge_runner::test_case_summary::{AnyTestCaseSummary, TestCaseSummary};

        use $crate::runner::TestCase;

        let result = TestCase::find_test_result(&$result);
        assert!(
            !result.test_case_summaries.is_empty(),
            "No test results found"
        );
        assert!(
            result.test_case_summaries.iter().all(|t| t.is_failed()),
            "Some tests didn't fail"
        );
    }};
}

#[macro_export]
macro_rules! assert_case_output_contains {
    ($result:expr, $test_case_name:expr, $asserted_msg:expr) => {{
        use forge_runner::test_case_summary::{AnyTestCaseSummary, TestCaseSummary};

        use $crate::runner::TestCase;

        let test_case_name = $test_case_name;
        let test_name_suffix = format!("::{test_case_name}");

        let result = TestCase::find_test_result(&$result);

        assert!(result.test_case_summaries.iter().any(|any_case| {
            if any_case.is_passed() || any_case.is_failed() {
                return any_case.msg().unwrap().contains($asserted_msg)
                    && any_case
                        .name()
                        .unwrap()
                        .ends_with(test_name_suffix.as_str());
            }
            false
        }));
    }};
}

#[macro_export]
macro_rules! assert_gas {
    ($result:expr, $test_case_name:expr, $asserted_gas:expr) => {{
        use forge_runner::test_case_summary::{AnyTestCaseSummary, TestCaseSummary};
        use $crate::runner::TestCase;

        let test_case_name = $test_case_name;
        let test_name_suffix = format!("::{test_case_name}");

        let result = TestCase::find_test_result(&$result);

        assert!(result.test_case_summaries.iter().any(|any_case| {
            match any_case {
                AnyTestCaseSummary::Fuzzing(case) => {
                    panic!("Cannot use assert_gas! for fuzzing tests")
                }
                AnyTestCaseSummary::Single(case) => match case {
                    TestCaseSummary::Passed { gas_info: gas, .. } => *gas == $asserted_gas,
                    _ => false,
                },
            }
        }));
    }};
}
