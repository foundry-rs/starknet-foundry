use anyhow::{anyhow, Context, Result};
use assert_fs::fixture::{FileTouch, FileWriteStr, PathChild};
use assert_fs::TempDir;
use cairo_lang_compiler::db::RootDatabase;
use cairo_lang_compiler::project::setup_project;
use cairo_lang_compiler::CompilerConfig;
use cairo_lang_filesystem::db::init_dev_corelib;
use cairo_lang_starknet::allowed_libfuncs::{validate_compatible_sierra_version, ListSelector};
use cairo_lang_starknet::casm_contract_class::CasmContractClass;
use cairo_lang_starknet::contract_class::compile_contract_in_prepared_db;
use cairo_lang_starknet::inline_macros::selector::SelectorMacro;
use cairo_lang_starknet::plugin::StarkNetPlugin;
use camino::{Utf8Path, Utf8PathBuf};
use forge::scarb::StarknetContractArtifacts;
use forge::{CrateLocation, TestCrateSummary};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use test_collector::LinkedLibrary;

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

    fn generate_sierra_and_casm(self, corelib_path: &Utf8Path) -> Result<(String, String)> {
        let path = TempDir::new()?;
        let contract_path = path.child("contract.cairo");
        contract_path.touch()?;
        contract_path.write_str(&self.code)?;

        let allowed_libfuncs_list = Some(ListSelector::default());

        let db = &mut {
            RootDatabase::builder()
                .with_macro_plugin(Arc::new(StarkNetPlugin::default()))
                .with_inline_macro_plugin(SelectorMacro::NAME, Arc::new(SelectorMacro))
                .build()?
        };
        init_dev_corelib(db, corelib_path.into());

        let main_crate_ids = setup_project(db, Path::new(&contract_path.path()))?;

        let contract =
            compile_contract_in_prepared_db(db, None, main_crate_ids, CompilerConfig::default())?;

        validate_compatible_sierra_version(
            &contract,
            if let Some(allowed_libfuncs_list) = allowed_libfuncs_list {
                allowed_libfuncs_list
            } else {
                ListSelector::default()
            },
        )?;

        let sierra =
            serde_json::to_string_pretty(&contract).with_context(|| "Serialization failed.")?;

        let casm = CasmContractClass::from_contract_class(contract, true)?;
        let casm = serde_json::to_string_pretty(&casm)?;

        Ok((sierra, casm))
    }
}

#[derive(Debug)]
pub struct TestCase {
    dir: TempDir,
    contracts: Vec<Contract>,
    enviroment_variables: HashMap<String, String>,
}

impl<'a> TestCase {
    pub const TEST_PATH: &'a str = "tests/test_case.cairo";
    const PACKAGE_NAME: &'a str = "my_package";

    pub fn from(test_code: &str, contracts: Vec<Contract>) -> Result<Self> {
        let dir = TempDir::new()?;
        let test_file = dir.child(Self::TEST_PATH);
        test_file.touch()?;
        test_file.write_str(test_code)?;

        dir.child("src/lib.cairo").touch().unwrap();

        Ok(Self {
            dir,
            contracts,
            enviroment_variables: HashMap::new(),
        })
    }

    pub fn set_env(&mut self, key: &str, value: &str) {
        self.enviroment_variables.insert(key.into(), value.into());
    }

    #[must_use]
    pub fn env(&self) -> &HashMap<String, String> {
        &self.enviroment_variables
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

    pub fn contracts(
        &self,
        corelib_path: &Utf8Path,
    ) -> Result<HashMap<String, StarknetContractArtifacts>> {
        self.contracts
            .clone()
            .into_iter()
            .map(|contract| {
                let name = contract.name.clone();
                let (sierra, casm) = contract.generate_sierra_and_casm(corelib_path)?;

                Ok((name, StarknetContractArtifacts { sierra, casm }))
            })
            .collect()
    }

    #[must_use]
    pub fn find_test_result(results: &[TestCrateSummary]) -> &TestCrateSummary {
        results
            .iter()
            .find(|r| r.test_crate_type == CrateLocation::Tests)
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
        use forge::test_case_summary::TestCaseSummary;
        use $crate::runner::TestCase;

        let result = TestCase::find_test_result(&$result);
        assert!(
            !result.test_case_summaries.is_empty(),
            "No test results found"
        );
        assert!(
            result
                .test_case_summaries
                .iter()
                .all(|r| matches!(r, TestCaseSummary::Passed { .. })),
            "Some tests didn't pass"
        );
    }};
}

#[macro_export]
macro_rules! assert_failed {
    ($result:expr) => {{
        use forge::test_case_summary::TestCaseSummary;

        use $crate::runner::TestCase;

        let result = TestCase::find_test_result(&$result);
        assert!(
            !result.test_case_summaries.is_empty(),
            "No test results found"
        );
        assert!(
            result
                .test_case_summaries
                .iter()
                .all(|r| matches!(r, TestCaseSummary::Failed { .. })),
            "Some tests didn't fail"
        );
    }};
}

#[macro_export]
macro_rules! assert_case_output_contains {
    ($result:expr, $test_case_name:expr, $asserted_msg:expr) => {{
        use forge::test_case_summary::TestCaseSummary;

        use $crate::runner::TestCase;

        let test_case_name = $test_case_name;
        let test_name_suffix = format!("::{test_case_name}");

        let result = TestCase::find_test_result(&$result);

        assert!(result.test_case_summaries.iter().any(|case| {
            match case {
                TestCaseSummary::Failed {
                    msg: Some(msg),
                    name,
                    ..
                }
                | TestCaseSummary::Passed {
                    msg: Some(msg),
                    name,
                    ..
                } => msg.contains($asserted_msg) && name.ends_with(test_name_suffix.as_str()),
                _ => false,
            }
        }));
    }};
}
