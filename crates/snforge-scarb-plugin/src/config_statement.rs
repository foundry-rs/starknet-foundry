use crate::{
    args::Arguments,
    asserts::assert_is_used_once,
    attributes::AttributeCollector,
    parse::{parse, parse_args},
};
use cairo_lang_macro::{Diagnostic, Diagnostics, ProcMacroResult, TokenStream};
use cairo_lang_syntax::node::{
    ast::{Condition, Expr, FunctionWithBody, GenericArg, GenericArgValue, Statement},
    db::SyntaxGroup,
    helpers::{GetIdentifier, PathSegmentEx},
    TypedSyntaxNode,
};
use cairo_lang_utils::Upcast;
use indoc::formatdoc;

#[allow(clippy::needless_pass_by_value)]
pub fn extend_with_config_cheatcodes<Collector>(
    args: TokenStream,
    item: TokenStream,
) -> ProcMacroResult
where
    Collector: AttributeCollector,
{
    let mut warns = vec![]; // Vec<Diagnostic> instead of Diagnostics because it does not allow to push ready Diagnostic

    match extend_with_config_cheatcodes_internal::<Collector>(&args, &item, &mut warns) {
        Ok(item) => ProcMacroResult::new(TokenStream::new(item)).with_diagnostics(warns.into()),
        Err(diagnostics) => ProcMacroResult::new(item).with_diagnostics(
            //TODO extend with warns
            diagnostics,
        ),
    }
}

fn extend_with_config_cheatcodes_internal<Collector>(
    args: &TokenStream,
    item: &TokenStream,
    warns: &mut Vec<Diagnostic>,
) -> Result<String, Diagnostics>
where
    Collector: AttributeCollector,
{
    let item = item.to_string();
    let (db, func) = parse::<Collector>(&item)?;

    let db = db.upcast();

    assert_is_used_once::<Collector>(db, &func)?;

    let (args_db, args) = parse_args::<Collector>(&args.to_string())?;

    let args = Arguments::new::<Collector>(args_db.upcast(), args, warns);

    let value = Collector::args_into_config_expression(args_db.upcast(), args, warns)?;

    let cheatcode_name = Collector::CHEATCODE_NAME;

    let config_cheatcode = formatdoc!(
        r#"
            let mut data = array![];

            {value}
            .serialize(ref data);

            starknet::testing::cheatcode::<'{cheatcode_name}'>(data);
        "#
    );

    Ok(append_config_statements(db, &func, &config_cheatcode))
}

const CONFIG_CHEATCODE: &str = "is_config_mode";

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

        // this function is named "starknet::testing::cheatcode"
        let segments = expr.path(db).elements(db);

        let [starknet, testing, cheatcode] = segments.as_slice() else {
            return None;
        };

        if starknet.identifier(db) != "starknet"
            || testing.identifier(db) != "testing"
            || cheatcode.identifier(db) != "cheatcode"
        {
            return None;
        }

        // it has single, unnamed generic argument
        let generics = cheatcode.generic_args(db)?;

        let [GenericArg::Unnamed(cheatcode)] = generics.as_slice() else {
            return None;
        };
        let GenericArgValue::Expr(expr) = cheatcode.value(db) else {
            return None;
        };
        // of type short string
        let Expr::ShortString(str) = expr.expr(db) else {
            return None;
        };

        // equal to configuration cheatcode
        if str.string_value(db)? == CONFIG_CHEATCODE {
            Some(if_expr.if_block(db).as_syntax_node().get_text(db))
        } else {
            None
        }
    });

    // there was already config check, omit it and collect remaining statements
    // also omit last one (`return;`) as it have to be inserted after all new statements
    let statements = if if_content.is_some() {
        &statements[1..statements.len() - 1]
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
                if starknet::testing::cheatcode::<'{CONFIG_CHEATCODE}'>() {{
                    {if_content}

                    {config_statements}

                    return;
                }}

                {statements}
            }}
        "
    )
}
