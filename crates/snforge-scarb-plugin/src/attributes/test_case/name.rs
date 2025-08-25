use crate::args::Arguments;
use crate::args::unnamed::UnnamedArgs;
use crate::attributes::ErrorExt;
use crate::attributes::test_case::TestCaseCollector;
use cairo_lang_macro::Diagnostics;
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::TypedSyntaxNode;
use cairo_lang_syntax::node::ast::Expr;
use regex::Regex;
use std::sync::LazyLock;

static RE_SANITIZE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[^a-zA-Z0-9]+").expect("Failed to create regex"));

fn sanitize_ident(raw: &str) -> String {
    let s = RE_SANITIZE
        .replace_all(raw, "_")
        .to_lowercase()
        .trim_matches('_')
        .to_string();
    if s.is_empty() { "_empty".into() } else { s }
}

fn generate_case_name(
    func_name: &str,
    unnamed_args: &UnnamedArgs,
    args_db: &SimpleParserDatabase,
) -> String {
    let parts = unnamed_args
        .iter()
        .map(|(_, expr)| {
            let expr_text = &expr.as_syntax_node().get_text(args_db);
            sanitize_ident(expr_text)
        })
        .collect::<Vec<_>>();

    let suffix = if parts.is_empty() {
        "case".to_string()
    } else {
        parts.join("_")
    };

    format!("{func_name}_{suffix}")
}

pub fn resolve_test_case_name(
    func_name: &str,
    arguments: &Arguments,
    args_db: &SimpleParserDatabase,
) -> Result<String, Diagnostics> {
    let suffix = if let Some(expr) = arguments.named().as_once_optional("name")? {
        match expr {
            Expr::String(_) | Expr::ShortString(_) => {
                sanitize_ident(&expr.as_syntax_node().get_text(args_db))
            }
            _ => {
                return Err(Diagnostics::from(TestCaseCollector::error(
                    "The 'name' argument must be a string literal.",
                )));
            }
        }
    } else {
        generate_case_name(
            func_name,
            &UnnamedArgs::new(
                &arguments
                    .unnamed()
                    .iter()
                    .map(|(i, expr)| (*i, (*expr).clone()))
                    .collect::<Vec<_>>()
                    .into_iter()
                    .collect(),
            ),
            args_db,
        )
    };

    Ok(format!("{func_name}_{suffix}"))
}
