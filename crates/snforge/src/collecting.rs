use crate::{CrateLocation, RunnerParams, BUILTINS};
use anyhow::{anyhow, Context, Result};
use cairo_lang_sierra::program::Program;
use camino::{Utf8Path, Utf8PathBuf};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use starknet::core::types::BlockId;
use test_collector::{collect_tests, ForkConfig, LinkedLibrary, RawForkConfig, TestCase};
use url::Url;
use walkdir::WalkDir;

pub(crate) type CompiledTestCrateRaw = CompiledTestCrate<RawForkConfig>;
pub(crate) type CompiledTestCrateRunnable = CompiledTestCrate<ValidatedForkConfig>;

pub(crate) type TestCaseRunnable = TestCase<ValidatedForkConfig>;

#[derive(Debug, Clone)]
pub struct CompiledTestCrate<T: ForkConfig> {
    pub sierra_program: Program,
    pub test_cases: Vec<TestCase<T>>,
    pub tests_location: CrateLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValidatedForkConfig {
    pub url: Url,
    pub block_id: BlockId,
}

impl ForkConfig for ValidatedForkConfig {}

#[derive(Debug, PartialEq)]
pub struct TestCompilationTarget {
    pub crate_root: Utf8PathBuf,
    pub crate_name: String,
    pub crate_location: CrateLocation,
    pub lib_content: String,
}

impl TestCompilationTarget {
    pub fn compile_tests(
        &self,
        linked_libraries: &[LinkedLibrary],
        corelib_path: &Utf8Path,
    ) -> Result<CompiledTestCrateRaw> {
        let (sierra_program, test_cases) = collect_tests(
            &self.crate_name,
            self.crate_root.as_std_path(),
            &self.lib_content,
            linked_libraries,
            &BUILTINS,
            corelib_path.into(),
            None,
        )?;

        Ok(CompiledTestCrate {
            sierra_program,
            test_cases,
            tests_location: self.crate_location,
        })
    }
}

pub fn collect_test_compilation_targets(
    package_path: &Utf8Path,
    package_name: &str,
    package_source_dir_path: &Utf8Path,
) -> Result<Vec<TestCompilationTarget>> {
    let mut compilation_targets = vec![TestCompilationTarget {
        crate_root: package_source_dir_path.to_path_buf(),
        crate_name: package_name.to_string(),
        crate_location: CrateLocation::Lib,
        lib_content: std::fs::read_to_string(package_source_dir_path.join("lib.cairo"))?,
    }];

    let tests_dir_path = package_path.join("tests");
    if tests_dir_path.exists() {
        compilation_targets.push(TestCompilationTarget {
            crate_name: "tests".to_string(),
            crate_location: CrateLocation::Tests,
            lib_content: get_or_create_test_lib_content(tests_dir_path.as_path())?,
            crate_root: tests_dir_path,
        });
    }

    Ok(compilation_targets)
}

pub(crate) fn compile_tests(
    targets: &Vec<TestCompilationTarget>,
    runner_params: &RunnerParams,
) -> Result<Vec<CompiledTestCrateRaw>> {
    targets
        .par_iter()
        .map(|target| {
            target.compile_tests(&runner_params.linked_libraries, &runner_params.corelib_path)
        })
        .collect()
}

fn get_or_create_test_lib_content(tests_folder_path: &Utf8Path) -> Result<String> {
    let tests_lib_path = tests_folder_path.join("lib.cairo");
    if tests_lib_path.try_exists()? {
        return Ok(std::fs::read_to_string(tests_lib_path)?);
    }

    let mut content = String::new();
    for entry in WalkDir::new(tests_folder_path)
        .max_depth(1)
        .sort_by_file_name()
    {
        let entry = entry
            .with_context(|| format!("Failed to read directory at path = {tests_folder_path}"))?;
        let path = Utf8Path::from_path(entry.path())
            .ok_or_else(|| anyhow!("Failed to convert path = {:?} to Utf8Path", entry.path()))?;

        if path.is_file() && path.extension().unwrap_or_default() == "cairo" {
            let mod_name = module_name_from_path(tests_folder_path, path);

            content.push_str(&format!("mod {mod_name};\n"));
        }
    }
    Ok(content)
}

fn module_name_from_path<'a>(tests_folder_path: &Utf8Path, path: &'a Utf8Path) -> &'a str {
    path.strip_prefix(tests_folder_path)
        .unwrap_or_else(|_| {
            panic!(
                "Path to test = {path} does not start with test_folder_path = {tests_folder_path}"
            )
        })
        .as_str()
        .strip_suffix(".cairo")
        .unwrap_or_else(|| panic!("Path to test = {path} should have .cairo extension"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CrateLocation;
    use assert_fs::fixture::PathCopy;
    use assert_fs::TempDir;

    #[test]
    fn collecting_test_compilation_targets() {
        let temp = TempDir::new().unwrap();
        temp.copy_from("tests/data/simple_package", &["**/*.cairo", "**/*.toml"])
            .unwrap();
        let package_path = Utf8PathBuf::from_path_buf(temp.to_path_buf()).unwrap();
        let package_source_dir_path = package_path.join("src");

        let compilation_targets = collect_test_compilation_targets(
            &package_path,
            "simple_package",
            &package_source_dir_path,
        )
        .unwrap();

        assert_eq!(
            compilation_targets,
            vec![
                TestCompilationTarget {
                    crate_root: package_source_dir_path,
                    crate_name: "simple_package".to_string(),
                    crate_location: CrateLocation::Lib,
                    lib_content: std::fs::read_to_string("tests/data/simple_package/src/lib.cairo").unwrap(),
                },
                TestCompilationTarget {
                    crate_root: package_path.join("tests"),
                    crate_name: "tests".to_string(),
                    crate_location: CrateLocation::Tests,
                    lib_content: "mod contract;\nmod ext_function_test;\nmod test_simple;\nmod without_prefix;\n".to_string(),
                }
            ]
        );
    }

    #[test]
    fn module_name() {
        let path = Utf8PathBuf::from("a/b/c");
        let file_path = path.join("e.cairo");

        let module_name = module_name_from_path(&path, &file_path);
        assert_eq!(module_name, "e");
    }

    #[test]
    #[should_panic(expected = "does not start with test_folder_path")]
    fn module_name_non_related_paths() {
        let path = Utf8PathBuf::from("a/b/c");
        let file_path = Utf8PathBuf::from("e.cairo");

        module_name_from_path(&path, &file_path);
    }

    #[test]
    #[should_panic(expected = "should have .cairo extension")]
    fn module_name_wrong_extension() {
        let path = Utf8PathBuf::from("a/b/c");
        let file_path = path.join("e.txt");

        module_name_from_path(&path, &file_path);
    }
}
