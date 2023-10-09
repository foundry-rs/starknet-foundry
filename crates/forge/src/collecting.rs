use crate::{RunnerConfig, RunnerParams, BUILTINS};
use anyhow::{anyhow, Context, Result};
use assert_fs::fixture::{FileTouch, PathChild, PathCopy};
use assert_fs::TempDir;
use cairo_lang_sierra::program::Program;
use camino::Utf8PathBuf;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use test_collector::{collect_tests, LinkedLibrary, TestCase};
use walkdir::WalkDir;

pub struct TestsFromCrate {
    pub sierra_program: Program,
    pub test_cases: Vec<TestCase>,
    pub test_crate_type: TestCrateType,
}

impl TestsFromCrate {
    pub fn filter_by_name(&self, filter: &str, exact_match: bool) -> Self {
        let mut result = vec![];
        for test in &self.test_cases {
            if exact_match {
                if test.name == filter {
                    result.push(test);
                }
            } else if test.name.contains(filter) {
                result.push(test);
            }
        }
        Self {
            sierra_program: self.sierra_program.clone(),
            test_cases: result.into_iter().cloned().collect(),
            test_crate_type: self.test_crate_type,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TestCrateType {
    /// Tests collected from the package
    Lib,
    /// Tests collected from the tests folder
    Tests,
}

#[derive(Debug, PartialEq)]
pub struct TestCrate {
    pub crate_root: Utf8PathBuf,
    pub crate_name: String,
    pub crate_type: TestCrateType,
}

impl TestCrate {
    pub fn compile_tests(
        &self,
        linked_libraries: &[LinkedLibrary],
        corelib_path: &Utf8PathBuf,
    ) -> Result<TestsFromCrate> {
        let (sierra_program, test_cases) = collect_tests(
            self.crate_root.as_str(),
            None,
            &self.crate_name,
            linked_libraries,
            Some(BUILTINS.clone()),
            corelib_path.into(),
        )?;

        Ok(TestsFromCrate {
            sierra_program,
            test_cases,
            test_crate_type: self.crate_type,
        })
    }
}

pub fn collect_test_crates(
    package_path: &Utf8PathBuf,
    package_name: &str,
    package_source_dir_path: &Utf8PathBuf,
    temp_dir: &TempDir,
) -> Result<Vec<TestCrate>> {
    let tests_dir_path = package_path.join("tests");

    let test_dir_crate = if tests_dir_path.exists() {
        let lib_path = tests_dir_path.join("lib.cairo");
        if lib_path.exists() {
            Some(TestCrate {
                crate_root: tests_dir_path,
                crate_name: "tests".to_string(),
                crate_type: TestCrateType::Tests,
            })
        } else {
            Some(pack_tests_into_one_file(temp_dir, &tests_dir_path)?)
        }
    } else {
        None
    };

    let mut test_crates = vec![TestCrate {
        crate_root: package_source_dir_path.clone(),
        crate_name: package_name.to_string(),
        crate_type: TestCrateType::Lib,
    }];
    if let Some(test_dir_crate) = test_dir_crate {
        test_crates.push(test_dir_crate);
    }

    Ok(test_crates)
}

pub fn compile_tests_from_test_crates(
    test_crates: &Vec<TestCrate>,
    runner_params: &RunnerParams,
) -> Result<Vec<TestsFromCrate>> {
    test_crates
        .par_iter()
        .map(|test_crate| {
            test_crate.compile_tests(&runner_params.linked_libraries, &runner_params.corelib_path)
        })
        .collect()
}

pub fn filter_tests_from_crates(
    tests_from_crates: Vec<TestsFromCrate>,
    runner_config: &RunnerConfig,
) -> Vec<TestsFromCrate> {
    if let Some(test_name_filter) = &runner_config.test_name_filter {
        tests_from_crates
            .into_iter()
            .map(|tc| tc.filter_by_name(test_name_filter, runner_config.exact_match))
            .collect()
    } else {
        tests_from_crates
    }
}

pub fn pack_tests_into_one_file(
    tmp_dir: &TempDir,
    tests_folder_path: &Utf8PathBuf,
) -> Result<TestCrate> {
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
        let path = entry.path();

        if path.is_file() && path.extension().unwrap_or_default() == "cairo" {
            let mod_name = path
                .strip_prefix(tests_folder_path)
                .expect("Each test file path should start with package path")
                .to_str()
                .context("Unable to convert test file path to string")?
                .strip_suffix(".cairo")
                .expect("Each test file path should have .cairo extension");

            content.push_str(&format!("mod {mod_name};\n"));
        }
    }

    std::fs::write(tests_lib_path, content).context("Failed to write to tests lib file")?;

    let tests_tmp_dir_path = Utf8PathBuf::from_path_buf(tmp_dir.to_path_buf())
        .map_err(|_| anyhow!("Failed to convert tests temporary directory to Utf8PathBuf"))?;

    Ok(TestCrate {
        crate_root: tests_tmp_dir_path,
        crate_name: "tests".to_string(),
        crate_type: TestCrateType::Tests,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scarb::ForgeConfig;
    use assert_fs::fixture::PathCopy;
    use assert_fs::TempDir;
    use test_collector::ExpectedTestResult;

    #[test]
    fn runner_config_argument_precedence() {
        let config_from_scarb = ForgeConfig {
            exit_first: false,
            fork: vec![],
            fuzzer_runs: Some(1234),
            fuzzer_seed: Some(1000),
        };
        let config = RunnerConfig::new(None, false, true, Some(100), Some(32), &config_from_scarb);
        assert_eq!(
            config,
            RunnerConfig {
                test_name_filter: None,
                exact_match: false,
                exit_first: true,
                fork_targets: vec![],
                fuzzer_runs: 100,
                fuzzer_seed: 32,
            }
        );
    }

    #[test]
    fn collecting_test_crates() {
        let temp = TempDir::new().unwrap();
        temp.copy_from("tests/data/simple_package", &["**/*.cairo", "**/*.toml"])
            .unwrap();
        let package_path = Utf8PathBuf::from_path_buf(temp.to_path_buf()).unwrap();

        let test_crates = collect_test_crates(
            &package_path,
            "simple_package",
            &package_path,
            &TempDir::new().unwrap(),
        )
        .unwrap();

        assert!(test_crates.contains(&TestCrate {
            crate_root: package_path,
            crate_name: "simple_package".to_string(),
            crate_type: TestCrateType::Lib,
        }));
        assert!(test_crates
            .iter()
            .any(|tc| tc.crate_name == "tests" && tc.crate_type == TestCrateType::Tests));
    }

    #[test]
    fn packing_tests() {
        let temp = TempDir::new().unwrap();
        temp.copy_from("tests/data/simple_package", &["**/*.cairo", "**/*.toml"])
            .unwrap();
        let package_path = Utf8PathBuf::from_path_buf(temp.to_path_buf()).unwrap();
        let tests_path = package_path.join("tests");

        let temp_dir = TempDir::new().unwrap();
        let tests = pack_tests_into_one_file(&temp_dir, &tests_path).unwrap();
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
        let mocked_tests = TestsFromCrate {
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
            ],
            test_crate_type: TestCrateType::Lib,
        };

        let filtered = mocked_tests.filter_by_name("do", false);
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

        let filtered = mocked_tests.filter_by_name("run", false);
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

        let filtered = mocked_tests.filter_by_name("thing", false);
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
            ]
        );

        let filtered = mocked_tests.filter_by_name("nonexistent", false);
        assert_eq!(filtered.test_cases, vec![]);

        let filtered = mocked_tests.filter_by_name("", false);
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
            ]
        );
    }

    #[test]
    fn filtering_tests_uses_whole_path() {
        let mocked_tests = TestsFromCrate {
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
            ],
            test_crate_type: TestCrateType::Tests,
        };

        let filtered = mocked_tests.filter_by_name("crate2::", false);
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
        let mocked_tests = TestsFromCrate {
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
            test_crate_type: TestCrateType::Tests,
        };

        let filtered = mocked_tests.filter_by_name("", true);
        assert_eq!(filtered.test_cases, vec![]);

        let filtered = mocked_tests.filter_by_name("thing", true);
        assert_eq!(filtered.test_cases, vec![]);

        let filtered = mocked_tests.filter_by_name("do_thing", true);
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

        let filtered = mocked_tests.filter_by_name("crate1::do_thing", true);
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

        let filtered = mocked_tests.filter_by_name("crate3::run_other_thing", true);
        assert_eq!(filtered.test_cases, vec![]);

        let filtered = mocked_tests.filter_by_name("outer::crate3::run_other_thing", true);
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
    fn filtering_tests_works_without_crate_in_test_name() {
        let mocked_tests = TestsFromCrate {
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
                    name: "thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
            ],
            test_crate_type: TestCrateType::Tests,
        };

        let result = mocked_tests.filter_by_name("thing", false);
        assert_eq!(
            result.test_cases,
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
                    name: "thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
            ]
        );
    }
}
