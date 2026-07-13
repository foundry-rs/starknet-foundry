use crate::utils::create_single_token;
use cairo_lang_macro::{Diagnostic, ProcMacroResult, TextSpan, TokenStream, TokenTree, quote};
use regex::Regex;
use std::sync::LazyLock;

static CONTRACT_PATH_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(::)?[A-Za-z_][A-Za-z0-9_]*(::[A-Za-z_][A-Za-z0-9_]*)*$")
        .expect("contract path regex should be valid")
});

#[must_use]
pub fn declare(args: &TokenStream) -> ProcMacroResult {
    match expand(args) {
        Ok(token_stream) => ProcMacroResult::new(token_stream),
        Err(diagnostic) => {
            ProcMacroResult::new(TokenStream::empty()).with_diagnostics(vec![diagnostic].into())
        }
    }
}

fn expand(args: &TokenStream) -> Result<TokenStream, Diagnostic> {
    let raw_path = args.to_string();
    let Some(contract_path) = normalize_path(&raw_path) else {
        return Err(Diagnostic::span_error(
            args_span(args),
            "`declare!` expects either a contract name (e.g. `MyContract`), an absolute module tree path (e.g. `my_package::module::MyContract`) or a partial module tree path (e.g. `module::MyContract`)",
        ));
    };

    let contract_path_literal =
        TokenStream::new(vec![create_single_token(format!(r#""{contract_path}""#))]);
    let path_tokens = TokenStream::new(vec![create_single_token(&contract_path)]);

    Ok(quote! {{
        snforge_std::_internals::assert_path_type::<#path_tokens::ContractState>();
        snforge_std::declare(#contract_path_literal)
    }})
}

fn normalize_path(raw_path: &str) -> Option<String> {
    let normalized = normalize_path_separators(strip_macro_arg_parentheses(raw_path.trim())?)?;

    is_valid_contract_path(&normalized).then(|| {
        normalized
            .strip_prefix("::")
            .unwrap_or(&normalized)
            .to_string()
    })
}

/// Returns a span covering all macro argument tokens for diagnostics.
/// Falls back to the call site when the macro is invoked without arguments.
fn args_span(args: &TokenStream) -> TextSpan {
    let mut spans = args.tokens.iter().map(|token| match token {
        TokenTree::Ident(token) => &token.span,
    });
    let Some(first) = spans.next() else {
        return TextSpan::call_site();
    };

    let (mut start, mut end) = (first.start, first.end);
    for span in spans {
        start = start.min(span.start);
        end = end.max(span.end);
    }

    TextSpan::new(start, end)
}

fn strip_macro_arg_parentheses(path: &str) -> Option<&str> {
    path.strip_prefix('(')?.strip_suffix(')').map(str::trim)
}

/// Normalizes whitespace around `::` separators in a path, but rejects whitespace that
/// would split a path segment into multiple identifiers.
/// For example, `my_package :: hello_starknet :: HelloStarknet` is normalized
/// to `my_package::hello_starknet::HelloStarknet`, but `Hello Starknet` is rejected.
fn normalize_path_separators(path: &str) -> Option<String> {
    let mut normalized = String::with_capacity(path.len());
    let mut chars = path.char_indices().peekable();

    while let Some((_, c)) = chars.next() {
        if c.is_whitespace() {
            let previous_allows_whitespace = normalized.ends_with(':');
            let next_non_whitespace = chars
                .clone()
                .find(|(_, next)| !next.is_whitespace())
                .map(|(_, next)| next);

            if previous_allows_whitespace || next_non_whitespace == Some(':') {
                continue;
            }

            return None;
        }

        normalized.push(c);
    }

    Some(normalized)
}

/// Validates that the given path is a valid contract path, which can be either:
/// - a contract name (e.g. `MyContract`)
/// - an absolute module tree path (e.g. `my_package::module::MyContract`)
/// - a partial module tree path (e.g. `module::MyContract`).
///
/// Expects whitespace around `::` separators to be normalized away before validation.
/// Rejects empty paths, empty segments, segments starting with digits, non-identifier
/// characters and whitespace inside path segments.
fn is_valid_contract_path(path: &str) -> bool {
    CONTRACT_PATH_REGEX.is_match(path)
}

#[cfg(test)]
mod tests {
    use super::{is_valid_contract_path, normalize_path, strip_macro_arg_parentheses};
    use test_case::test_case;

    #[test_case("HelloStarknet"; "contract name")]
    #[test_case("my_package::hello_starknet::HelloStarknet"; "full module path")]
    #[test_case("::my_package::hello_starknet::HelloStarknet"; "full module path with leading colons")]
    #[test_case("alias::HelloStarknet"; "partial module path")]
    #[test_case("::alias::HelloStarknet"; "partial module path with leading colons")]
    fn valid_contract_path(path: &str) {
        assert!(is_valid_contract_path(path));
    }

    #[test_case("\"HelloStarknet\""; "non-path argument")]
    #[test_case("my-package::HelloStarknet"; "invalid module path")]
    #[test_case("1_Contract"; "identifier starting with digit")]
    #[test_case("hello_starknet::1_Contract"; "path segment starting with digit")]
    #[test_case("hello_starknet::"; "trailing empty segment")]
    #[test_case("hello_starknet::::HelloStarknet"; "empty middle segment")]
    #[test_case("::::HelloStarknet"; "multiple leading separators")]
    #[test_case(""; "empty string")]
    fn invalid_contract_path(path: &str) {
        assert!(!is_valid_contract_path(path));
    }

    #[test]
    fn normalizes_whitespace_around_path_separators() {
        assert_eq!(
            normalize_path("(my_package :: hello_starknet :: HelloStarknet)"),
            Some("my_package::hello_starknet::HelloStarknet".to_string())
        );
    }

    #[test]
    fn normalizes_leading_colons() {
        assert_eq!(
            normalize_path("(::my_package::hello_starknet::HelloStarknet)"),
            Some("my_package::hello_starknet::HelloStarknet".to_string())
        );
    }

    #[test]
    fn rejects_path_without_macro_arg_parentheses() {
        assert!(normalize_path("HelloStarknet").is_none());
    }

    #[test_case("Hello Starknet"; "two identifiers separated by whitespace")]
    #[test_case("my_package::hello_starknet::Hello Starknet"; "path segment with whitespace")]
    fn rejects_whitespace_between_identifiers(path: &str) {
        assert!(normalize_path(&format!("({path})")).is_none());
    }

    #[test]
    fn strips_macro_arg_parentheses() {
        assert_eq!(
            strip_macro_arg_parentheses("(HelloStarknet)"),
            Some("HelloStarknet")
        );
    }
}
