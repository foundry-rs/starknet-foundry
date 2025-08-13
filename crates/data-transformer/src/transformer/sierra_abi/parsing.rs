use crate::transformer::split_expressions;
use anyhow::{Result, bail};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::ast::WrappedTokenTree;
use cairo_lang_syntax::node::ast::{
    ArgClause, ArgList, Expr, ExprInlineMacro, Modifier, PathSegment, PathSegment::Simple,
};
use cairo_lang_syntax::node::{Terminal, TypedSyntaxNode};
use itertools::Itertools;

fn modifier_syntax_token(item: &Modifier) -> &'static str {
    match item {
        Modifier::Ref(_) => "ref",
        Modifier::Mut(_) => "mut",
    }
}

pub fn parse_argument_list(arguments: &ArgList, db: &SimpleParserDatabase) -> Result<Vec<Expr>> {
    let args_lists = arguments;
    let arguments = arguments.elements(db);

    if let Some(modifiers) = arguments
        .map(|arg| arg.modifiers(db).elements(db))
        .find(|mod_list| mod_list.len() != 0)
    {
        let modifiers = modifiers
            .map(|modifier| modifier_syntax_token(&modifier))
            .collect_vec();

        match &modifiers[..] {
            [] => unreachable!(),
            [single] => bail!(r#""{single}" modifier is not allowed"#),
            [multiple @ .., last] => {
                bail!(
                    "{} and {} modifiers are not allowed",
                    multiple.iter().join(", "),
                    last
                )
            }
        }
    }

    args_lists
        .elements(db)
        .map(|arg| match arg.arg_clause(db) {
            ArgClause::Unnamed(expr) => Ok(expr.value(db)),
            ArgClause::Named(_) => {
                bail!("Named arguments are not allowed")
            }
            ArgClause::FieldInitShorthand(_) => {
                bail!("Field init shorthands are not allowed")
            }
        })
        .collect::<Result<Vec<Expr>>>()
}

pub fn parse_inline_macro(
    invocation: &ExprInlineMacro,
    db: &SimpleParserDatabase,
) -> Result<Vec<Expr>> {
    match invocation
        .path(db)
        .segments(db)
        .elements(db)
        .last()
        .expect("Macro must have a name")
    {
        Simple(simple) => {
            let macro_name = simple.ident(db).text(db);
            if macro_name != "array" {
                bail!(
                    r#"Invalid macro name, expected "array![]", got "{}""#,
                    macro_name
                )
            }
        }
        PathSegment::WithGenericArgs(_) => {
            bail!("Invalid path specified: generic args in array![] macro not supported")
        }
        PathSegment::Missing(_segment) => {
            bail!("Path segment missing")
        }
    }

    match invocation.arguments(db).subtree(db) {
        WrappedTokenTree::Bracketed(token_tree) => {
            let node_text = token_tree
                .tokens(db)
                .elements(db)
                .map(|token| token.as_syntax_node().get_text(db).to_string())
                .collect::<String>();
            split_expressions(&node_text, db)
        }
        WrappedTokenTree::Parenthesized(_) | WrappedTokenTree::Braced(_) => {
            bail!("`array` macro supports only square brackets: array![]")
        }
        WrappedTokenTree::Missing(_) => unreachable!(
            "If any type of parentheses is missing, then diagnostics have been reported and whole flow should have already been terminated."
        ),
    }
}
