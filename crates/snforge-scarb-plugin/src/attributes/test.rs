use super::{internal_config_statement::InternalConfigStatementCollector, AttributeInfo};
use crate::{
    args::Arguments,
    common::{into_proc_macro_result, with_parsed_values},
};
use cairo_lang_macro::{Diagnostic, Diagnostics, ProcMacroResult, TokenStream};
use cairo_lang_syntax::node::{ast::FunctionWithBody, db::SyntaxGroup, Terminal, TypedSyntaxNode};
use indoc::formatdoc;
use itertools::Itertools;
use std::env;

struct TestCollector;

impl AttributeInfo for TestCollector {
    const ATTR_NAME: &'static str = "test";
}

#[must_use]
pub fn test(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    into_proc_macro_result(args, item, |args, item, warns| {
        with_parsed_values::<TestCollector>(args, item, warns, test_internal)
    })
}

#[allow(clippy::ptr_arg)]
#[allow(clippy::needless_pass_by_value)]
fn test_internal(
    db: &dyn SyntaxGroup,
    func: &FunctionWithBody,
    _args_db: &dyn SyntaxGroup,
    args: Arguments,
    _warns: &mut Vec<Diagnostic>,
) -> Result<String, Diagnostics> {
    args.assert_is_empty::<TestCollector>()?;

    let config = InternalConfigStatementCollector::ATTR_NAME;

    let func_item = func.as_syntax_node().get_text(db);
    let name = func.declaration(db).name(db).text(db).to_string();

    // TODO use const
    let tests_to_run = env::var("SNFORGE_TEST_FILTER_NAMES");

    if let Ok(tests_to_run) = tests_to_run {
        let mut tests_to_run = tests_to_run.split(',');
        if tests_to_run.contains(&name.as_str()) {
            return Ok(formatdoc!(
                "
            #[snforge_internal_test_executable]
            #[{config}]
            {func_item}
        "
            ));
        }

        return Ok(formatdoc!(
            "
            #[{config}]
            {func_item}
        "
        ));
    }

    let result = formatdoc!(
        "
            #[snforge_internal_test_executable]
            #[{config}]
            {func_item}
        "
    );

    Ok(result)
}
