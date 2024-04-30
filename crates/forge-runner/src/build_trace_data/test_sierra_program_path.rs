use crate::compiled_runnable::CrateLocation;
use anyhow::Context;
use anyhow::Result;
use cairo_lang_sierra::program::VersionedProgram;
use camino::{Utf8Path, Utf8PathBuf};
use fs4::FileExt;
use std::fs;
use std::fs::File;
use std::io::BufWriter;

pub const VERSIONED_PROGRAMS_DIR: &str = ".snfoundry_versioned_programs";

/// A path to a file with deserialized [`VersionedProgram`] that comes
/// from compiled test crate. Needed to provide path to source sierra in
/// [`trace_data::CairoExecutionInfo`].
#[derive(Clone)]
pub struct VersionedProgramPath(Utf8PathBuf);

impl VersionedProgramPath {
    pub fn save_versioned_program(
        versioned_program: &VersionedProgram,
        crate_location: CrateLocation,
        tests_programs_dir: &Utf8Path,
        package_name: &str,
    ) -> Result<Self> {
        // unique filename since pair (package_name, crate_location) is always unique
        let test_sierra_program_path =
            tests_programs_dir.join(format!("{package_name}_{crate_location:?}.sierra.json",));

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
        serde_json::to_writer(file, &versioned_program)
            .context("Failed to serialize VersionedProgram")?;
        output_file.unlock().with_context(|| {
            format!("Couldn't lock the output file = {test_sierra_program_path}")
        })?;

        Ok(Self(test_sierra_program_path))
    }
}

impl From<VersionedProgramPath> for Utf8PathBuf {
    fn from(value: VersionedProgramPath) -> Self {
        value.0
    }
}
