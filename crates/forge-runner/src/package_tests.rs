use cairo_lang_sierra::ids::GenericTypeId;
use cairo_lang_sierra::program::ProgramArtifact;
use camino::Utf8PathBuf;
use std::sync::Arc;
use universal_sierra_compiler_api::AssembledProgramWithDebugInfo;

pub mod raw;
pub mod with_config;
pub mod with_config_resolved;

#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq)]
pub enum TestTargetLocation {
    /// Main crate in a package
    Lib,
    /// Crate in the `tests/` directory
    Tests,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct TestDetails {
    pub sierra_entry_point_statement_idx: usize,
    pub parameter_types: Vec<(GenericTypeId, i16)>,
    pub return_types: Vec<(GenericTypeId, i16)>,
}

#[derive(Debug, Clone)]
pub struct TestTarget<C> {
    pub tests_location: TestTargetLocation,
    pub sierra_program: ProgramArtifact,
    pub sierra_program_path: Arc<Utf8PathBuf>,
    pub casm_program: Arc<AssembledProgramWithDebugInfo>,
    pub test_cases: Vec<TestCase<C>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestCase<C> {
    pub test_details: TestDetails,
    pub name: String,
    pub config: C,
}
