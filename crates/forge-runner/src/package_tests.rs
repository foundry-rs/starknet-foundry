use crate::forge_config::ForgeTrackedResource;
use crate::package_tests::raw::TestTargetRaw;
use crate::package_tests::with_config::{TestCaseWithConfig, TestTargetWithConfig};
use crate::running::config_run::run_config_pass;
use crate::running::hints_to_params;
use anyhow::{Result, anyhow};
use cairo_lang_sierra::extensions::NamedType;
use cairo_lang_sierra::extensions::bitwise::BitwiseType;
use cairo_lang_sierra::extensions::circuit::{AddModType, MulModType};
use cairo_lang_sierra::extensions::core::{CoreLibfunc, CoreType};
use cairo_lang_sierra::extensions::ec::EcOpType;
use cairo_lang_sierra::extensions::pedersen::PedersenType;
use cairo_lang_sierra::extensions::poseidon::PoseidonType;
use cairo_lang_sierra::extensions::range_check::{RangeCheck96Type, RangeCheckType};
use cairo_lang_sierra::extensions::segment_arena::SegmentArenaType;
use cairo_lang_sierra::ids::{ConcreteTypeId, GenericTypeId};
use cairo_lang_sierra::program::{GenFunction, ProgramArtifact, StatementIdx, TypeDeclaration};
use cairo_lang_sierra::program_registry::ProgramRegistry;
use cairo_lang_sierra_type_size::get_type_size_map;
use cairo_lang_utils::unordered_hash_map::UnorderedHashMap;
use cairo_vm::serde::deserialize_program::ReferenceManager;
use cairo_vm::types::builtin_name::BuiltinName;
use cairo_vm::types::program::Program;
use cairo_vm::types::relocatable::MaybeRelocatable;
use camino::Utf8PathBuf;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::Serialize;
use starknet_types_core::felt::Felt;
use std::collections::HashMap;
use std::sync::Arc;
use universal_sierra_compiler_api::compile_raw_sierra_at_path;
use universal_sierra_compiler_api::representation::RawCasmProgram;

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
    #[tracing::instrument(skip_all, level = "debug")]
    pub fn build(
        func: &GenFunction<StatementIdx>,
        type_declarations: &HashMap<u64, &TypeDeclaration>,
        type_size_map: &UnorderedHashMap<ConcreteTypeId, i16>,
    ) -> TestDetails {
        let map_types = |concrete_types: &[ConcreteTypeId]| {
            concrete_types
                .iter()
                .map(|ty| {
                    let ty = type_declarations[&ty.id];

                    (ty.long_id.generic_id.clone(), type_size_map[&ty.id])
                })
                .collect()
        };

        TestDetails {
            sierra_entry_point_statement_idx: func.entry_point.0,
            parameter_types: map_types(&func.signature.param_types),
            return_types: map_types(&func.signature.ret_types),
        }
    }

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

    pub fn try_into_program(&self, casm_program: &RawCasmProgram) -> Result<Program> {
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

// TODO: Remove in next PRs
#[derive(Debug, Clone)]
pub struct TestTargetDeprecated<C> {
    pub tests_location: TestTargetLocation,
    pub sierra_program: ProgramArtifact,
    pub sierra_program_path: Arc<Utf8PathBuf>,
    pub casm_program: Arc<RawCasmProgram>,
    pub test_cases: Vec<TestCase<C>>,
}

// TODO: Remove in next PRs
impl TestTargetDeprecated<TestCaseWithConfig> {
    #[tracing::instrument(skip_all, level = "debug")]
    pub fn from_raw_deprecated(
        test_target_raw: TestTargetRaw,
        tracked_resource: &ForgeTrackedResource,
    ) -> Result<TestTargetWithConfig> {
        macro_rules! by_id {
            ($field:ident) => {{
                let temp: HashMap<_, _> = test_target_raw
                    .sierra_program
                    .program
                    .$field
                    .iter()
                    .map(|f| (f.id.id, f))
                    .collect();

                temp
            }};
        }
        let funcs = by_id!(funcs);
        let type_declarations = by_id!(type_declarations);

        let casm_program = Arc::new(compile_raw_sierra_at_path(
            test_target_raw.sierra_program_path.as_std_path(),
        )?);

        let sierra_program_registry =
            ProgramRegistry::<CoreType, CoreLibfunc>::new(&test_target_raw.sierra_program.program)?;
        let type_size_map = get_type_size_map(
            &test_target_raw.sierra_program.program,
            &sierra_program_registry,
        )
        .ok_or_else(|| anyhow!("can not get type size map"))?;

        let default_executables = vec![];
        let debug_info = test_target_raw.sierra_program.debug_info.clone();
        let executables = debug_info
            .as_ref()
            .and_then(|info| info.executables.get("snforge_internal_test_executable"))
            .unwrap_or(&default_executables);

        let test_cases = executables
            .par_iter()
            .map(|case| -> Result<TestCaseWithConfig> {
                let func = funcs[&case.id];

                let test_details = TestDetails::build(func, &type_declarations, &type_size_map);

                let raw_config = run_config_pass(&test_details, &casm_program, tracked_resource)?;

                Ok(TestCaseWithConfig {
                    config: raw_config.into(),
                    name: case.debug_name.clone().unwrap().into(),
                    test_details,
                })
            })
            .collect::<Result<_>>()?;

        Ok(TestTargetWithConfig {
            tests_location: test_target_raw.tests_location,
            test_cases,
            sierra_program: test_target_raw.sierra_program,
            sierra_program_path: test_target_raw.sierra_program_path.into(),
            casm_program,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestCase<C> {
    pub test_details: TestDetails,
    pub name: String,
    pub config: C,
}

#[derive(Debug, Clone)]
pub struct TestTarget<T> {
    pub tests_location: TestTargetLocation,
    pub sierra_program: ProgramArtifact,
    pub sierra_program_path: Arc<Utf8PathBuf>,
    pub test_cases: Vec<T>,
}

impl TestTarget<TestCandidate> {
    #[tracing::instrument(skip_all, level = "debug")]
    pub fn from_raw(test_target_raw: TestTargetRaw) -> Result<TestTarget<TestCandidate>> {
        macro_rules! by_id {
            ($field:ident) => {{
                let temp: HashMap<_, _> = test_target_raw
                    .sierra_program
                    .program
                    .$field
                    .iter()
                    .map(|f| (f.id.id, f))
                    .collect();

                temp
            }};
        }
        let funcs = by_id!(funcs);
        let type_declarations = by_id!(type_declarations);

        let sierra_program_registry =
            ProgramRegistry::<CoreType, CoreLibfunc>::new(&test_target_raw.sierra_program.program)?;
        let type_size_map = get_type_size_map(
            &test_target_raw.sierra_program.program,
            &sierra_program_registry,
        )
        .ok_or_else(|| anyhow!("can not get type size map"))?;

        let default_executables = vec![];
        let debug_info = test_target_raw.sierra_program.debug_info.clone();
        let executables = debug_info
            .as_ref()
            .and_then(|info| info.executables.get("snforge_internal_test_executable"))
            .unwrap_or(&default_executables);

        let test_cases = executables
            .par_iter()
            .map(|case| {
                let func = funcs[&case.id];
                let name = case.debug_name.clone().unwrap().into();
                let test_details = TestDetails::build(func, &type_declarations, &type_size_map);

                Ok(TestCandidate { name, test_details })
            })
            .collect::<Result<_>>()?;

        Ok(TestTarget {
            tests_location: test_target_raw.tests_location,
            test_cases,
            sierra_program: test_target_raw.sierra_program,
            sierra_program_path: test_target_raw.sierra_program_path.into(),
        })
    }
}

pub struct TestCandidate {
    pub name: String,
    pub test_details: TestDetails,
}
