use anyhow::Context;
use anyhow::Result;
use cairo_lang_sierra::program::VersionedProgram;
use camino::{Utf8Path, Utf8PathBuf};
use fs4::FileExt;
use std::fs;
use std::fs::File;
use std::io::BufWriter;

pub const TESTS_PROGRAMS_DIR: &str = ".snfoundry_test_code";

/// A path to a file with deserialized [`VersionedProgram`] that comes
/// from compiled test crate. Needed to provide path to source sierra in
/// [`trace_data::CairoExecutionInfo`].
#[derive(Clone)]
pub struct TestSierraProgramPath(Utf8PathBuf);

impl TestSierraProgramPath {
    pub fn save_sierra_test_program_from_test_crate(
        versioned_program_from_crate: &VersionedProgram,
        crate_location: &str,
        tests_programs_dir: &Utf8Path,
        package_name: &str,
    ) -> Result<Self> {
        // unique filename since pair (package_name, crate_location) is always unique
        let test_sierra_program_path =
            tests_programs_dir.join(format!("{package_name}_{crate_location}.sierra.json",));

        fs::create_dir_all(test_sierra_program_path.parent().unwrap())
            .context("Failed to create directory for tests sierra programs")?;
        let output_file = File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&test_sierra_program_path)?;

        output_file.lock_exclusive().with_context(|| {
            format!("Couldn't lock the output file = {test_sierra_program_path}")
        })?;
        let file = BufWriter::new(&output_file);
        serde_json::to_writer(file, &versioned_program_from_crate)
            .context("Failed to serialize VersionedProgram")?;
        output_file.unlock().with_context(|| {
            format!("Couldn't lock the output file = {test_sierra_program_path}")
        })?;

        Ok(Self(test_sierra_program_path))
    }
}

impl From<TestSierraProgramPath> for Utf8PathBuf {
    fn from(value: TestSierraProgramPath) -> Self {
        value.0
    }
}
