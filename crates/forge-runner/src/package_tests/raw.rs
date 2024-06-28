use super::TestTargetLocation;
use cairo_lang_sierra::{ids::FunctionId, program::Program};
use serde::Deserialize;
use smol_str::SmolStr;
use std::collections::HashMap;

/// these structs are representation of scarb output for `scarb build --test`

/// produced by scarb
pub struct TestTargetRaw {
    pub sierra_program: ProgramArtifact,
    pub tests_location: TestTargetLocation,
}

// this should be deleted once we can bump cairo-lang-* deps

//TODO this should be cairo_lang_sierra::program::ProgramArtifact but can't bump cairo_lang_sierra dependency
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct ProgramArtifact {
    #[serde(flatten)]
    pub program: Program,
    pub debug_info: DebugInfo,
}

//TODO this should be cairo_lang_sierra::debug_info::DebugInfo but can't bump cairo_lang_sierra dependency
#[derive(Clone, Debug, Eq, PartialEq, Default, Deserialize)]
pub struct DebugInfo {
    // here are more fields but we don't need these
    #[serde(default)]
    pub executables: HashMap<SmolStr, Vec<FunctionId>>,
}
