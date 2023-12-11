pub const CACHE_DIR: &str = ".snfoundry_cache";
pub const PREV_TESTS_FAILED: &str = ".prev_tests_failed";

use anyhow::{Ok, Result};
use camino::Utf8PathBuf;
use forge_runner::test_case_summary::AnyTestCaseSummary;
use scarb_metadata::MetadataCommand;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};

pub fn cached_failed_tests_names(cache_dir_path: &Utf8PathBuf) -> Result<Option<Vec<String>>> {
    let tests_failed_path = cache_dir_path.join(PREV_TESTS_FAILED);
    if !tests_failed_path.exists() {
        return Ok(None);
    }

    let file = File::open(tests_failed_path)?;
    let buf: BufReader<File> = BufReader::new(file);
    let tests: Vec<String> = buf
        .lines()
        .map(|l| l.expect("Could not parse line"))
        .collect();
    if tests.is_empty() {
        return Ok(None);
    }
    Ok(Some(tests))
}

fn get_or_create_cache_dir(cache_dir_path: &Utf8PathBuf) -> Result<&Utf8PathBuf> {
    std::fs::create_dir_all(cache_dir_path)?;
    Ok(cache_dir_path)
}

pub fn set_cached_failed_tests_names(
    all_failed_tests: &[AnyTestCaseSummary],
    cache_dir_path: &Utf8PathBuf,
) -> Result<()> {
    let tests_failed_path = get_or_create_cache_dir(cache_dir_path)?.join(PREV_TESTS_FAILED);

    let file = File::create(tests_failed_path)?;
    let mut file = BufWriter::new(file);
    for line in all_failed_tests {
        let name = line.name().unwrap();
        writeln!(file, "{name}")?;
    }
    Ok(())
}

pub fn clean_cache() -> Result<()> {
    let scarb_metadata = MetadataCommand::new().inherit_stderr().exec()?;
    let workspace_root = scarb_metadata.workspace.root.clone();
    let cache_dir = workspace_root.join(CACHE_DIR);
    if cache_dir.exists() {
        fs::remove_dir_all(cache_dir)?;
    }
    Ok(())
}
