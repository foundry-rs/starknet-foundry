use crate::e2e::common::runner::{setup_package, test_runner};
use forge_runner::DEFAULT_CACHE_DIR;
use std::fs;
use std::path::Path;
#[cfg(unix)]
use std::path::PathBuf;
use universal_sierra_compiler_api::CASM_CACHE_DIR;
#[cfg(unix)]
use universal_sierra_compiler_api::version_command;

#[test]
fn creates_raw_and_contract_casm_cache_entries() {
    let temp = setup_package("targets/unit_and_integration");

    test_runner(&temp).assert().success();

    let casm_cache_dir = temp.path().join(DEFAULT_CACHE_DIR).join(CASM_CACHE_DIR);
    assert!(contains_json_file(&casm_cache_dir.join("raw")));
    assert!(contains_json_file(&casm_cache_dir.join("contract")));
}

#[cfg(unix)]
#[test]
fn reuses_cached_casm_without_invoking_compiler() {
    let temp = setup_package("targets/unit_and_integration");

    test_runner(&temp).assert().success();

    let fake_compiler = fake_compiler_that_fails_on_compile(temp.path());
    test_runner(&temp)
        .env("UNIVERSAL_SIERRA_COMPILER", fake_compiler)
        .assert()
        .success();
}

fn contains_json_file(path: &Path) -> bool {
    let Ok(entries) = fs::read_dir(path) else {
        return false;
    };

    entries.filter_map(Result::ok).any(|entry| {
        let path = entry.path();
        if path.is_dir() {
            contains_json_file(&path)
        } else {
            path.extension()
                .is_some_and(|extension| extension == "json")
        }
    })
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
