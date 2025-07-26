use crate::running::hints_to_params;
use anyhow::Result;
use cairo_lang_sierra::extensions::NamedType;
use cairo_lang_sierra::extensions::bitwise::BitwiseType;
use cairo_lang_sierra::extensions::circuit::{AddModType, MulModType};
use cairo_lang_sierra::extensions::ec::EcOpType;
use cairo_lang_sierra::extensions::pedersen::PedersenType;
use cairo_lang_sierra::extensions::poseidon::PoseidonType;
use cairo_lang_sierra::extensions::range_check::{RangeCheck96Type, RangeCheckType};
use cairo_lang_sierra::extensions::segment_arena::SegmentArenaType;
use cairo_lang_sierra::ids::GenericTypeId;
use cairo_lang_sierra::program::ProgramArtifact;
use cairo_vm::serde::deserialize_program::ReferenceManager;
use cairo_vm::types::builtin_name::BuiltinName;
use cairo_vm::types::program::Program;
use cairo_vm::types::relocatable::MaybeRelocatable;
use camino::Utf8PathBuf;
use serde::Serialize;
use starknet_types_core::felt::Felt;
use std::collections::HashMap;
use std::sync::Arc;
use universal_sierra_compiler_api::AssembledProgramWithDebugInfo;

pub mod raw;
pub mod with_config;
pub mod with_config_resolved;

// If modifying this, make sure that the order of builtins matches that from
// `#[implicit_precedence(...)` in generated test code.
const BUILTIN_ORDER: [(BuiltinName, GenericTypeId); 9] = [
    (BuiltinName::pedersen, PedersenType::ID),
    (BuiltinName::range_check, RangeCheckType::ID),
    (BuiltinName::bitwise, BitwiseType::ID),
    (BuiltinName::ec_op, EcOpType::ID),
    (BuiltinName::poseidon, PoseidonType::ID),
    (BuiltinName::segment_arena, SegmentArenaType::ID),
    (BuiltinName::range_check96, RangeCheck96Type::ID),
    (BuiltinName::add_mod, AddModType::ID),
    (BuiltinName::mul_mod, MulModType::ID),
];

#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq, Serialize)]
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

impl TestDetails {
    #[must_use]
    pub fn builtins(&self) -> Vec<BuiltinName> {
        let mut builtins = vec![];
        for (builtin_name, builtin_ty) in BUILTIN_ORDER {
            if self.parameter_types.iter().any(|(ty, _)| ty == &builtin_ty) {
                builtins.push(builtin_name);
            }
        }
        builtins
    }

    pub fn try_into_program(
        &self,
        casm_program: &AssembledProgramWithDebugInfo,
    ) -> Result<Program> {
        let builtins = self.builtins();

        let assembled_program = &casm_program.assembled_cairo_program;
        let hints_dict = hints_to_params(assembled_program);
        let data: Vec<MaybeRelocatable> = assembled_program
            .bytecode
            .iter()
            .map(Felt::from)
            .map(MaybeRelocatable::from)
            .collect();

        Program::new(
            builtins.clone(),
            data,
            Some(0),
            hints_dict,
            ReferenceManager {
                references: Vec::new(),
            },
            HashMap::new(),
            vec![],
            None,
        )
        .map_err(std::convert::Into::into)
    }
}

#[derive(Debug, Clone)]
pub struct TestTarget<C> {
    pub tests_location: TestTargetLocation,
    pub sierra_program: ProgramArtifact,
    pub sierra_program_path: Arc<Utf8PathBuf>,
    pub casm_program: Arc<AssembledProgramWithDebugInfo>,
    pub scarb_profile: String,
    pub test_cases: Vec<TestCase<C>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestCase<C> {
    pub test_details: TestDetails,
    pub name: String,
    pub config: C,
}
