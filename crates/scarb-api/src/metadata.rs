pub use scarb_metadata::{Metadata, MetadataCommand, MetadataCommandError, PackageMetadata};

pub trait MetadataCommandExt {
    fn run(&mut self) -> Result<Metadata, MetadataCommandError>;
}

impl MetadataCommandExt for MetadataCommand {
    fn run(&mut self) -> Result<Metadata, MetadataCommandError> {
        self.inherit_stdout();

        let result = self.exec();

        if result.is_err() {
            println!("error: could not gather project metadata from Scarb due to previous error");
        }

        result
    }
}
