use anyhow::{Context, Result};
pub use scarb_metadata::{Metadata, MetadataCommand, MetadataCommandError, PackageMetadata};

pub trait MetadataCommandExt {
    fn run(&mut self) -> Result<Metadata>;
}

impl MetadataCommandExt for MetadataCommand {
    fn run(&mut self) -> Result<Metadata> {
        self.inherit_stdout()
            .exec()
            .context("error: could not gather project metadata from Scarb due to previous error")
    }
}
