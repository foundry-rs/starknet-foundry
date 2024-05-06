use crate::{
    args::Arguments,
    asserts::assert_is_used_once,
    attributes::AttributeCollector,
    parse::{parse, parse_args},
    MacroResult,
};
use cairo_lang_macro::TokenStream;
use cairo_lang_syntax::node::{
    ast::{Condition, Expr, FunctionWithBody, GenericArg, GenericArgValue, Statement},
    db::SyntaxGroup,
    helpers::{GetIdentifier, PathSegmentEx},
    TypedSyntaxNode,
};
use cairo_lang_utils::Upcast;
use indoc::formatdoc;

pub trait ExtendWithConfig {
    fn extend_with_config_cheatcodes(args: TokenStream, item: TokenStream) -> MacroResult;
}

impl<T> ExtendWithConfig for T
where
    T: AttributeCollector,
{
    fn extend_with_config_cheatcodes(args: TokenStream, item: TokenStream) -> MacroResult {
        let item = item.to_string();
        let (db, func) = parse::<Self>(&item)?;

        let db = db.upcast();

        assert_is_used_once::<Self>(db, &func)?;

        let (args_db, args) = parse_args::<Self>(&args.to_string())?;

        let (args, empty_args_list_warn) = Arguments::new::<Self>(args_db.upcast(), args);

        let value = Self::args_into_body(args_db.upcast(), args).map_err(|err| {
            if let Some(empty_args_list_warn) = empty_args_list_warn {
                err.warn(empty_args_list_warn)
            } else {
                err
            }
        })?;

        let cheatcode_name = Self::CHEATCODE_NAME;

        let config_cheatcode = formatdoc!(
            r#"
                let mut data = array![];

                {value}
                .serialize(ref data);

                cheatcode::<'{cheatcode_name}'>(data);
            "#
        );

        Ok(TokenStream::new(append_config_statements(
            db,
            &func,
            &config_cheatcode,
        )))
    }
}

const CONFIG_CHEATCODE: &str = "was_configuration_set";

macro_rules! propagate {
    ($pattern:pat = $expression:expr) => {
        let $pattern = $expression else { return None };
    };
}

fn append_config_statements(
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
        propagate!(Statement::Expr(expr) = stmt);
        propagate!(Expr::If(if_expr) = expr.expr(db));
        // it's condition is function call
        propagate!(Condition::Expr(expr) = if_expr.condition(db));
        propagate!(Expr::FunctionCall(expr) = expr.expr(db));

        // this function is named "cheatcode"
        let segments = expr.path(db).elements(db);

        propagate!([segment] = segments.as_slice());

        if segment.identifier(db) != "cheatcode" {
            return None;
        }

        // it has single, unnamed generic argument
        let generics = segment.generic_args(db)?;

        propagate!([GenericArg::Unnamed(cheatcode)] = generics.as_slice());
        propagate!(GenericArgValue::Expr(expr) = cheatcode.value(db));
        // of type short string
        propagate!(Expr::ShortString(str) = expr.expr(db));

        // equal to configuration cheatcode
        if str.string_value(db)? == CONFIG_CHEATCODE {
            Some(if_expr.if_block(db).as_syntax_node().get_text(db))
        } else {
            None
        }
    });

    // there was already config check, ommit it and collect remaining statements
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
                if cheatcode::<'{CONFIG_CHEATCODE}'>() {{
                    {if_content}

                    {extra_statements}
                }}

                {statements}
            }}
        "
    )
}
