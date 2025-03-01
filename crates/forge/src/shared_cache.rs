use anyhow::Result;
use camino::Utf8PathBuf;
use forge_runner::test_case_summary::AnyTestCaseSummary;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, ErrorKind, Write};

#[derive(Debug, PartialEq, Default, Clone)]
pub struct FailedTestsCache {
    cache_file: Utf8PathBuf,
}

const FILE_WITH_PREV_TESTS_FAILED: &str = ".prev_tests_failed";

impl FailedTestsCache {
    #[must_use]
    pub fn new(cache_dir: &Utf8PathBuf) -> Self {
        Self {
            cache_file: cache_dir.join(FILE_WITH_PREV_TESTS_FAILED),
        }
    }

    pub fn load(&self) -> Result<Vec<String>> {
        let file = match File::open(&self.cache_file) {
            Ok(file) => file,
            Err(err) if err.kind() == ErrorKind::NotFound => return Ok(vec![]),
            Err(err) => Err(err)?,
        };
        let buf: BufReader<File> = BufReader::new(file);

        let tests = buf.lines().collect::<Result<Vec<_>, _>>()?;

        Ok(tests)
    }

    pub fn save_failed_tests(&self, all_failed_tests: &[AnyTestCaseSummary]) -> Result<()> {
        std::fs::create_dir_all(self.cache_file.parent().unwrap())?;

        let file = File::create(&self.cache_file)?;

        let mut file = BufWriter::new(file);

        for line in all_failed_tests {
            let name = line.name().unwrap();

            writeln!(file, "{name}")?;
        }
        Ok(())
    }
}
