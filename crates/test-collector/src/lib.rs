use crate::sierra_casm_generator::SierraCasmGenerator;
use anyhow::{anyhow, bail, Context, Result};
use cairo_felt::Felt252;
use cairo_lang_compiler::db::RootDatabase;
use cairo_lang_compiler::diagnostics::DiagnosticsReporter;
use cairo_lang_debug::DebugWithDb;
use cairo_lang_defs::db::DefsGroup;
use cairo_lang_defs::ids::{FreeFunctionId, FunctionWithBodyId, ModuleId, ModuleItemId};
use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::ToOption;
use cairo_lang_filesystem::cfg::{Cfg, CfgSet};
use cairo_lang_filesystem::db::{AsFilesGroupMut, FilesGroup, FilesGroupEx};
use cairo_lang_filesystem::ids::{CrateId, CrateLongId, Directory};
use cairo_lang_lowering::ids::ConcreteFunctionWithBodyId;
use cairo_lang_project::{ProjectConfig, ProjectConfigContent};
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::items::functions::GenericFunctionId;
use cairo_lang_semantic::{ConcreteFunction, FunctionLongId};
use cairo_lang_sierra::extensions::enm::EnumType;
use cairo_lang_sierra::extensions::NamedType;
use cairo_lang_sierra::program::{GenericArg, Program};
use cairo_lang_sierra_generator::db::SierraGenGroup;
use cairo_lang_sierra_generator::replace_ids::replace_sierra_ids_in_program;
use cairo_lang_starknet::inline_macros::get_dep_component::{
    GetDepComponentMacro, GetDepComponentMutMacro,
};
use cairo_lang_starknet::inline_macros::selector::SelectorMacro;
use cairo_lang_starknet::plugin::StarkNetPlugin;
use cairo_lang_syntax::attribute::structured::{Attribute, AttributeArg, AttributeArgVariant};
use cairo_lang_syntax::node::ast;
use cairo_lang_syntax::node::ast::{ArgClause, Expr};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::helpers::GetIdentifier;
use cairo_lang_test_plugin::test_config::{PanicExpectation, TestExpectation};
use cairo_lang_test_plugin::{try_extract_test_config, TestConfig};
use cairo_lang_utils::ordered_hash_map::OrderedHashMap;
use cairo_lang_utils::OptionHelper;
use conversions::StarknetConversions;
use itertools::Itertools;
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use plugin::TestPlugin;
use smol_str::SmolStr;
use starknet::core::types::{BlockId, BlockTag};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod plugin;

pub mod sierra_casm_generator;

const FORK_ATTR: &str = "fork";
const FUZZER_ATTR: &str = "fuzzer";

/// Expectation for a panic case.
#[derive(Debug, Clone, PartialEq)]
pub enum ExpectedPanicValue {
    /// Accept any panic value.
    Any,
    /// Accept only this specific vector of panics.
    Exact(Vec<Felt252>),
}

impl From<PanicExpectation> for ExpectedPanicValue {
    fn from(value: PanicExpectation) -> Self {
        match value {
            PanicExpectation::Any => ExpectedPanicValue::Any,
            PanicExpectation::Exact(vec) => ExpectedPanicValue::Exact(vec),
        }
    }
}

/// Expectation for a result of a test.
#[derive(Debug, Clone, PartialEq)]
pub enum ExpectedTestResult {
    /// Running the test should not panic.
    Success,
    /// Running the test should result in a panic.
    Panics(ExpectedPanicValue),
}

