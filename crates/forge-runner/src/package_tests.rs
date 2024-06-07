use cairo_lang_sierra::ids::GenericTypeId;
use cairo_lang_sierra::program::ProgramArtifact;
use serde::Deserialize;

pub mod raw;
pub mod with_config;
pub mod with_config_resolved;

#[derive(Debug, PartialEq, Clone, Copy, Deserialize, Hash, Eq)]
pub enum TestTargetLocation {
    /// Main crate in a package
    Lib,
    /// Crate in the `tests/` directory
    Tests,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Default)]
pub struct TestDetails {
    #[serde(rename = "entry_point_offset")]
    pub sierra_entry_point_statement_idx: usize,
    pub parameter_types: Vec<(GenericTypeId, i16)>,
    pub return_types: Vec<(GenericTypeId, i16)>,
}

#[derive(Debug, Clone)]
pub struct TestTarget {
    pub tests_location: TestTargetLocation,
    pub sierra_program: ProgramArtifact,
    pub test_cases: Vec<TestCase>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestCase {
    pub test_details: TestDetails,
    pub name: String,
}
