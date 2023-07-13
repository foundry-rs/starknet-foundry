use anyhow::{anyhow, Context, Result};
use assert_fs::fixture::{FileTouch, FileWriteStr, PathChild};
use assert_fs::TempDir;
use cairo_lang_compiler::db::RootDatabase;
use cairo_lang_compiler::project::setup_project;
use cairo_lang_compiler::CompilerConfig;
use cairo_lang_filesystem::db::init_dev_corelib;
use cairo_lang_starknet::allowed_libfuncs::{validate_compatible_sierra_version, ListSelector};
use cairo_lang_starknet::contract_class::compile_contract_in_prepared_db;
use cairo_lang_starknet::plugin::StarkNetPlugin;
use camino::Utf8PathBuf;
use forge::scarb::StarknetContractArtifacts;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use test_collector::LinkedLibrary;

#[derive(Debug, Clone)]
pub struct Contract {
    name: String,
    code: String,
}

impl Contract {
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

    fn generate_sierra(self, corelib_path: &Path) -> Result<String> {
        let path = TempDir::new()?;
        let contract_path = path.child("contract.cairo");
        contract_path.touch()?;
        contract_path.write_str(&self.code)?;

        let allowed_libfuncs_list = Some(ListSelector::default());

        let db = &mut {
            RootDatabase::builder()
                .with_semantic_plugin(Arc::new(StarkNetPlugin::default()))
                .build()?
        };
        init_dev_corelib(db, corelib_path.to_path_buf());

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

        Ok(sierra)
    }
}

#[derive(Debug)]
pub struct TestCase {
    dir: TempDir,
    contracts: Vec<Contract>,
}

impl<'a> TestCase {
    const TEST_PATH: &'a str = "test_case.cairo";
    const PACKAGE_NAME: &'a str = "my_package";

    pub fn from(test_code: &str, contracts: Vec<Contract>) -> Result<Self> {
        let dir = TempDir::new()?;
        let test_file = dir.child(Self::TEST_PATH);
        test_file.touch()?;
        test_file.write_str(test_code)?;

        dir.child("src/lib.cairo").touch().unwrap();

        Ok(Self { dir, contracts })
    }

    pub fn path(&self) -> Result<Utf8PathBuf> {
        Utf8PathBuf::from_path_buf(self.dir.path().to_path_buf())
            .map_err(|_| anyhow!("Failed to convert TestCase path to Utf8PathBuf"))
    }

    pub fn linked_libraries(&self) -> Vec<LinkedLibrary> {
        vec![LinkedLibrary {
            name: Self::PACKAGE_NAME.to_string(),
            path: self.dir.path().join("src"),
        }]
    }

    pub fn contracts(
        &self,
        corelib_path: &Path,
    ) -> Result<HashMap<String, StarknetContractArtifacts>> {
        self.contracts
            .clone()
            .into_iter()
            .map(|contract| {
                Ok((
                    contract.name.clone(),
                    StarknetContractArtifacts {
                        sierra: contract.generate_sierra(corelib_path)?,
                        casm: None,
                    },
                ))
            })
            .collect()
    }
}

#[macro_export]
macro_rules! test_case {
    ( $test_code:expr ) => ({
        use $crate::common::runner::TestCase;
        TestCase::from($test_code, vec![]).unwrap()
    });
    ( $test_code:expr, $( $contract:expr ),*) => ({
        use $crate::common::runner::TestCase;

        let contracts = vec![$($contract,)*];
        TestCase::from($test_code, contracts).unwrap()
    });
}

#[macro_export]
macro_rules! assert_passed {
    ($result:expr) => {{
        assert!($result.iter().all(|result| {
            result
                .test_case_summaries
                .iter()
                .all(|r| matches!(r, TestCaseSummary::Passed { .. }))
        }));
    }};
}

#[macro_export]
macro_rules! assert_failed {
    ($result:expr) => {{
        assert!($result.iter().all(|result| {
            result
                .test_case_summaries
                .iter()
                .all(|r| matches!(r, TestCaseSummary::Failed { .. }))
        }));
    }};
}
