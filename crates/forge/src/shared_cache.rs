pub const CACHE_DIR: &str = ".snfoundry_cache";
pub const PREV_TESTS_FAILED: &str = ".prev_tests_failed";

use anyhow::{Ok, Result};
use camino::Utf8PathBuf;
use scarb_metadata::MetadataCommand;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};

use crate::test_case_summary::TestCaseSummary;

pub fn get_cached_failed_tests_names() -> Result<Option<Vec<String>>> {
    let tests_failed_path = get_cache_dir()?.join(PREV_TESTS_FAILED);
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

pub fn get_cache_dir() -> Result<Utf8PathBuf> {
    let scarb_metadata = MetadataCommand::new().inherit_stderr().exec();
    let scarb_metadata = scarb_metadata?;
    let workspace_root = scarb_metadata.workspace.root.clone();
    Ok(workspace_root.join(CACHE_DIR))
}

pub fn get_or_create_cache_dir() -> Result<Utf8PathBuf> {
    let cache_dir_path = get_cache_dir()?;
    std::fs::create_dir_all(&cache_dir_path)?;
    Ok(cache_dir_path)
}

pub fn cache_failed_tests_names(all_failed_tests: &[TestCaseSummary]) -> Result<()> {
    let tests_failed_path = get_or_create_cache_dir()?.join(PREV_TESTS_FAILED);
    let file = File::create(tests_failed_path)?;
    let mut file = BufWriter::new(file);
    for line in all_failed_tests {
        if let TestCaseSummary::Failed { name, .. } = line {
            writeln!(file, "{name}")?;
        }
    }

    Ok(())
}

pub fn clean_cache() -> Result<()> {
    let cache_dir = get_cache_dir()?;
    if cache_dir.exists() {
        fs::remove_dir_all(cache_dir)?;
    }
    Ok(())
}
