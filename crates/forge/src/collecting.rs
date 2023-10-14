use crate::{CrateLocation, RunnerConfig, RunnerParams, TestsToRun, BUILTINS};
use anyhow::{anyhow, Context, Result};
use assert_fs::fixture::{FileTouch, PathChild, PathCopy};
use assert_fs::TempDir;
use cairo_lang_sierra::program::Program;
use camino::{Utf8Path, Utf8PathBuf};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use test_collector::{collect_tests, LinkedLibrary, TestCase};
use walkdir::WalkDir;
#[derive(Debug, Clone)]
pub(crate) struct CompiledTestCrate {
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

    fn extract_ignored(self) -> Self {
        let result = self
            .test_cases
            .into_iter()
            .filter(|case| case.ignored)
            .collect();
        Self {
            test_cases: result,
            ..self
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct TestCompilationTarget {
    pub(crate) crate_root: Utf8PathBuf,
    pub(crate) crate_name: String,
    pub(crate) crate_location: CrateLocation,
}

impl TestCompilationTarget {
    pub(crate) fn compile_tests(
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

    pub(crate) fn ensure_lib_file_exists(self, temp_dir: &TempDir) -> Result<Self> {
        match self.crate_location {
            CrateLocation::Lib => Ok(self),
            CrateLocation::Tests => {
                let lib_path = self.crate_root.join("lib.cairo");
                if lib_path.exists() {
                    Ok(self)
                } else {
                    pack_tests_into_single_crate(&self.crate_name, temp_dir, &self.crate_root)
                }
            }
        }
    }
}

pub(crate) fn collect_test_compilation_targets(
    package_path: &Utf8Path,
    package_name: &str,
    package_source_dir_path: &Utf8Path,
) -> Vec<TestCompilationTarget> {
    let mut compilation_targets = vec![TestCompilationTarget {
        crate_root: package_source_dir_path.to_path_buf(),
        crate_name: package_name.to_string(),
        crate_location: CrateLocation::Lib,
    }];

    let tests_dir_path = package_path.join("tests");
    if tests_dir_path.exists() {
        compilation_targets.push(TestCompilationTarget {
            crate_root: tests_dir_path,
            crate_name: "tests".to_string(),
            crate_location: CrateLocation::Tests,
        });
    }

    compilation_targets
}

pub(crate) fn compile_tests(
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

pub(crate) fn filter_tests_from_crates(
    compiled_test_crates: Vec<CompiledTestCrate>,
    runner_config: &RunnerConfig,
) -> Vec<CompiledTestCrate> {
    let filtered_by_name = if let Some(test_name_filter) = &runner_config.test_name_filter {
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
    };

    if let TestsToRun::Ignored = runner_config.tests_to_run {
        filtered_by_name
            .into_iter()
            .map(CompiledTestCrate::extract_ignored)
            .collect()
    } else {
        filtered_by_name
    }
}

fn pack_tests_into_single_crate(
    name: &str,
    tmp_dir: &TempDir,
    tests_folder_path: &Utf8Path,
) -> Result<TestCompilationTarget> {
    let tmp_dir = tmp_dir.child(name);
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
    use crate::CrateLocation;
    use assert_fs::fixture::PathCopy;
    use assert_fs::TempDir;
    use test_collector::ExpectedTestResult;

    #[test]
    fn collecting_test_compilation_targets() {
        let temp = TempDir::new().unwrap();
        temp.copy_from("tests/data/simple_package", &["**/*.cairo", "**/*.toml"])
            .unwrap();
        let package_path = Utf8PathBuf::from_path_buf(temp.to_path_buf()).unwrap();

        let compilation_targets =
            collect_test_compilation_targets(&package_path, "simple_package", &package_path);

        assert_eq!(
            compilation_targets,
            vec![
                TestCompilationTarget {
                    crate_root: package_path.clone(),
                    crate_name: "simple_package".to_string(),
                    crate_location: CrateLocation::Lib,
                },
                TestCompilationTarget {
                    crate_root: package_path.join("tests"),
                    crate_name: "tests".to_string(),
                    crate_location: CrateLocation::Tests,
                }
            ]
        );
    }

    #[test]
    fn ensure_lib_in_compilation_targets() {
        let temp = TempDir::new().unwrap();
        temp.copy_from("tests/data/simple_package", &["**/*.cairo", "**/*.toml"])
            .unwrap();
        let package_path = Utf8PathBuf::from_path_buf(temp.to_path_buf()).unwrap();

        let compilation_targets = vec![
            TestCompilationTarget {
                crate_root: package_path.clone(),
                crate_name: "simple_package".to_string(),
                crate_location: CrateLocation::Lib,
            },
            TestCompilationTarget {
                crate_root: package_path.join("tests"),
                crate_name: "tests".to_string(),
                crate_location: CrateLocation::Tests,
            },
        ];

        let temp_for_tests = TempDir::new().unwrap();
        let temp_for_tests_path = Utf8PathBuf::from_path_buf(temp_for_tests.to_path_buf()).unwrap();
        let optimized_compilation_targets: Vec<TestCompilationTarget> = compilation_targets
            .into_iter()
            .map(|ct| ct.ensure_lib_file_exists(&temp_for_tests))
            .collect::<Result<_>>()
            .unwrap();

        assert_eq!(
            optimized_compilation_targets,
            vec![
                TestCompilationTarget {
                    crate_root: package_path.clone(),
                    crate_name: "simple_package".to_string(),
                    crate_location: CrateLocation::Lib,
                },
                TestCompilationTarget {
                    crate_root: temp_for_tests_path.clone().join("tests"),
                    crate_name: "tests".to_string(),
                    crate_location: CrateLocation::Tests,
                },
            ]
        );

        let virtual_lib_path = temp_for_tests_path.join("tests/lib.cairo");
        let virtual_lib_u8_content = std::fs::read(&virtual_lib_path).unwrap();
        let virtual_lib_content = std::str::from_utf8(&virtual_lib_u8_content).unwrap();

        assert!(virtual_lib_path.try_exists().unwrap());
        assert!(virtual_lib_content.contains("mod contract;"));
        assert!(virtual_lib_content.contains("mod ext_function_test;"));
        assert!(virtual_lib_content.contains("mod test_simple;"));
        assert!(virtual_lib_content.contains("mod without_prefix;"));
    }

    #[test]
    fn ensure_lib_in_compilation_targets_with_multiple_tests_dirs() {
        let temp = TempDir::new().unwrap();
        temp.copy_from("tests/data/simple_package", &["**/*.cairo", "**/*.toml"])
            .unwrap();
        let package_path = Utf8PathBuf::from_path_buf(temp.to_path_buf()).unwrap();

        let compilation_targets = vec![
            TestCompilationTarget {
                crate_root: package_path.join("tests"),
                crate_name: "tests".to_string(),
                crate_location: CrateLocation::Tests,
            },
            TestCompilationTarget {
                crate_root: package_path.join("tests"),
                crate_name: "other_tests".to_string(),
                crate_location: CrateLocation::Tests,
            },
        ];

        let temp_for_tests = TempDir::new().unwrap();
        let temp_for_tests_path = Utf8PathBuf::from_path_buf(temp_for_tests.to_path_buf()).unwrap();
        let optimized_compilation_targets: Vec<TestCompilationTarget> = compilation_targets
            .into_iter()
            .map(|ct| ct.ensure_lib_file_exists(&temp_for_tests))
            .collect::<Result<_>>()
            .unwrap();

        assert_eq!(
            optimized_compilation_targets,
            vec![
                TestCompilationTarget {
                    crate_root: temp_for_tests_path.clone().join("tests"),
                    crate_name: "tests".to_string(),
                    crate_location: CrateLocation::Tests,
                },
                TestCompilationTarget {
                    crate_root: temp_for_tests_path.clone().join("other_tests"),
                    crate_name: "tests".to_string(),
                    crate_location: CrateLocation::Tests,
                },
            ]
        );

        for name in ["tests", "other_tests"] {
            let virtual_lib_path = temp_for_tests_path.join(name).join("lib.cairo");
            let virtual_lib_u8_content = std::fs::read(&virtual_lib_path).unwrap();
            let virtual_lib_content = std::str::from_utf8(&virtual_lib_u8_content).unwrap();

            assert!(virtual_lib_path.try_exists().unwrap());
            assert!(virtual_lib_content.contains("mod contract;"));
            assert!(virtual_lib_content.contains("mod ext_function_test;"));
            assert!(virtual_lib_content.contains("mod test_simple;"));
            assert!(virtual_lib_content.contains("mod without_prefix;"));
        }
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
                    ignored: false,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "crate2::run_other_thing".to_string(),
                    available_gas: None,
                    ignored: true,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "outer::crate2::execute_next_thing".to_string(),
                    available_gas: None,
                    ignored: true,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "thing".to_string(),
                    available_gas: None,
                    ignored: false,
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
                ignored: false,
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
                ignored: true,
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
                    ignored: false,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "crate2::run_other_thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                    ignored: true,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "outer::crate2::execute_next_thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                    ignored: true,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "thing".to_string(),
                    available_gas: None,
                    ignored: false,
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
                    ignored: false,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "crate2::run_other_thing".to_string(),
                    available_gas: None,
                    ignored: true,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "outer::crate2::execute_next_thing".to_string(),
                    available_gas: None,
                    ignored: true,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "thing".to_string(),
                    available_gas: None,
                    ignored: false,
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
                    ignored: false,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "crate2::run_other_thing".to_string(),
                    available_gas: None,
                    ignored: true,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "outer::crate2::run_other_thing".to_string(),
                    available_gas: None,
                    ignored: true,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "thing".to_string(),
                    available_gas: None,
                    ignored: false,
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
                    ignored: true,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "outer::crate2::run_other_thing".to_string(),
                    available_gas: None,
                    ignored: true,
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
                    ignored: false,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "crate2::run_other_thing".to_string(),
                    available_gas: None,
                    ignored: true,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "outer::crate3::run_other_thing".to_string(),
                    available_gas: None,
                    ignored: true,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                    fuzzer_config: None,
                },
                TestCase {
                    name: "do_thing".to_string(),
                    available_gas: None,
                    ignored: false,
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
                ignored: false,
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
                ignored: false,
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
                ignored: true,
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
