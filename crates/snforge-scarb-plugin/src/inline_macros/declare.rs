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
            "`declare!` expects a contract module path like `HelloStarknet` or `my_package::my_module::MyContract`",
        ));
    }

    let contract_path_literal =
        TokenStream::new(vec![create_single_token(format!(r#""{contract_path}""#))]);
    let type_check_path = type_check_path(&contract_path);
    let path_tokens = TokenStream::new(vec![create_single_token(&type_check_path)]);

    Ok(quote! {{
        snforge_std::_internals::assert_path_type::<#path_tokens::ContractState>();
        snforge_std::declare(#contract_path_literal)
    }})
}

fn normalize_path(raw_path: &str) -> String {
    let normalized: String = raw_path.chars().filter(|c| !c.is_whitespace()).collect();

    trim_wrapping_delimiters(&normalized).to_string()
}

fn trim_wrapping_delimiters(path: &str) -> &str {
    match (
        path.as_bytes().first().copied(),
        path.as_bytes().last().copied(),
    ) {
        (Some(b'('), Some(b')')) | (Some(b'['), Some(b']')) | (Some(b'{'), Some(b'}')) => {
            &path[1..path.len() - 1]
        }
        _ => path,
    }
}

fn is_valid_contract_path(path: &str) -> bool {
    let mut parts = path.split("::");
    let mut count = 0usize;

    for part in &mut parts {
        if part.is_empty() || !part.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return false;
        }
        count += 1;
    }

    count >= 1
}

fn type_check_path(path: &str) -> String {
    let mut segments = path.split("::");
    let Some(first_segment) = segments.next() else {
        return path.to_string();
    };
    let Some(second_segment) = segments.next() else {
        return first_segment.to_string();
    };

    if segments.next().is_none() {
        second_segment.to_string()
    } else {
        path.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        is_valid_contract_path, normalize_path, trim_wrapping_delimiters, type_check_path,
    };

    #[test]
    fn valid_contract_path() {
        assert!(is_valid_contract_path(
            "my_package::hello_starknet::HelloStarknet"
        ));
        assert!(is_valid_contract_path("HelloStarknet"));
        assert!(is_valid_contract_path("alias::HelloStarknet"));
    }

    #[test]
    fn invalid_contract_path() {
        assert!(!is_valid_contract_path("\"HelloStarknet\""));
        assert!(!is_valid_contract_path("my-package::HelloStarknet"));
        assert!(!is_valid_contract_path(""));
    }

    #[test]
    fn normalizes_whitespace() {
        assert_eq!(
            normalize_path("my_package :: hello_starknet :: HelloStarknet"),
            "my_package::hello_starknet::HelloStarknet"
        );
    }

    #[test]
    fn trims_wrapping_parentheses() {
        assert_eq!(
            trim_wrapping_delimiters("(my_package::hello_starknet::HelloStarknet)"),
            "my_package::hello_starknet::HelloStarknet"
        );
    }

    #[test]
    fn uses_contract_name_for_two_segment_type_check_path() {
        assert_eq!(
            type_check_path("hello_starknet::HelloStarknet"),
            "HelloStarknet"
        );
    }

    #[test]
    fn uses_full_path_for_longer_type_check_path() {
        assert_eq!(
            type_check_path("my_package::hello_starknet::HelloStarknet"),
            "my_package::hello_starknet::HelloStarknet"
        );
    }
}
