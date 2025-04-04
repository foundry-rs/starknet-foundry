use anyhow::{Result, bail};
use cairo_lang_diagnostics::DiagnosticsBuilder;
use cairo_lang_filesystem::ids::{FileKind, FileLongId, VirtualFile};
use cairo_lang_parser::parser::Parser;
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::Terminal;
use cairo_lang_syntax::node::ast::{
    ArgClause, ArgList, Expr, ExprInlineMacro, Modifier, PathSegment, PathSegment::Simple,
    WrappedArgList,
};
use cairo_lang_utils::Intern;
use itertools::Itertools;

pub fn parse_expression(source: &str, db: &SimpleParserDatabase) -> Result<Expr> {
    let file = FileLongId::Virtual(VirtualFile {
        parent: None,
        name: "parser_input".into(),
        content: source.to_string().into(),
        code_mappings: [].into(),
        kind: FileKind::Expr,
    })
    .intern(db);

    let mut diagnostics = DiagnosticsBuilder::default();
    let expression = Parser::parse_file_expr(db, &mut diagnostics, file, source);
    let diagnostics = diagnostics.build();

    if diagnostics.check_error_free().is_err() {
        bail!(
            "Invalid Cairo expression found in input calldata \"{}\":\n{}",
            source,
            diagnostics.format(db)
        )
    }

    Ok(expression)
}

fn modifier_syntax_token(item: &Modifier) -> &'static str {
    match item {
        Modifier::Ref(_) => "ref",
        Modifier::Mut(_) => "mut",
    }
}

pub fn parse_argument_list(arguments: &ArgList, db: &SimpleParserDatabase) -> Result<Vec<Expr>> {
    let arguments = arguments.elements(db);

    if let Some(modifiers) = arguments
        .iter()
        .map(|arg| arg.modifiers(db).elements(db))
        .find(|mod_list| !mod_list.is_empty())
    {
        let modifiers = modifiers.iter().map(modifier_syntax_token).collect_vec();

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

    arguments
        .iter()
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
        .elements(db)
        .iter()
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
    }

    match invocation.arguments(db) {
        WrappedArgList::BracketedArgList(args) => {
            let arglist = args.arguments(db);
            parse_argument_list(&arglist, db)
        }
        WrappedArgList::ParenthesizedArgList(_) | WrappedArgList::BracedArgList(_) => {
            bail!("`array` macro supports only square brackets: array![]")
        }
        WrappedArgList::Missing(_) => unreachable!(
            "If any type of parentheses is missing, then diagnostics have been reported and whole flow should have already been terminated."
        ),
    }
}
