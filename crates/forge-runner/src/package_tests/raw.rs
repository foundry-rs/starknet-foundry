use super::TestTargetLocation;
use cairo_lang_sierra::program::ProgramArtifact;
use camino::Utf8PathBuf;

/// these structs are representation of scarb output for `scarb build --test`
/// produced by scarb
pub struct TestTargetRaw {
    pub sierra_program: ProgramArtifact,
    pub sierra_program_path: Utf8PathBuf,
    pub tests_location: TestTargetLocation,
    pub scarb_profile: String,
}
