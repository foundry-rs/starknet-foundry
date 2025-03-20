use crate::utils::{get_statements, TypedSyntaxNodeAsText};
use crate::{
    args::Arguments,
    attributes::AttributeCollector,
    common::{into_proc_macro_result, with_parsed_values},
};
use cairo_lang_macro::{Diagnostic, Diagnostics, ProcMacroResult, TokenStream};
use cairo_lang_syntax::node::{ast::FunctionWithBody, db::SyntaxGroup};
use indoc::formatdoc;

pub fn extend_with_config_cheatcodes<Collector>(
    args: TokenStream,
    item: TokenStream,
) -> ProcMacroResult
where
    Collector: AttributeCollector,
{
    into_proc_macro_result(args, item, |args, item, warns| {
        with_parsed_values::<Collector>(args, item, warns, with_config_cheatcodes::<Collector>)
    })
}

fn with_config_cheatcodes<Collector>(
    db: &dyn SyntaxGroup,
    func: &FunctionWithBody,
    args_db: &dyn SyntaxGroup,
    args: Arguments,
    warns: &mut Vec<Diagnostic>,
) -> Result<String, Diagnostics>
where
    Collector: AttributeCollector,
{
    let value = Collector::args_into_config_expression(args_db, args, warns)?;

    let cheatcode_name = Collector::CHEATCODE_NAME;

    let config_cheatcode = formatdoc!(
        r"
            let mut data = array![];

            {value}
            .serialize(ref data);

            starknet::testing::cheatcode::<'{cheatcode_name}'>(data.span());
        "
    );

    Ok(append_config_statements(db, func, &config_cheatcode))
}

pub fn append_config_statements(
    db: &dyn SyntaxGroup,
    func: &FunctionWithBody,
    config_statements: &str,
) -> String {
    let vis = func.visibility(db).as_text(db);
    let attrs = func.attributes(db).as_text(db);
    let declaration = func.declaration(db).as_text(db);

    let (statements, if_content) = get_statements(db, func);

    formatdoc!(
        "
            {attrs}
            {vis} {declaration} {{
                if snforge_std::_internals::_is_config_run() {{
                    {if_content}

                    {config_statements}

                    return;
                }}

                {statements}
            }}
        "
    )
}
