use super::TestTargetLocation;
use cairo_lang_sierra::program::ProgramArtifact;
use camino::Utf8PathBuf;

/// these structs are representation of scarb output for `scarb build --test`
/// produced by scarb
pub struct TestTargetRaw {
    pub sierra_file_path: Utf8PathBuf,
    pub sierra_program: ProgramArtifact,
    pub tests_location: TestTargetLocation,
}
