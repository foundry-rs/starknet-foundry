use crate::{external_inputs::ExternalInput, utils::create_single_token};
use cairo_lang_macro::{Diagnostic, ProcMacroResult, TextSpan, TokenStream, quote};
use serde_json::Value;
use std::path::Path;

#[must_use]
pub fn declare_from_file(args: &TokenStream) -> ProcMacroResult {
    match expand(args) {
        Ok(token_stream) => ProcMacroResult::new(token_stream),
        Err(diagnostic) => {
            ProcMacroResult::new(TokenStream::empty()).with_diagnostics(vec![diagnostic].into())
        }
    }
}

fn expand(args: &TokenStream) -> Result<TokenStream, Diagnostic> {
    let raw_path = args.to_string();
    let Some(path) = parse_path_literal(&raw_path) else {
        return Err(Diagnostic::span_error(
            TextSpan::call_site(),
            "`declare_from_file!` expects a string literal path to a Sierra contract class JSON file",
        ));
    };

    validate_sierra_file(&path)?;

    let path_literal = TokenStream::new(vec![create_single_token(
        serde_json::to_string(&path).expect("path literal serialization should not fail"),
    )]);

    Ok(quote! {
        snforge_std::declare_from_file(#path_literal)
    })
}

fn parse_path_literal(raw_path: &str) -> Option<String> {
    let literal = raw_path.trim().strip_prefix('(')?.strip_suffix(')')?.trim();
    // `scarb fmt` adds a trailing comma for multiline macro calls.
    let literal = literal.strip_suffix(',').unwrap_or(literal).trim();

    serde_json::from_str(literal).ok()
}

fn validate_sierra_file(path: &str) -> Result<(), Diagnostic> {
    let path = Path::new(path);
    let sierra = ExternalInput::read_to_string(path).map_err(|error| {
        Diagnostic::span_error(
            TextSpan::call_site(),
            format!("Failed to read Sierra file at {}: {error}", path.display()),
        )
    })?;

    let sierra = serde_json::from_str::<Value>(&sierra).map_err(|error| {
        Diagnostic::span_error(
            TextSpan::call_site(),
            format!(
                "Failed to parse Sierra contract class JSON at {}: {error}",
                path.display()
            ),
        )
    })?;

    if is_sierra_contract_class_json(&sierra) {
        Ok(())
    } else {
        Err(Diagnostic::span_error(
            TextSpan::call_site(),
            format!(
                "File {} is not a valid Sierra contract class JSON",
                path.display()
            ),
        ))
    }
}

fn is_sierra_contract_class_json(value: &Value) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    let Some(entry_points_by_type) = object
        .get("entry_points_by_type")
        .and_then(Value::as_object)
    else {
        return false;
    };
    let Some(debug_info) = object
        .get("sierra_program_debug_info")
        .and_then(Value::as_object)
    else {
        return false;
    };

    object
        .get("contract_class_version")
        .is_some_and(Value::is_string)
        && object.get("sierra_program").is_some_and(Value::is_array)
        && object.get("abi").is_some_and(Value::is_array)
        && ["CONSTRUCTOR", "EXTERNAL", "L1_HANDLER"]
            .iter()
            .all(|key| entry_points_by_type.get(*key).is_some_and(Value::is_array))
        && ["type_names", "libfunc_names", "user_func_names"]
            .iter()
            .all(|key| debug_info.get(*key).is_some_and(Value::is_array))
}

#[cfg(test)]
mod tests {
    use super::parse_path_literal;

    #[test]
    fn parses_string_literal_path() {
        assert_eq!(
            parse_path_literal(r#"("target/dev/hello.contract_class.json")"#),
            Some("target/dev/hello.contract_class.json".to_string())
        );
    }

    #[test]
    fn parses_string_literal_path_with_trailing_comma() {
        assert_eq!(
            parse_path_literal(r#"("target/dev/hello.contract_class.json",)"#),
            Some("target/dev/hello.contract_class.json".to_string())
        );
    }

    #[test]
    fn rejects_non_string_literal_path() {
        assert!(parse_path_literal("(target::dev)").is_none());
    }

    #[test]
    fn rejects_path_without_macro_arg_parentheses() {
        assert!(parse_path_literal(r#""target/dev/hello.contract_class.json""#).is_none());
    }
}
