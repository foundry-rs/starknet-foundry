use super::TestTargetLocation;
use cairo_lang_sierra::program::ProgramArtifact;

/// these structs are representation of scarb output for `scarb build --test`

/// produced by scarb
pub struct TestTargetRaw {
    pub sierra_program: ProgramArtifact,
    pub tests_location: TestTargetLocation,
}
