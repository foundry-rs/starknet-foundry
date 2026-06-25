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
    let contract_path = normalize_path(&raw_path);

    if !is_valid_contract_path(&contract_path) {
        return Err(Diagnostic::span_error(
            TextSpan::call_site(),
            "`declare!` expects either a contract name (e.g. `MyContract`), an absolute module tree path (e.g. `my_package::module::MyContract`) or a partial module tree path (e.g. `module::MyContract`)",
        ));
    }

    let contract_path_literal =
        TokenStream::new(vec![create_single_token(format!(r#""{contract_path}""#))]);
    let path_tokens = TokenStream::new(vec![create_single_token(&contract_path)]);

    Ok(quote! {{
        snforge_std::_internals::assert_path_type::<#path_tokens::ContractState>();
        snforge_std::declare(#contract_path_literal)
    }})
}

fn normalize_path(raw_path: &str) -> String {
    let mut normalized: String = raw_path.chars().filter(|c| !c.is_whitespace()).collect();

    while has_wrapping_delimiters(&normalized) {
        normalized.pop();
        normalized.remove(0);
    }

    normalized
}

fn trim_wrapping_delimiters(path: &str) -> &str {
    let mut trimmed = path;

    while has_wrapping_delimiters(trimmed) {
        trimmed = &trimmed[1..trimmed.len() - 1];
    }

    trimmed
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
    use super::{is_valid_contract_path, normalize_path, trim_wrapping_delimiters};

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
    fn normalizes_whitespace_and_nested_delimiters() {
        assert_eq!(
            normalize_path("(( my_package :: hello_starknet :: HelloStarknet ))"),
            "my_package::hello_starknet::HelloStarknet"
        );
    }

    #[test]
    fn trims_nested_wrapping_parentheses() {
        assert_eq!(
            trim_wrapping_delimiters("((my_package::hello_starknet::HelloStarknet))"),
            "my_package::hello_starknet::HelloStarknet"
        );
    }
}
