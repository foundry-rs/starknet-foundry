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
    match extend_with_config_cheatcodes_internal::<Collector>(&args, &item) {
        Ok((item, warn)) => {
            let result = ProcMacroResult::new(TokenStream::new(item));

            if let Some(warn) = warn {
                result.with_diagnostics(warn.into())
            } else {
                result
            }
        }
        Err(diagnostics) => ProcMacroResult::new(item).with_diagnostics(diagnostics),
    }
}

fn extend_with_config_cheatcodes_internal<Collector>(
    args: &TokenStream,
    item: &TokenStream,
) -> Result<(String, Option<Diagnostic>), Diagnostics>
where
    Collector: AttributeCollector,
{
    let item = item.to_string();
    let (db, func) = parse::<Collector>(&item)?;

    let db = db.upcast();

    assert_is_used_once::<Collector>(db, &func)?;

    let (args_db, args) = parse_args::<Collector>(&args.to_string())?;

    let (args, empty_args_list_warn) = Arguments::new::<Collector>(args_db.upcast(), args);

    let value = Collector::args_into_body(args_db.upcast(), args).map_err(|err| {
        if let Some(empty_args_list_warn) = &empty_args_list_warn {
            err.warn(&empty_args_list_warn.message)
        } else {
            err
        }
    })?;

    let cheatcode_name = Collector::CHEATCODE_NAME;

    let config_cheatcode = formatdoc!(
        r#"
            let mut data = array![];

            {value}
            .serialize(ref data);

            cheatcode::<'{cheatcode_name}'>(data);
        "#
    );

    Ok((
        append_config_statements(db, &func, &config_cheatcode),
        empty_args_list_warn,
    ))
}

const CONFIG_CHEATCODE: &str = "is_config_mode";

pub fn append_config_statements(
    db: &dyn SyntaxGroup,
    func: &FunctionWithBody,
    extra_statements: &str,
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

        // this function is named "cheatcode"
        let segments = expr.path(db).elements(db);

        let [segment] = segments.as_slice() else {
            return None;
        };

        if segment.identifier(db) != "cheatcode" {
            return None;
        }

        // it has single, unnamed generic argument
        let generics = segment.generic_args(db)?;

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
                if cheatcode::<'{CONFIG_CHEATCODE}'>() {{
                    {if_content}

                    {extra_statements}

                    return;
                }}

                {statements}
            }}
        "
    )
}
