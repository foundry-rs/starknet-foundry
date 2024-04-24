use crate::compiled_runnable::CompiledTestCrateRunnable;
use anyhow::Context;
use anyhow::Result;
use cairo_lang_sierra::program::VersionedProgram;
use camino::Utf8PathBuf;
use fs4::FileExt;
use std::fs::File;
use std::io::BufWriter;

pub struct SierraTestCodePath {
    pub path: Utf8PathBuf,
}

impl SierraTestCodePath {
    pub fn new(test_crate: &CompiledTestCrateRunnable, output_path: Utf8PathBuf) -> Result<Self> {
        let versioned_program: VersionedProgram = test_crate.sierra_program.clone().into();
        let output_file = File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&output_path)?;

        output_file
            .lock_exclusive()
            .with_context(|| format!("Couldn't lock the output file = {output_path}"))?;
        let file = BufWriter::new(&output_file);
        serde_json::to_writer(file, &versioned_program)
            .context("Failed to serialize VersionedProgram")?;
        output_file
            .unlock()
            .with_context(|| format!("Couldn't lock the output file = {output_path}"))?;

        Ok(Self { path: output_path })
    }
}
