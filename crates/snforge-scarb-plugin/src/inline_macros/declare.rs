use crate::utils::create_single_token;
use cairo_lang_macro::{Diagnostic, ProcMacroResult, TextSpan, TokenStream, quote};

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
            TextSpan::call_site(),
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
    let normalized = normalize_path_separators(strip_macro_arg_delimiters(raw_path.trim()))?;

    is_valid_contract_path(&normalized).then_some(normalized)
}

fn strip_macro_arg_delimiters(path: &str) -> &str {
    if has_wrapping_delimiters(path) {
        path[1..path.len() - 1].trim()
    } else {
        path
    }
}

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

fn has_wrapping_delimiters(path: &str) -> bool {
    matches!(
        (path.as_bytes().first(), path.as_bytes().last()),
        (Some(b'('), Some(b')')) | (Some(b'['), Some(b']')) | (Some(b'{'), Some(b'}'))
    )
}

fn is_valid_contract_path(path: &str) -> bool {
    for part in path.split("::") {
        let mut chars = part.chars();
        let Some(first_char) = chars.next() else {
            return false;
        };
        if !(first_char.is_ascii_alphabetic() || first_char == '_')
            || !chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::{is_valid_contract_path, normalize_path, strip_macro_arg_delimiters};
    use test_case::test_case;

    #[test_case("HelloStarknet"; "contract name")]
    #[test_case("my_package::hello_starknet::HelloStarknet"; "full module path")]
    #[test_case("alias::HelloStarknet"; "partial module path")]
    fn valid_contract_path(path: &str) {
        assert!(is_valid_contract_path(path));
    }

    #[test_case("\"HelloStarknet\""; "non-path argument")]
    #[test_case("my-package::HelloStarknet"; "invalid module path")]
    #[test_case("1_Contract"; "identifier starting with digit")]
    #[test_case("hello_starknet::1_Contract"; "path segment starting with digit")]
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

    #[test_case("Hello Starknet"; "two identifiers separated by whitespace")]
    #[test_case("my_package::hello_starknet::Hello Starknet"; "path segment with whitespace")]
    fn rejects_whitespace_between_identifiers(path: &str) {
        assert!(normalize_path(path).is_none());
    }

    #[test_case("((HelloStarknet))"; "parentheses")]
    #[test_case("([HelloStarknet])"; "brackets")]
    #[test_case("({HelloStarknet})"; "braces")]
    fn rejects_wrapped_paths(path: &str) {
        assert!(normalize_path(path).is_none());
    }

    #[test]
    fn strips_only_single_macro_arg_delimiter_layer() {
        assert_eq!(
            strip_macro_arg_delimiters("((HelloStarknet))"),
            "(HelloStarknet)"
        );
    }
}
