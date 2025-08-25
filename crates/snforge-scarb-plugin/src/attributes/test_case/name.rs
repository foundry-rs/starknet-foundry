use crate::args::Arguments;
use crate::args::unnamed::UnnamedArgs;
use crate::attributes::ErrorExt;
use crate::attributes::test_case::TestCaseCollector;
use cairo_lang_macro::Diagnostics;
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::TypedSyntaxNode;
use cairo_lang_syntax::node::ast::Expr;
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

static RE_SANITIZE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[^a-zA-Z0-9]+").expect("Failed to create regex"));

fn sanitize_str(raw: &str) -> String {
    let s = RE_SANITIZE
        .replace_all(raw, "_")
        .to_lowercase()
        .trim_matches('_')
        .to_string();
    if s.is_empty() { "_empty".into() } else { s }
}

fn expr_label(expr: &Expr, db: &SimpleParserDatabase) -> String {
    sanitize_str(&expr.as_syntax_node().get_text(db))
}

fn case_fn_name(
    func_name: &str,
    unnamed_args: &UnnamedArgs,
    args_db: &SimpleParserDatabase,
) -> String {
    let parts = unnamed_args
        .iter()
        .map(|(_, expr)| expr_label(expr, args_db))
        .collect::<Vec<_>>();

    let suffix = if parts.is_empty() {
        "case".to_string()
    } else {
        parts.join("_")
    };

    format!("{func_name}_{suffix}")
}

#[must_use]
fn vec_pairs_to_map<T>(pairs: Vec<(usize, T)>) -> HashMap<usize, T> {
    pairs.into_iter().collect()
}

pub fn get_test_case_name(
    func_name: &str,
    arguments: &Arguments,
    args_db: &SimpleParserDatabase,
) -> Result<String, Diagnostics> {
    let named_args = arguments.named();
    let test_case_name = named_args
        .as_once_optional("name")?
        .map(|arg| {
            let expr = arg;
            match expr {
                Expr::String(_) | Expr::ShortString(_) => {
                    Ok(expr.as_syntax_node().get_text(args_db).to_string())
                }
                _ => Err(TestCaseCollector::error(
                    "The 'name' argument must be a string literal.",
                )),
            }
        })
        .transpose()?;

    if let Some(ref name) = test_case_name {
        let sanitized = sanitize_str(name);
        let test_case_name = format!("{func_name}_{sanitized}");
        Ok(test_case_name)
    } else {
        let unnamed_args = arguments.unnamed();
        let name = case_fn_name(
            func_name,
            &UnnamedArgs::new(&vec_pairs_to_map(
                unnamed_args
                    .clone()
                    .into_iter()
                    .map(|(i, expr)| (i, expr.clone()))
                    .collect::<Vec<_>>(),
            )),
            args_db,
        );
        Ok(name)
    }
}
