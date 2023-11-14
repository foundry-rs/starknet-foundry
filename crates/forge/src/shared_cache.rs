pub const CACHE_DIR: &str = ".snfoundry_cache";
#[cfg(test)]
use mockall::{automock, predicate::str};

#[cfg_attr(test, automock)]
pub mod helpers {
    use anyhow::Result;
    use camino::Utf8PathBuf;
    use scarb_metadata::MetadataCommand;
    use std::fs::{self, File};
    use std::io::{BufRead, BufReader, Write};

    use crate::shared_cache::CACHE_DIR;
    use crate::test_case_summary::TestCaseSummary;

    pub fn read_tests_failed_file() -> Result<Vec<String>> {
        let tests_failed_path = cache_dir()?.join(".prev_tests_failed");
        let file = File::open(tests_failed_path)?;
        let buf = BufReader::new(file);
        Ok(buf
            .lines()
            .map(|l| l.expect("Could not parse line"))
            .collect())
    }

    pub fn cache_dir() -> Result<Utf8PathBuf> {
        let scarb_metadata = MetadataCommand::new().inherit_stderr().exec();
        let scarb_metadata = scarb_metadata?;
        let workspace_root = scarb_metadata.workspace.root.clone();
        let cache_dir_path = workspace_root.join(CACHE_DIR);
        std::fs::create_dir_all(&cache_dir_path)?;
        Ok(cache_dir_path)
    }

    pub fn write_failed_tests(all_failed_tests: &[TestCaseSummary]) -> Result<()> {
        let tests_failed_path = cache_dir()?.join(".prev_tests_failed");
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(tests_failed_path)?;

        for line in all_failed_tests {
            if let TestCaseSummary::Failed { name, .. } = line {
                file.write_all((name.clone() + "\n").as_bytes())
                    .expect("Can not write to file");
            }
        }

        Ok(())
    }

    pub fn clean_cache() -> Result<()> {
        let cache_dir = cache_dir()?;
        if cache_dir.exists() {
            fs::remove_dir_all(cache_dir)?;
        }
        Ok(())
    }
}

pub use helpers::*;
