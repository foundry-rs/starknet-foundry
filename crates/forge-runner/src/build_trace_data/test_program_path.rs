use anyhow::Context;
use anyhow::Result;
use cairo_lang_sierra::program::VersionedProgram;
use camino::Utf8PathBuf;
use fs4::FileExt;
use std::fs;
use std::fs::File;
use std::io::BufWriter;

pub const TESTS_PROGRAMS_DIR: &str = ".snfoundry_test_code";

#[derive(Clone)]
pub struct TestProgramPath {
    path: Utf8PathBuf,
}

impl TestProgramPath {
    pub fn save_test_program(
        versioned_program: &VersionedProgram,
        test_program_path: Utf8PathBuf,
    ) -> Result<Self> {
        fs::create_dir_all(test_program_path.parent().unwrap())
            .context("Failed to create directory for tests sierra programs")?;
        let output_file = File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&test_program_path)?;

        output_file
            .lock_exclusive()
            .with_context(|| format!("Couldn't lock the output file = {test_program_path}"))?;
        let file = BufWriter::new(&output_file);
        serde_json::to_writer(file, &versioned_program)
            .context("Failed to serialize VersionedProgram")?;
        output_file
            .unlock()
            .with_context(|| format!("Couldn't lock the output file = {test_program_path}"))?;

        Ok(Self {
            path: test_program_path,
        })
    }
}

impl From<TestProgramPath> for Utf8PathBuf {
    fn from(value: TestProgramPath) -> Self {
        value.path
    }
}
