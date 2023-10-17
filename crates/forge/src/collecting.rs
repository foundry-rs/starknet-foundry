use crate::{RunnerConfig, RunnerParams, BUILTINS};
use anyhow::{anyhow, Context, Result};
use assert_fs::fixture::{FileTouch, PathChild, PathCopy};
use assert_fs::TempDir;
use cairo_lang_sierra::program::Program;
use camino::{Utf8Path, Utf8PathBuf};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use test_collector::{collect_tests, LinkedLibrary, TestCase};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct CompiledTestCrate {
    pub sierra_program: Program,
    pub test_cases: Vec<TestCase>,
    pub tests_location: CrateLocation,
}

impl CompiledTestCrate {
    pub fn filter_by_name(self, filter: &str) -> Self {
        let test_cases = self
            .test_cases
            .into_iter()
            .filter(|tc| tc.name.contains(filter))
            .collect();
        Self { test_cases, ..self }
    }

    pub fn filter_by_exact_name(self, filter: &str) -> Self {
        let test_cases = self
            .test_cases
            .into_iter()
            .filter(|tc| tc.name == filter)
            .collect();
        Self { test_cases, ..self }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CrateLocation {
    /// Main crate in a package
    Lib,
    /// Crate in the `tests/` directory
    Tests,
}

#[derive(Debug, PartialEq)]
pub struct TestCompilationTarget {
    pub crate_root: Utf8PathBuf,
    pub crate_name: String,
    pub crate_location: CrateLocation,
}

impl TestCompilationTarget {
    pub fn compile_tests(
        &self,
        linked_libraries: &[LinkedLibrary],
        corelib_path: &Utf8Path,
    ) -> Result<CompiledTestCrate> {
        let (sierra_program, test_cases) = collect_tests(
            self.crate_root.as_str(),
            None,
            &self.crate_name,
            linked_libraries,
            &BUILTINS,
            corelib_path.into(),
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
    temp_dir: &TempDir,
) -> Result<Vec<TestCompilationTarget>> {
    let tests_dir_path = package_path.join("tests");

    let test_dir_crate = if tests_dir_path.exists() {
        let lib_path = tests_dir_path.join("lib.cairo");
        if lib_path.exists() {
            Some(TestCompilationTarget {
                crate_root: tests_dir_path,
                crate_name: "tests".to_string(),
                crate_location: CrateLocation::Tests,
            })
        } else {
            Some(pack_tests_into_single_crate(temp_dir, &tests_dir_path)?)
        }
    } else {
        None
    };

    let mut compilation_targets = vec![TestCompilationTarget {
        crate_root: package_source_dir_path.to_path_buf(),
        crate_name: package_name.to_string(),
        crate_location: CrateLocation::Lib,
    }];
    if let Some(test_dir_crate) = test_dir_crate {
        compilation_targets.push(test_dir_crate);
    }

    Ok(compilation_targets)
}

pub fn compile_tests(
    targets: &Vec<TestCompilationTarget>,
    runner_params: &RunnerParams,
) -> Result<Vec<CompiledTestCrate>> {
    targets
        .par_iter()
        .map(|target| {
            target.compile_tests(&runner_params.linked_libraries, &runner_params.corelib_path)
        })
        .collect()
}

pub fn filter_tests_from_crates(
    compiled_test_crates: Vec<CompiledTestCrate>,
    runner_config: &RunnerConfig,
) -> Vec<CompiledTestCrate> {
    if let Some(test_name_filter) = &runner_config.test_name_filter {
        compiled_test_crates
            .into_iter()
            .map(|tc| {
                if runner_config.exact_match {
                    tc.filter_by_exact_name(test_name_filter)
                } else {
                    tc.filter_by_name(test_name_filter)
                }
            })
            .collect()
    } else {
        compiled_test_crates
    }
}

fn pack_tests_into_single_crate(
    tmp_dir: &TempDir,
    tests_folder_path: &Utf8Path,
) -> Result<TestCompilationTarget> {
    tmp_dir
        .copy_from(tests_folder_path, &["**/*.cairo"])
        .context("Unable to copy files to temporary directory")?;

    let tests_lib_path = tmp_dir.child("lib.cairo");
    assert!(
        !(tests_lib_path.try_exists()?),
        "Path = {:?} already exists",
        tests_lib_path.path()
    );
    tests_lib_path.touch()?;

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

    std::fs::write(tests_lib_path, content).context("Failed to write to tests lib file")?;

    let tests_tmp_dir_path = Utf8PathBuf::from_path_buf(tmp_dir.to_path_buf())
        .map_err(|_| anyhow!("Failed to convert tests temporary directory to Utf8PathBuf"))?;

    Ok(TestCompilationTarget {
        crate_root: tests_tmp_dir_path,
        crate_name: "tests".to_string(),
        crate_location: CrateLocation::Tests,
    })
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
    use assert_fs::fixture::PathCopy;
    use assert_fs::TempDir;
    use test_collector::ExpectedTestResult;

    #[test]
    fn collecting_test_compilation_targets() {
        let temp = TempDir::new().unwrap();
        temp.copy_from("tests/data/simple_package", &["**/*.cairo", "**/*.toml"])
            .unwrap();
        let package_path = Utf8PathBuf::from_path_buf(temp.to_path_buf()).unwrap();

        let compilation_targets = collect_test_compilation_targets(
            &package_path,
            "simple_package",
            &package_path,
            &TempDir::new().unwrap(),
        )
        .unwrap();

        assert!(compilation_targets.contains(&TestCompilationTarget {
            crate_root: package_path,
            crate_name: "simple_package".to_string(),
            crate_location: CrateLocation::Lib,
        }));
        assert!(compilation_targets
            .iter()
            .any(|tc| tc.crate_name == "tests" && tc.crate_location == CrateLocation::Tests));
    }

    #[test]
    fn packing_tests() {
        let temp = TempDir::new().unwrap();
        temp.copy_from("tests/data/simple_package", &["**/*.cairo", "**/*.toml"])
            .unwrap();
        let package_path = Utf8PathBuf::from_path_buf(temp.to_path_buf()).unwrap();
        let tests_path = package_path.join("tests");

        let temp_dir = TempDir::new().unwrap();
        let tests = pack_tests_into_single_crate(&temp_dir, &tests_path).unwrap();
        let virtual_lib_path = tests.crate_root.join("lib.cairo");
        let virtual_lib_u8_content = std::fs::read(&virtual_lib_path).unwrap();
        let virtual_lib_content = std::str::from_utf8(&virtual_lib_u8_content).unwrap();

        assert!(virtual_lib_path.try_exists().unwrap());
        assert!(virtual_lib_content.contains("mod contract;"));
        assert!(virtual_lib_content.contains("mod ext_function_test;"));
        assert!(virtual_lib_content.contains("mod test_simple;"));
        assert!(virtual_lib_content.contains("mod without_prefix;"));
    }

    fn program_for_testing() -> Program {
        Program {
            type_declarations: vec![],
            libfunc_declarations: vec![],
            statements: vec![],
            funcs: vec![],
        }
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn filtering_tests() {
        let mocked_tests = CompiledTestCrate {
            sierra_program: program_for_testing(),
            test_cases: vec![
                TestCase {
                    name: "crate1::do_thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "crate2::run_other_thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "outer::crate2::execute_next_thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
            ],
            tests_location: CrateLocation::Lib,
        };

        let filtered = mocked_tests.clone().filter_by_name("do");
        assert_eq!(
            filtered.test_cases,
            vec![TestCase {
                name: "crate1::do_thing".to_string(),
                available_gas: None,
                expected_result: ExpectedTestResult::Success,
                fork_config: None,
                fuzzer_config: None
            },]
        );

        let filtered = mocked_tests.clone().filter_by_name("run");
        assert_eq!(
            filtered.test_cases,
            vec![TestCase {
                name: "crate2::run_other_thing".to_string(),
                available_gas: None,
                expected_result: ExpectedTestResult::Success,
                fork_config: None,
                fuzzer_config: None
            },]
        );

        let filtered = mocked_tests.clone().filter_by_name("thing");
        assert_eq!(
            filtered.test_cases,
            vec![
                TestCase {
                    name: "crate1::do_thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "crate2::run_other_thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "outer::crate2::execute_next_thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
            ]
        );

        let filtered = mocked_tests.clone().filter_by_name("nonexistent");
        assert_eq!(filtered.test_cases, vec![]);

        let filtered = mocked_tests.clone().filter_by_name("");
        assert_eq!(
            filtered.test_cases,
            vec![
                TestCase {
                    name: "crate1::do_thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "crate2::run_other_thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "outer::crate2::execute_next_thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
            ]
        );
    }

    #[test]
    fn filtering_with_no_tests() {
        let mocked_tests = CompiledTestCrate {
            sierra_program: program_for_testing(),
            test_cases: vec![],
            tests_location: CrateLocation::Lib,
        };
        let filtered = mocked_tests.clone().filter_by_name("");
        assert_eq!(filtered.test_cases, vec![]);
        let filtered = mocked_tests.clone().filter_by_name("thing");
        assert_eq!(filtered.test_cases, vec![]);

        let filtered = mocked_tests.clone().filter_by_exact_name("");
        assert_eq!(filtered.test_cases, vec![]);
        let filtered = mocked_tests.clone().filter_by_exact_name("thing");
        assert_eq!(filtered.test_cases, vec![]);
    }

    #[test]
    fn filtering_tests_uses_whole_path() {
        let mocked_tests = CompiledTestCrate {
            sierra_program: program_for_testing(),
            test_cases: vec![
                TestCase {
                    name: "crate1::do_thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "crate2::run_other_thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "outer::crate2::run_other_thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
            ],
            tests_location: CrateLocation::Tests,
        };

        let filtered = mocked_tests.filter_by_name("crate2::");
        assert_eq!(
            filtered.test_cases,
            vec![
                TestCase {
                    name: "crate2::run_other_thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "outer::crate2::run_other_thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
            ]
        );
    }

    #[test]
    fn filtering_with_exact_match() {
        let mocked_tests = CompiledTestCrate {
            sierra_program: program_for_testing(),
            test_cases: vec![
                TestCase {
                    name: "crate1::do_thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "crate2::run_other_thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "outer::crate3::run_other_thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "do_thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
            ],
            tests_location: CrateLocation::Tests,
        };

        let filtered = mocked_tests.clone().filter_by_exact_name("");
        assert_eq!(filtered.test_cases, vec![]);

        let filtered = mocked_tests.clone().filter_by_exact_name("thing");
        assert_eq!(filtered.test_cases, vec![]);

        let filtered = mocked_tests.clone().filter_by_exact_name("do_thing");
        assert_eq!(
            filtered.test_cases,
            vec![TestCase {
                name: "do_thing".to_string(),
                available_gas: None,
                expected_result: ExpectedTestResult::Success,
                fork_config: None,
                fuzzer_config: None,
            },]
        );

        let filtered = mocked_tests
            .clone()
            .filter_by_exact_name("crate1::do_thing");
        assert_eq!(
            filtered.test_cases,
            vec![TestCase {
                name: "crate1::do_thing".to_string(),
                available_gas: None,
                expected_result: ExpectedTestResult::Success,
                fork_config: None,
                fuzzer_config: None,
            },]
        );

        let filtered = mocked_tests
            .clone()
            .filter_by_exact_name("crate3::run_other_thing");
        assert_eq!(filtered.test_cases, vec![]);

        let filtered = mocked_tests
            .clone()
            .filter_by_exact_name("outer::crate3::run_other_thing");
        assert_eq!(
            filtered.test_cases,
            vec![TestCase {
                name: "outer::crate3::run_other_thing".to_string(),
                available_gas: None,
                expected_result: ExpectedTestResult::Success,
                fork_config: None,
                fuzzer_config: None,
            },]
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
