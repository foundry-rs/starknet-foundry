use cairo_lang_macro::{quote, Diagnostic, Severity, TextSpan, Token, TokenStream, TokenTree};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::ast::{Condition, Expr, FunctionWithBody, Statement};
use cairo_lang_syntax::node::helpers::GetIdentifier;
use cairo_lang_syntax::node::with_db::SyntaxNodeWithDb;
use cairo_lang_syntax::node::TypedSyntaxNode;
use indoc::formatdoc;

pub fn higher_severity(a: Severity, b: Severity) -> Severity {
    match (a, b) {
        (Severity::Warning, Severity::Warning) => Severity::Warning,
        _ => Severity::Error,
    }
}
pub fn format_error_message(variants: &[Diagnostic]) -> String {
    let formatted_variants: Vec<String> = variants
        .iter()
        .map(|variant| format!("- variant: {}", variant.message))
        .collect();

    formatdoc! {"
        All options failed
        {}
        Resolve at least one of them
    ", formatted_variants.join("\n")}
}

/// The `branch` macro is used to evaluate multiple expressions and return the first successful result.
/// If all expressions fail, it collects the error messages and returns a combined error.
///
/// This macro is used instead of a function because it can perform lazy evaluation and has better readability.
#[macro_export]
macro_rules! branch {
    ($($result:expr_2021),+) => {{
        let mut messages = Vec::new();
        let mut result = None;

        $(
            if result.is_none() {
                match $result {
                    Ok(val) => {
                        result = Some(val);
                    },
                    Err(err) => {
                        messages.push(err);
                    },
                }
            }
        )+

        if let Some(result) = result {
            Ok(result)
        } else {
            Err(Diagnostic {
                message: $crate::utils::format_error_message(&messages),
                severity: messages.into_iter().fold(Severity::Warning, |acc, diagnostic| $crate::utils::higher_severity(acc, diagnostic.severity)),
                span: None,
            })
        }
    }};
}

pub fn create_single_token(content: impl AsRef<str>) -> TokenTree {
    TokenTree::Ident(Token::new(content, TextSpan::call_site()))
}

pub trait SyntaxNodeUtils {
    fn to_token_stream(&self, db: &SimpleParserDatabase) -> TokenStream;
}

impl<T: TypedSyntaxNode> SyntaxNodeUtils for T {
    fn to_token_stream(&self, db: &SimpleParserDatabase) -> TokenStream {
        let syntax = self.as_syntax_node();
        let syntax = SyntaxNodeWithDb::new(&syntax, db);
        quote!(#syntax)
    }
}

// Gets test statements and content of `if` statement that checks if function is run in config mode
pub fn get_statements(
    db: &SimpleParserDatabase,
    func: &FunctionWithBody,
) -> (TokenStream, TokenStream) {
    let statements = func.body(db).statements(db).elements(db);

    let if_content = statements.first().and_then(|stmt| {
        // first statement is `if`
        let Statement::Expr(expr) = stmt else {
            return None;
        };
        let Expr::If(if_expr) = expr.expr(db) else {
            return None;
        };
        // its condition is function call
        let Condition::Expr(expr) = if_expr.condition(db) else {
            return None;
        };
        let Expr::FunctionCall(expr) = expr.expr(db) else {
            return None;
        };

        // this function is named "snforge_std::_internals::is_config_run"
        let segments = expr.path(db).segments(db).elements(db);

        let [snforge_std, cheatcode, is_config_run] = segments.as_slice() else {
            return None;
        };

        if snforge_std.identifier(db) != "snforge_std"
            || cheatcode.identifier(db) != "_internals"
            || is_config_run.identifier(db) != "is_config_run"
        {
            return None;
        }

        let statements = if_expr.if_block(db).statements(db).elements(db);

        // omit last one (`return;`) as it have to be inserted after all new statements
        Some(
            statements[..statements.len() - 1]
                .iter()
                .map(|stmt| stmt.to_token_stream(db))
                .fold(TokenStream::empty(), |mut acc, token| {
                    acc.extend(token);
                    acc
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
    .map(|stmt| stmt.to_token_stream(db))
    .fold(TokenStream::empty(), |mut acc, token| {
        acc.extend(token);
        acc
    });

    (statements, if_content.unwrap_or_else(TokenStream::empty))
}

#[macro_export]
macro_rules! format_ident {
    ($name:literal $(,$formats:expr),*) => {
        {
            use cairo_lang_macro::{TextSpan, Token, TokenTree};

            let content = format!($name,$($formats),*);
            TokenTree::Ident(Token::new(content, TextSpan::call_site()))
        }
    };
}
