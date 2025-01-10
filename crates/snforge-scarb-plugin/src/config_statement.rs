use crate::{
    args::Arguments,
    attributes::AttributeCollector,
    common::{into_proc_macro_result, with_parsed_values},
};
use cairo_lang_macro::{Diagnostic, Diagnostics, ProcMacroResult, TokenStream};
use cairo_lang_syntax::node::{
    ast::{Condition, Expr, FunctionWithBody, Statement},
    db::SyntaxGroup,
    helpers::GetIdentifier,
    TypedSyntaxNode,
};
use indoc::formatdoc;

#[allow(clippy::needless_pass_by_value)]
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
    let vis = func.visibility(db).as_syntax_node().get_text(db);
    let attrs = func.attributes(db).as_syntax_node().get_text(db);
    let declaration = func.declaration(db).as_syntax_node().get_text(db);
    let statements = func.body(db).statements(db).elements(db);

    let if_content = statements.first().and_then(|stmt| {
        // first statement is `if`
        let Statement::Expr(expr) = stmt else {
            return None;
        };
        let Expr::If(if_expr) = expr.expr(db) else {
            return None;
        };
        // it's condition is function call
        let Condition::Expr(expr) = if_expr.condition(db) else {
            return None;
        };
        let Expr::FunctionCall(expr) = expr.expr(db) else {
            return None;
        };

        // this function is named "snforge_std::_internals::_is_config_run"
        let segments = expr.path(db).elements(db);

        let [snforge_std, cheatcode, is_config_run] = segments.as_slice() else {
            return None;
        };

        if snforge_std.identifier(db) != "snforge_std"
            || cheatcode.identifier(db) != "_internals"
            || is_config_run.identifier(db) != "_is_config_run"
        {
            return None;
        }

        let statements = if_expr.if_block(db).statements(db).elements(db);

        // omit last one (`return;`) as it have to be inserted after all new statements
        Some(
            statements[..statements.len() - 1]
                .iter()
                .fold(String::new(), |acc, statement| {
                    acc + "\n" + &statement.as_syntax_node().get_text(db)
                }),
        )
    });

    // there was already config check, omit it and collect remaining statements
    let statements = if if_content.is_some() {
        &statements[1..]
    } else {
        &statements[..]
    }
    .iter()
    .fold(String::new(), |acc, stmt| {
        acc + &stmt.as_syntax_node().get_text(db)
    });

    let if_content = if_content.unwrap_or_default();

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
