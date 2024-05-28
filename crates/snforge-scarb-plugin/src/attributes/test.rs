use super::{internal_config_statement::InternalConfigStatementCollector, AttributeInfo};
use crate::{
    args::Arguments,
    common::{into_proc_macro_result, with_parsed_values},
};
use cairo_lang_macro::{Diagnostic, Diagnostics, ProcMacroResult, TokenStream};
use cairo_lang_syntax::node::{ast::FunctionWithBody, db::SyntaxGroup, TypedSyntaxNode};
use indoc::formatdoc;

struct TestCollector;

impl AttributeInfo for TestCollector {
    const ATTR_NAME: &'static str = "test";
    const ARGS_FORM: &'static str = "";
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

    let result = formatdoc!(
        "
            #[snforge_internal_test_executable]
            #[{config}]
            {func_item}
        "
    );

    Ok(result)
}
