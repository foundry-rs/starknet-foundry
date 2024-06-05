use anyhow::Result;
use camino::Utf8PathBuf;
use forge_runner::test_case_summary::AnyTestCaseSummary;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, ErrorKind, Write};

#[derive(Debug, PartialEq, Default)]
pub struct FailedTestsCache {
    cache_dir_path: Utf8PathBuf,
}

const PREV_TESTS_FAILED: &str = ".prev_tests_failed";

impl FailedTestsCache {
    pub fn new(cache_dir_path: Utf8PathBuf) -> Self {
        Self { cache_dir_path }
    }

    pub fn load(&self) -> Result<Vec<String>> {
        let tests_failed_path = self.cache_dir_path.join(PREV_TESTS_FAILED);

        let file = match File::open(tests_failed_path) {
            Ok(file) => file,
            Err(err) if err.kind() == ErrorKind::NotFound => return Ok(vec![]),
            Err(err) => Err(err)?,
        };
        let buf: BufReader<File> = BufReader::new(file);

        let tests = buf.lines().map(|l| l).collect::<Result<Vec<_>, _>>()?;

        Ok(tests)
    }

    pub fn save_failed_tests(&self, all_failed_tests: &[AnyTestCaseSummary]) -> Result<()> {
        std::fs::create_dir_all(&self.cache_dir_path)?;

        let tests_failed_path = self.cache_dir_path.join(PREV_TESTS_FAILED);

        let file = File::create(tests_failed_path)?;

        let mut file = BufWriter::new(file);

        for line in all_failed_tests {
            let name = line.name().unwrap();

            writeln!(file, "{name}")?;
        }
        Ok(())
    }

    pub fn clean(&self) -> Result<()> {
        if self.cache_dir_path.exists() {
            fs::remove_dir_all(&self.cache_dir_path)?;
        }
        Ok(())
    }
}
