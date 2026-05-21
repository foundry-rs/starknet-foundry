use crate::utils::{assert_diagnostics, empty_function, format_output};
use cairo_lang_macro::{Diagnostic, quote};
use indoc::formatdoc;
use snforge_scarb_plugin::attributes::available_gas::available_gas;

#[test]
fn works_with_empty() {
    let args = quote!(());

    let result = available_gas(args, empty_function());

    insta::assert_snapshot!(format_output(&result));

    assert_diagnostics(
        &result,
        &[Diagnostic::warn(
            "#[available_gas] used with empty argument list. Either remove () or specify some arguments",
        )],
    );
}

#[test]
fn fails_with_non_number_literal() {
    let args = quote!((l2_gas: "123"));

    let result = available_gas(args, empty_function());

    assert_diagnostics(
        &result,
        &[Diagnostic::error(formatdoc!(
            "#[available_gas] <l2_gas> should be number literal"
        ))],
    );
}

#[test]
fn work_with_number_some_set() {
    let args = quote!((l1_gas: 123));

    let result = available_gas(args, empty_function());

    assert_diagnostics(&result, &[]);

    insta::assert_snapshot!(format_output(&result));
}

#[test]
fn work_with_number_all_set() {
    let args = quote!((l1_gas: 1, l1_data_gas: 2, l2_gas: 3));

    let result = available_gas(args, empty_function());

    assert_diagnostics(&result, &[]);

    insta::assert_snapshot!(format_output(&result));
}

#[test]
fn is_used_once() {
    let args = quote!((l2_gas: 1, l2_gas: 3));

    let result = available_gas(args, empty_function());

    assert_diagnostics(
        &result,
        &[Diagnostic::error(formatdoc!(
            "<l2_gas> argument was specified 2 times, expected to be used only once"
        ))],
    );
}

#[test]
fn does_not_work_with_unnamed_arg() {
    let args = quote!((3));

    let result = available_gas(args, empty_function());

    assert_diagnostics(
        &result,
        &[Diagnostic::error(formatdoc!(
            "#[available_gas] can be used with named arguments only [possible values: l1_gas, l1_data_gas, l2_gas]. invalid arguments found: 3"
        ))],
    );
}

#[test]
fn fails_with_unexpected_args() {
    let args = quote!((sth: 1000));
    let result = available_gas(args, empty_function());

    assert_diagnostics(
        &result,
        &[Diagnostic::error(formatdoc!(
            "#[available_gas] unexpected argument(s): <sth>"
        ))],
    );
}

#[test]
fn handles_number_overflow_l1() {
    let args = quote!((l1_gas: 18446744073709551616));

    let result = available_gas(args, empty_function());

    assert_diagnostics(
        &result,
        &[Diagnostic::error(formatdoc!(
            "#[available_gas] l1_gas it too large (max permissible value is 18446744073709551615)"
        ))],
    );
}

#[test]
fn handles_number_overflow_l1_data() {
    let args = quote!((l1_data_gas: 18446744073709551616));

    let result = available_gas(args, empty_function());

    assert_diagnostics(
        &result,
        &[Diagnostic::error(formatdoc!(
            "#[available_gas] l1_data_gas it too large (max permissible value is 18446744073709551615)"
        ))],
    );
}

#[test]
fn handles_number_overflow_l2() {
    let args = quote!((l2_gas: 18446744073709551616));

    let result = available_gas(args, empty_function());

    assert_diagnostics(
        &result,
        &[Diagnostic::error(formatdoc!(
            "#[available_gas] l2_gas it too large (max permissible value is 18446744073709551615)"
        ))],
    );
}

#[test]
fn max_permissible_value() {
    let args = quote!((l2_gas: 18446744073709551615));

    let result = available_gas(args, empty_function());

    assert_diagnostics(&result, &[]);

    insta::assert_snapshot!(format_output(&result));
}
