pub use scarb_metadata::{Metadata, MetadataCommand, MetadataCommandError, PackageMetadata};

pub trait MetadataCommandExt {
    fn run(&mut self) -> Result<Metadata, MetadataCommandError>;
}

impl MetadataCommandExt for MetadataCommand {
    fn run(&mut self) -> Result<Metadata, MetadataCommandError> {
        // logic will go here
        // keep it now this way to don't rename method calls later
        self.exec()
    }
}
