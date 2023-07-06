use anyhow::{anyhow, Result};
use assert_fs::fixture::{FileTouch, FileWriteStr, PathChild};
use assert_fs::TempDir;
use camino::Utf8PathBuf;
use std::path::PathBuf;
use test_collector::LinkedLibrary;

pub struct TestCase {
    dir: TempDir,
    _contract_paths: Vec<PathBuf>,
}

impl<'a> TestCase {
    const TEST_PATH: &'a str = "test_case.cairo";
    const PACKAGE_NAME: &'a str = "my_package";

    pub fn from(test_code: &str, contract_paths: Vec<PathBuf>) -> Result<Self> {
        let dir = TempDir::new()?;
        let test_file = dir.child(Self::TEST_PATH);
        test_file.touch()?;
        test_file.write_str(test_code)?;

        dir.child("src/lib.cairo").touch().unwrap();

        Ok(Self {
            dir,
            _contract_paths: contract_paths,
        })
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
}

#[macro_export]
macro_rules! test_case {
    ($test_code:expr) => {{
        use $crate::common::runner::TestCase;

        let test = TestCase::from($test_code, vec![]).unwrap();
        test
    }};
}

#[macro_export]
macro_rules! assert_passed {
    ($result:expr) => {{
        assert!($result.iter().all(|result| {
            result
                .test_unit_summaries
                .iter()
                .all(forge::TestUnitSummary::passed)
        }));
    }};
}
