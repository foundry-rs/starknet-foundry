use cairo_lang_sierra::program::Program;
use serde::Deserialize;
use snforge_test_collector_interface::{CrateLocation, TestCaseRaw};

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CompiledTestCrateRaw {
    pub sierra_program: Program,
    pub test_cases: Vec<TestCaseRaw>,
    pub tests_location: CrateLocation,
}
