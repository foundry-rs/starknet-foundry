use crate::utils::{assert_diagnostics, empty_function, format_output};
use cairo_lang_macro::{Diagnostic, TokenStream, quote};
use snforge_scarb_plugin::attributes::test_case::test_case;

pub fn function_with_params() -> TokenStream {
    quote!(
        fn test_add(x: i128, y: i128, expected: i128) {}
    )
}

#[test]
fn works_with_args() {
    let args = quote!((1, 2, 3));

    let result = test_case(args, function_with_params());

    assert_diagnostics(&result, &[]);

    insta::assert_snapshot!(format_output(&result));
}

#[test]
fn works_with_name_and_args() {
    let args = quote!((name: "one_and_two", 1, 2, 3));

    let result = test_case(args, function_with_params());

    assert_diagnostics(&result, &[]);

    insta::assert_snapshot!(format_output(&result));
}

#[test]
fn invalid_args_number() {
    let args = quote!((1, 2));

    let result = test_case(args, function_with_params());

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[test_case] Expected 3 arguments, but got 2",
        )],
    );
}

#[test]
fn name_passed_multiple_times() {
    let args = quote!((name: "a", name: "b", 1, 2, 3));

    let result = test_case(args, function_with_params());

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "<name> argument was specified 2 times, expected to be used only once",
        )],
    );
}

#[test]
fn function_without_params() {
    let args = quote!((1, 2, 3));

    let result = test_case(args, empty_function());

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[test_case] The function must have at least one parameter to use #[test_case] attribute",
        )],
    );
}

#[test]
fn fails_with_unexpected_named_args() {
    let args = quote!((name: "test", tomato: 123, 1, 2, 3));

    let result = test_case(args, function_with_params());

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[test_case] unexpected argument(s): <tomato>",
        )],
    );
}
