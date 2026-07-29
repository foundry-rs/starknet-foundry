use crate::e2e::common::runner::{setup_package, test_runner};
use forge_runner::DEFAULT_CACHE_DIR;
use std::fs;
use std::path::{Path, PathBuf};
use universal_sierra_compiler_api::CASM_CACHE_DIR;
#[cfg(unix)]
use universal_sierra_compiler_api::version_command;

#[test]
fn creates_raw_and_contract_casm_cache_entries() {
    let temp = setup_package("targets/unit_and_integration");

    test_runner(&temp).assert().success();

    let casm_cache_dir = casm_cache_dir(temp.path());

    assert_cache_entries_created(&json_files(&casm_cache_dir.join("raw")), "raw");
    assert_cache_entries_created(&json_files(&casm_cache_dir.join("contract")), "contract");
}

#[cfg(unix)]
#[test]
fn reuses_cached_casm_without_invoking_compiler() {
    let temp = setup_package("targets/unit_and_integration");

    test_runner(&temp).assert().success();

    let casm_cache_dir = casm_cache_dir(temp.path());
    let raw_cache_dir = casm_cache_dir.join("raw");
    let contract_cache_dir = casm_cache_dir.join("contract");
    let raw_cache_entries = json_files(&raw_cache_dir);
    let contract_cache_entries = json_files(&contract_cache_dir);
    assert_cache_entries_created(&raw_cache_entries, "raw");
    assert_cache_entries_created(&contract_cache_entries, "contract");

    let fake_compiler = fake_compiler_that_fails_on_compile(temp.path());
    test_runner(&temp)
        .env("UNIVERSAL_SIERRA_COMPILER", fake_compiler)
        .assert()
        .success();

    assert_eq!(
        json_files(&raw_cache_dir),
        raw_cache_entries,
        "raw cache entries should be reused without creating new files",
    );
    assert_eq!(
        json_files(&contract_cache_dir),
        contract_cache_entries,
        "contract cache entries should be reused without creating new files",
    );
}

#[test]
fn creates_new_cache_entries_after_sierra_changes() {
    let temp = setup_package("targets/unit_and_integration");

    test_runner(&temp).assert().success();

    let casm_cache_dir = casm_cache_dir(temp.path());
    let raw_cache_dir = casm_cache_dir.join("raw");
    let contract_cache_dir = casm_cache_dir.join("contract");
    let raw_cache_entries = json_files(&raw_cache_dir);
    let contract_cache_entries = json_files(&contract_cache_dir);
    assert_cache_entries_created(&raw_cache_entries, "raw");
    assert_cache_entries_created(&contract_cache_entries, "contract");

    replace_file_contents(
        &temp.path().join("tests/tests.cairo"),
        "'balance != 100'",
        "'balance still 100'",
    );
    replace_file_contents(
        &temp.path().join("src/lib.cairo"),
        "arr.append('DAYTAH');",
        "arr.append('CACHE');",
    );

    test_runner(&temp).assert().success();

    assert_new_cache_entries_created(&raw_cache_entries, &json_files(&raw_cache_dir), "raw");
    assert_new_cache_entries_created(
        &contract_cache_entries,
        &json_files(&contract_cache_dir),
        "contract",
    );
}

fn casm_cache_dir(temp_dir: &Path) -> PathBuf {
    temp_dir.join(DEFAULT_CACHE_DIR).join(CASM_CACHE_DIR)
}

fn assert_cache_entries_created(cache_entries: &[PathBuf], cache_kind: &str) {
    assert!(
        !cache_entries.is_empty(),
        "{cache_kind} cache should contain at least one json file",
    );
}

fn assert_new_cache_entries_created(
    old_cache_entries: &[PathBuf],
    new_cache_entries: &[PathBuf],
    cache_kind: &str,
) {
    assert!(
        new_cache_entries.len() > old_cache_entries.len(),
        "{cache_kind} cache should contain new entries after sierra changes: old={old_cache_entries:?}, new={new_cache_entries:?}",
    );
    assert!(
        old_cache_entries
            .iter()
            .all(|cache_entry| new_cache_entries.contains(cache_entry)),
        "{cache_kind} cache should preserve existing entries after sierra changes",
    );
}

fn replace_file_contents(path: &Path, from: &str, to: &str) {
    let contents = fs::read_to_string(path).unwrap();
    assert_eq!(
        contents.matches(from).count(),
        1,
        "{from} should occur exactly once in {}",
        path.display()
    );
    let updated_contents = contents.replace(from, to);
    fs::write(path, updated_contents).unwrap();
}

fn json_files(path: &Path) -> Vec<PathBuf> {
    let mut files = vec![];
    collect_json_files(path, &mut files);
    files.sort();
    files
}

fn collect_json_files(path: &Path, files: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));

    for entry in entries {
        let entry = entry
            .unwrap_or_else(|error| panic!("failed to read entry in {}: {error}", path.display()));
        let path = entry.path();
        if path.is_dir() {
            collect_json_files(&path, files);
        } else if path
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            files.push(path);
        }
    }
}

#[cfg(unix)]
fn fake_compiler_that_fails_on_compile(temp_dir: &Path) -> PathBuf {
    use std::os::unix::fs::PermissionsExt as _;

    let compiler = temp_dir.join("fake-universal-sierra-compiler");
    fs::write(compiler.with_extension("version"), compiler_version()).unwrap();
    fs::write(
        &compiler,
        r#"#!/usr/bin/env bash
if [[ "$1" == "--version" ]]; then
  cat "$0.version"
  exit 0
fi
echo "unexpected universal-sierra-compiler invocation: $*" >&2
exit 99
"#,
    )
    .unwrap();

    let mut permissions = fs::metadata(&compiler).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&compiler, permissions).unwrap();

    compiler
}

#[cfg(unix)]
fn compiler_version() -> Vec<u8> {
    let output = version_command().unwrap().output().unwrap();
    assert!(output.status.success());
    output.stdout
}
