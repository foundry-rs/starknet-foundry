use super::TestTargetLocation;
use cairo_lang_sierra::program::ProgramArtifact;
use camino::Utf8PathBuf;
use universal_sierra_compiler_api::SierraProgramHash;

/// these structs are representation of scarb output for `scarb build --test`
/// produced by scarb
pub struct TestTargetRaw {
    pub sierra_program: ProgramArtifact,
    pub sierra_program_path: Utf8PathBuf,
    pub sierra_program_hash: SierraProgramHash,
    pub tests_location: TestTargetLocation,
}
