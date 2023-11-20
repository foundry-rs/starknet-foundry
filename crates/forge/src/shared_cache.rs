pub const CACHE_DIR: &str = ".snfoundry_cache";
pub const PREV_TESTS_FAILED: &str = ".prev_tests_failed";

use anyhow::{Ok, Result};
use camino::Utf8PathBuf;
use scarb_metadata::MetadataCommand;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};

use crate::test_case_summary::TestCaseSummary;
#[derive(Serialize, Deserialize)]
pub struct RerunFailed {
    pub failed: Vec<String>,
    pub passed: Vec<String>,
}

pub fn get_cached_failed_tests_names(cache_dir_path: &Utf8PathBuf) -> Result<Option<RerunFailed>> {
    let tests_failed_path = cache_dir_path.join(PREV_TESTS_FAILED);
    if !tests_failed_path.exists() {
        return Ok(None);
    }

    let file = File::open(tests_failed_path)?;
    let buf: BufReader<File> = BufReader::new(file);
    let tests: String = buf
        .lines()
        .map(|l| l.expect("Could not parse line"))
        .collect();

    let json: RerunFailed = serde_json::from_str(&tests)?;

    if json.failed.is_empty() {
        return Ok(None);
    }
    Ok(Some(json))
}

fn get_or_create_cache_dir(cache_dir_path: &Utf8PathBuf) -> Result<&Utf8PathBuf> {
    std::fs::create_dir_all(cache_dir_path)?;
    Ok(cache_dir_path)
}

pub fn cache_failed_tests_names(
    failed_tests_names: &[String],
    all_passed_tests_names: &[String],
    cache_dir_path: &Utf8PathBuf,
) -> Result<()> {
    let tests_failed_path = get_or_create_cache_dir(cache_dir_path)?.join(PREV_TESTS_FAILED);
    dbg!(&tests_failed_path);
    let file = File::create(tests_failed_path)?;
    let mut file = BufWriter::new(file);

    let returned_failed = RerunFailed {
        failed: failed_tests_names.to_vec(),
        passed: all_passed_tests_names.to_vec(),
    };

    let string = serde_json::to_string(&returned_failed)?;
    file.write(string.as_bytes())?;

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