impl From<TestExpectation> for ExpectedTestResult {
    fn from(value: TestExpectation) -> Self {
        match value {
            TestExpectation::Success => ExpectedTestResult::Success,
            TestExpectation::Panics(panic_expectation) => {
                ExpectedTestResult::Panics(panic_expectation.into())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RawForkConfig {
    Id(String),
    Params(RawForkParams),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawForkParams {
    pub url: String,
    pub block_id: BlockId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FuzzerConfig {
    pub fuzzer_runs: u32,
    pub fuzzer_seed: u64,
}

/// The configuration for running a single test.
#[derive(Debug)]
pub struct SingleTestConfig {
    /// The amount of gas the test requested.
    pub available_gas: Option<usize>,
    /// The expected result of the run.
    pub expected_result: ExpectedTestResult,
    /// Should the test be ignored.
    pub ignored: bool,
    /// The configuration of forked network.
    pub fork_config: Option<RawForkConfig>,
    /// Custom fuzzing configuration
    pub fuzzer_config: Option<FuzzerConfig>,
}

/// Finds the tests in the requested crates.
pub fn find_all_tests(
    db: &dyn SemanticGroup,
    main_crate: CrateId,
) -> Vec<(FreeFunctionId, SingleTestConfig)> {
    let mut tests = vec![];
    let modules = db.crate_modules(main_crate);
    for module_id in modules.iter() {
        let Ok(module_items) = db.module_items(*module_id) else {
            continue;
        };
        tests.extend(module_items.iter().filter_map(|item| {
            let ModuleItemId::FreeFunction(func_id) = item else {
                return None;
            };
            let Ok(attrs) = db.function_with_body_attributes(FunctionWithBodyId::Free(*func_id))
            else {
                return None;
            };
            Some((
                *func_id,
                forge_try_extract_test_config(db.upcast(), &attrs).unwrap()?,
            ))
        }));
    }
    tests
}

/// Extracts the configuration of a tests from attributes, or returns the diagnostics if the
/// attributes are set illegally.
pub fn forge_try_extract_test_config(
    db: &dyn SyntaxGroup,
    attrs: &[Attribute],
) -> Result<Option<SingleTestConfig>, Vec<PluginDiagnostic>> {
    let maybe_test_config = try_extract_test_config(db, attrs.to_vec())?;
    let fork_attr = attrs.iter().find(|attr| attr.id.as_str() == FORK_ATTR);
    let fuzzer_attr = attrs.iter().find(|attr| attr.id.as_str() == FUZZER_ATTR);

    let mut diagnostics = vec![];

    if maybe_test_config.is_none() {
        for attr in [fork_attr, fuzzer_attr].into_iter().flatten() {
            diagnostics.push(PluginDiagnostic {
                stable_ptr: attr.id_stable_ptr.untyped(),
                message: "Attribute should only appear on tests.".into(),
            });
        }
    }

    let fork_config = if let Some(attr) = fork_attr {
        if attr.args.is_empty() {
            None
        } else {
            extract_fork_config(db, attr).on_none(|| {
                diagnostics.push(PluginDiagnostic {
                    stable_ptr: attr.args_stable_ptr.untyped(),
                    message: "Expected fork config must be of the form `url: <double quote \
                                  string>, block_id: <snforge_std::BlockId>`."
                        .into(),
                });
            })
        }
    } else {
        None
    };

    let fuzzer_config = if let Some(attr) = fuzzer_attr {
        extract_fuzzer_config(db, attr).on_none(|| {
            diagnostics.push(PluginDiagnostic {
                stable_ptr: attr.args_stable_ptr.untyped(),
                message: "Expected fuzzer config must be of the form `runs: <u32>, seed: <u64>`"
                    .into(),
            });
        })
    } else {
        None
    };

    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    let result = maybe_test_config.map(
        |TestConfig {
             available_gas,
             expectation,
             ignored,
         }| SingleTestConfig {
            available_gas,
            expected_result: expectation.into(),
            ignored,
            fork_config,
            fuzzer_config,
        },
    );
    Ok(result)
}

/// Tries to extract the fork configuration.
fn extract_fork_config(db: &dyn SyntaxGroup, attr: &Attribute) -> Option<RawForkConfig> {
    if attr.args.is_empty() {
        return None;
    }

    match &attr.args[0].variant {
        AttributeArgVariant::Unnamed { value: fork_id, .. } => {
            extract_fork_config_from_id(fork_id, db)
        }
        _ => extract_fork_config_from_args(db, attr),
    }
}

fn extract_fuzzer_config(db: &dyn SyntaxGroup, attr: &Attribute) -> Option<FuzzerConfig> {
    let [AttributeArg {
        variant:
            AttributeArgVariant::Named {
                name: fuzzer_runs_name,
                value: fuzzer_runs,
                ..
            },
        ..
    }, AttributeArg {
        variant:
            AttributeArgVariant::Named {
                name: fuzzer_seed_name,
                value: fuzzer_seed,
                ..
            },
        ..
    }] = &attr.args[..]
    else {
        return None;
    };

    if fuzzer_runs_name != "runs" || fuzzer_seed_name != "seed" {
        return None;
    };

    let fuzzer_runs = extract_numeric_value(db, fuzzer_runs)?.to_u32()?;
    let fuzzer_seed = extract_numeric_value(db, fuzzer_seed)?.to_u64()?;

    Some(FuzzerConfig {
        fuzzer_runs,
        fuzzer_seed,
    })
}

fn extract_numeric_value(db: &dyn SyntaxGroup, expr: &ast::Expr) -> Option<BigInt> {
    let ast::Expr::Literal(literal) = expr else {
        return None;
    };

    literal.numeric_value(db)
}

fn extract_fork_config_from_id(id: &ast::Expr, db: &dyn SyntaxGroup) -> Option<RawForkConfig> {
    let ast::Expr::String(id_str) = id else {
        return None;
    };
    let id = id_str.string_value(db)?;

    Some(RawForkConfig::Id(id))
}

fn extract_fork_config_from_args(db: &dyn SyntaxGroup, attr: &Attribute) -> Option<RawForkConfig> {
    let [AttributeArg {
        variant:
            AttributeArgVariant::Named {
                name: url_arg_name,
                value: url,
                ..
            },
        ..
    }, AttributeArg {
        variant:
            AttributeArgVariant::Named {
                name: block_id_arg_name,
                value: block_id,
                ..
            },
        ..
    }] = &attr.args[..]
    else {
        return None;
    };

    if url_arg_name != "url" {
        return None;
    }
    let ast::Expr::String(url_str) = url else {
        return None;
    };
    let url = url_str.string_value(db)?;

    if block_id_arg_name != "block_id" {
        return None;
    }
    let ast::Expr::FunctionCall(block_id) = block_id else {
        return None;
    };

    let elements: Vec<String> = block_id
        .path(db)
        .elements(db)
        .iter()
        .map(|e| e.identifier(db).to_string())
        .collect();
    if !(elements.len() == 2
        && elements[0] == "BlockId"
        && ["Number", "Hash", "Tag"].contains(&elements[1].as_str()))
    {
        return None;
    }

    let block_id_type = block_id
        .path(db)
        .elements(db)
        .last()
        .unwrap()
        .identifier(db)
        .to_string();

    let args = block_id.arguments(db).args(db).elements(db);
    let expr = match args.first()?.arg_clause(db) {
        ArgClause::Unnamed(unnamed_arg_clause) => Some(unnamed_arg_clause.value(db)),
        _ => None,
    }?;
    let block_id = try_get_block_id(db, &block_id_type, &expr)?;

    Some(RawForkConfig::Params(RawForkParams { url, block_id }))
}

fn try_get_block_id(db: &dyn SyntaxGroup, block_id_type: &str, expr: &Expr) -> Option<BlockId> {
    match block_id_type {
        "Number" => {
            if let Expr::Literal(value) = expr {
                return Some(BlockId::Number(
                    u64::try_from(value.numeric_value(db).unwrap()).ok()?,
                ));
            }
            None
        }
        "Hash" => {
            if let Expr::Literal(value) = expr {
                return Some(BlockId::Hash(
                    Felt252::from(value.numeric_value(db).unwrap()).to_field_element(),
                ));
            }
            None
        }
        "Tag" => {
            if let Expr::Path(block_tag) = expr {
                let tag = block_tag
                    .elements(db)
                    .last()
                    .unwrap()
                    .identifier(db)
                    .to_string();
                return match tag.as_str() {
                    "Latest" => Some(BlockId::Tag(BlockTag::Latest)),
                    _ => None,
                };
            }
            None
        }
        _ => None,
    }
}

/// Represents a dependency of a Cairo project
#[derive(Debug, Clone)]
pub struct LinkedLibrary {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TestCaseRaw {
    pub name: String,
    pub available_gas: Option<usize>,
    pub ignored: bool,
    pub expected_result: ExpectedTestResult,
    pub fork_config: Option<RawForkConfig>,
    pub fuzzer_config: Option<FuzzerConfig>,
}

pub fn collect_tests(
    crate_name: &str,
    crate_root: &Path,
    lib_content: &str,
    linked_libraries: &[LinkedLibrary],
    builtins: &[&str],
    corelib_path: PathBuf,
    output_path: Option<&str>,
) -> Result<(Program, Vec<TestCaseRaw>)> {
    let crate_roots: OrderedHashMap<SmolStr, PathBuf> = linked_libraries
        .iter()
        .cloned()
        .map(|source_root| (source_root.name.into(), source_root.path))
        .collect();

    let project_config = ProjectConfig {
        base_path: crate_root.into(),
        corelib: Some(Directory::Real(corelib_path)),
        content: ProjectConfigContent { crate_roots },
    };

    // code taken from crates/cairo-lang-test-runner/src/lib.rs
    let db = &mut {
        let mut b = RootDatabase::builder();
        b.with_cfg(CfgSet::from_iter([Cfg::name("test")]));
        b.with_macro_plugin(Arc::new(TestPlugin::default()));
        b.with_macro_plugin(Arc::new(StarkNetPlugin::default()));
        b.with_inline_macro_plugin(SelectorMacro::NAME, Arc::new(SelectorMacro));
        b.with_inline_macro_plugin(GetDepComponentMacro::NAME, Arc::new(GetDepComponentMacro));
        b.with_inline_macro_plugin(
            GetDepComponentMutMacro::NAME,
            Arc::new(GetDepComponentMutMacro),
        );
        b.with_project_config(project_config);
        b.build()?
    };

    let main_crate_id =
        insert_lib_entrypoint_content_into_db(db, crate_name, crate_root, lib_content);

    if DiagnosticsReporter::stderr().check(db) {
        return Err(anyhow!(
            "Failed to add linked library, for a detailed information, please go through the logs \
             above"
        ));
    }
    let all_tests = find_all_tests(db, main_crate_id);

    let z: Vec<ConcreteFunctionWithBodyId> = all_tests
        .iter()
        .filter_map(|(func_id, _cfg)| {
            ConcreteFunctionWithBodyId::from_no_generics_free(db, *func_id)
        })
        .collect();

    let sierra_program = db
        .get_sierra_program_for_functions(z)
        .to_option()
        .context("Compilation failed without any diagnostics")
        .context("Failed to get sierra program")?;

    let collected_tests: Vec<TestCaseRaw> = all_tests
        .into_iter()
        .map(|(func_id, test)| {
            (
                format!(
                    "{:?}",
                    FunctionLongId {
                        function: ConcreteFunction {
                            generic_function: GenericFunctionId::Free(func_id),
                            generic_args: vec![]
                        }
                    }
                    .debug(db)
                ),
                test,
            )
        })
        .collect_vec()
        .into_iter()
        .map(|(test_name, config)| {
            if config.available_gas.is_some() {
                bail!("{} - Attribute `available_gas` is not supported: Contract functions execution cost would not be included in the gas calculation.", test_name)
            };
            Ok(TestCaseRaw {
                name: test_name,
                available_gas: config.available_gas,
                ignored: config.ignored,
                expected_result: config.expected_result,
                fork_config: config.fork_config,
                fuzzer_config: config.fuzzer_config,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let sierra_program = replace_sierra_ids_in_program(db, &sierra_program);

    validate_tests(sierra_program.clone(), &collected_tests, builtins)?;

    if let Some(path) = output_path {
        fs::write(path, sierra_program.to_string()).context("Failed to write output")?;
    }
    Ok((sierra_program, collected_tests))
}

// inspired with cairo-lang-compiler/src/project.rs:49 (part of setup_single_project_file)
fn insert_lib_entrypoint_content_into_db(
    db: &mut RootDatabase,
    crate_name: &str,
    crate_root: &Path,
    lib_content: &str,
) -> CrateId {
    let main_crate_id = db.intern_crate(CrateLongId::Real(SmolStr::from(crate_name)));
    db.set_crate_root(
        main_crate_id,
        Some(Directory::Real(crate_root.to_path_buf())),
    );

    let module_id = ModuleId::CrateRoot(main_crate_id);
    let file_id = db.module_main_file(module_id).unwrap();
    db.as_files_group_mut()
        .override_file_content(file_id, Some(Arc::new(lib_content.to_string())));

    main_crate_id
}

fn validate_tests(
    sierra_program: Program,
    collected_tests: &Vec<TestCaseRaw>,
    ignored_params: &[&str],
) -> Result<(), anyhow::Error> {
    let casm_generator = match SierraCasmGenerator::new(sierra_program) {
        Ok(casm_generator) => casm_generator,
        Err(e) => panic!("{}", e),
    };
    for test in collected_tests {
        let func = casm_generator.find_function(&test.name)?;
        let mut filtered_params: Vec<String> = Vec::new();
        for param in &func.params {
            let param_str = &param.ty.debug_name.as_ref().unwrap().to_string();
            if !ignored_params.contains(&param_str.as_str()) {
                filtered_params.push(param_str.to_string());
            }
        }

        let signature = &func.signature;
        let ret_types = &signature.ret_types;
        let tp = &ret_types[ret_types.len() - 1];
        let info = casm_generator.get_info(tp);
        let mut maybe_return_type_name = None;
        if info.long_id.generic_id == EnumType::ID {
            if let GenericArg::UserType(ut) = &info.long_id.generic_args[0] {
                if let Some(name) = ut.debug_name.as_ref() {
                    maybe_return_type_name = Some(name.as_str());
                }
            }
        }
        if let Some(return_type_name) = maybe_return_type_name {
            if !return_type_name.starts_with("core::panics::PanicResult::") {
                anyhow::bail!(
                    "The test function {} always succeeds and cannot be used as a test. Make sure to include panickable statements such as `assert` in your test",
                    test.name
                );
            }
            if return_type_name != "core::panics::PanicResult::<((),)>" {
                anyhow::bail!(
                    "Test function {} returns a value {}, it is required that test functions do \
                     not return values",
                    test.name,
                    return_type_name
                );
            }
        } else {
            anyhow::bail!(
                "Couldn't read result type for test function {} possible cause: The test function {} \
                 always succeeds and cannot be used as a test. Make sure to include panickable statements such as `assert` in your test",
                test.name,
                test.name
            );
        }
    }

    Ok(())
}
