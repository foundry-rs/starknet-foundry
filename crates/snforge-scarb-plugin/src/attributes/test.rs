use super::{internal_config_statement::InternalConfigStatementCollector, AttributeInfo, ErrorExt};
use crate::attributes::fuzzer::wrapper::FuzzerWrapperCollector;
use crate::attributes::fuzzer::{FuzzerCollector, FuzzerConfigCollector};
use crate::utils::TypedSyntaxNodeAsText;
use crate::{
    args::Arguments,
    common::{into_proc_macro_result, with_parsed_values},
};
use cairo_lang_macro::{Diagnostic, Diagnostics, ProcMacroResult, TokenStream};
use cairo_lang_syntax::node::{ast::FunctionWithBody, db::SyntaxGroup, Terminal, TypedSyntaxNode};
use indoc::formatdoc;
use std::env::{self, VarError};
use std::ops::Not;

pub struct TestCollector;

impl AttributeInfo for TestCollector {
    const ATTR_NAME: &'static str = "test";
}

#[must_use]
pub fn test(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    into_proc_macro_result(args, item, |args, item, warns| {
        with_parsed_values::<TestCollector>(args, item, warns, test_internal)
    })
}

#[expect(clippy::ptr_arg)]
#[expect(clippy::needless_pass_by_value)]
fn test_internal(
    db: &dyn SyntaxGroup,
    func: &FunctionWithBody,
    _args_db: &dyn SyntaxGroup,
    args: Arguments,
    _warns: &mut Vec<Diagnostic>,
) -> Result<String, Diagnostics> {
    args.assert_is_empty::<TestCollector>()?;
    ensure_parameters_only_with_fuzzer_attribute(db, func)?;

    let config = InternalConfigStatementCollector::ATTR_NAME;

    let _func_item = func.as_syntax_node().get_text(db);
    let name = func.declaration(db).name(db).text(db).to_string();

    let test_filter = get_forge_test_filter().ok();
    let test_skip = get_forge_test_skip().ok();

    let should_run_test = match test_filter {
        Some(ref filter) => name.contains(filter),
        None => true,
    };

    let should_run_test = match test_skip {
        Some(filter) => !filter.split("<<>>").any(|s| name.contains(s)),
        None => should_run_test,
    };

    let name = func.declaration(db).name(db).as_text(db);
    let signature = func.declaration(db).signature(db).as_text(db);
    let statements = func.body(db).statements(db).as_text(db);
    let attributes = func.attributes(db).as_text(db);

    if should_run_test {
        Ok(formatdoc!(
            "
            #[snforge_internal_test_executable]
            #[implicit_precedence(core::pedersen::Pedersen, core::RangeCheck, core::integer::Bitwise, core::ec::EcOp, core::poseidon::Poseidon, core::SegmentArena, core::circuit::RangeCheck96, core::circuit::AddMod, core::circuit::MulMod, core::gas::GasBuiltin, System)]
            fn {name}(mut _data: Span<felt252>) -> Span::<felt252> {{
                core::internal::require_implicit::<System>();
                core::internal::revoke_ap_tracking();
                core::option::OptionTraitImpl::expect(core::gas::withdraw_gas(), 'Out of gas');

                core::option::OptionTraitImpl::expect(
                    core::gas::withdraw_gas_all(core::gas::get_builtin_costs()), 'Out of gas',
                );
                {name}_return_wrapper();

                let mut arr = ArrayTrait::new();
                core::array::ArrayTrait::span(@arr)
            }}

            {attributes}
            #[{config}]
            fn {name}_return_wrapper{signature} {{
                {statements}
            }}
        "
        ))
    } else {
        Ok(formatdoc!(
            "
        "
        ))
    }
}

fn get_forge_test_filter() -> Result<String, VarError> {
    env::var("SNFORGE_TEST_FILTER")
}

fn get_forge_test_skip() -> Result<String, VarError> {
    env::var("SNFORGE_TEST_SKIP")
}
fn ensure_parameters_only_with_fuzzer_attribute(
    db: &dyn SyntaxGroup,
    func: &FunctionWithBody,
) -> Result<(), Diagnostic> {
    if has_parameters(db, func) && no_fuzzer_attribute(db, func) {
        Err(TestCollector::error(
            "function with parameters must have #[fuzzer] attribute",
        ))?;
    }

    Ok(())
}

fn has_parameters(db: &dyn SyntaxGroup, func: &FunctionWithBody) -> bool {
    func.declaration(db)
        .signature(db)
        .parameters(db)
        .elements(db)
        .is_empty()
        .not()
}

fn no_fuzzer_attribute(db: &dyn SyntaxGroup, func: &FunctionWithBody) -> bool {
    const FUZZER_ATTRIBUTES: [&str; 3] = [
        FuzzerCollector::ATTR_NAME,
        FuzzerWrapperCollector::ATTR_NAME,
        FuzzerConfigCollector::ATTR_NAME,
    ];

    func.attributes(db)
        .elements(db)
        .iter()
        .any(|attr| {
            FUZZER_ATTRIBUTES.contains(
                &attr
                    .attr(db)
                    .as_syntax_node()
                    .get_text_without_trivia(db)
                    .as_str(),
            )
        })
        .not()
}
