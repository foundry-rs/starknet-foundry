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

fn sanitize_expr(expr: &Expr, db: &SimpleParserDatabase) -> String {
    let expr_text = &expr.as_syntax_node().get_text(db);
    let expr_sanitized = RE_SANITIZE
        .replace_all(expr_text, "_")
        .to_lowercase()
        .trim_matches('_')
        .to_string();

    if expr_sanitized.is_empty() {
        "_empty".into()
    } else {
        expr_sanitized
    }
}

fn generate_case_suffix(
    unnamed_args: &UnnamedArgs,
    db: &SimpleParserDatabase,
) -> Result<String, Diagnostics> {
    if unnamed_args.is_empty() {
        return Err(Diagnostics::from(TestCaseCollector::error(
            "At least one argument is required if 'name' is not provided.",
        )));
    }

    let exprs = unnamed_args
        .iter()
        .map(|(_, expr)| sanitize_expr(expr, db))
        .collect::<Vec<_>>();

    Ok(exprs.join("_"))
}

pub fn resolve_test_case_name(
    func_name: &str,
    arguments: &Arguments,
    db: &SimpleParserDatabase,
) -> Result<String, Diagnostics> {
    let named_args = arguments.named();
    let test_case_name = named_args.as_once_optional("name")?;

    let suffix = if let Some(expr) = test_case_name {
        sanitize_expr(expr, db)
    } else {
        generate_case_suffix(&arguments.unnamed(), db)?
    };

    Ok(format!("{func_name}_{suffix}"))
}
