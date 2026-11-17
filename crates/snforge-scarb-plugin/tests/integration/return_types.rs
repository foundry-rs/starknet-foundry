use crate::multiple_attributes::get_function;
use crate::utils::{assert_diagnostics, format_output};
use cairo_lang_macro::{quote, TokenStream};
use snforge_scarb_plugin::attributes::available_gas::available_gas;
use snforge_scarb_plugin::attributes::extract_config::extract_config;
use snforge_scarb_plugin::attributes::fork::fork;
use snforge_scarb_plugin::attributes::fuzzer::wrapper::fuzzer_wrapper;
use snforge_scarb_plugin::attributes::test::test;

/// `Result<NonZero<u8>, u8>` stands in for any type without a `Default` impl.
fn returning_function() -> TokenStream {
    quote!(
        fn returning_fn() -> Result<NonZero<u8>, u8> {
            Result::Ok(1)
        }
    )
}

fn assert_constructs_no_default(output: &str) {
    assert!(
        !output.contains("Default::default()"),
        "generated code constructs a default value of the test's return type:\n{output}"
    );
}

#[test]
fn test_leaves_returning_function_untouched() {
    let result = test(TokenStream::empty(), returning_function());
    assert_diagnostics(&result, &[]);
    assert_constructs_no_default(&format_output(&result));

    insta::with_settings!({ snapshot_suffix => "after_test" }, {
        insta::assert_snapshot!(format_output(&result));
    });

    // `#[__snforge_extract_config]` is consumed when it expands, so drop it like the real chain does
    let item = get_function(&result.token_stream, "returning_fn", true);
    let result = extract_config(TokenStream::empty(), item);
    assert_diagnostics(&result, &[]);
    assert_constructs_no_default(&format_output(&result));

    insta::with_settings!({ snapshot_suffix => "after_extract_config" }, {
        insta::assert_snapshot!(format_output(&result));
    });
}

#[test]
fn config_attribute_statements_are_lifted_out() {
    let result = available_gas(quote!((l2_gas: 999)), returning_function());
    assert_diagnostics(&result, &[]);

    // `#[__snforge_extract_config]` is consumed when it expands, so drop it like the real chain does
    let item = get_function(&result.token_stream, "returning_fn", true);
    let result = extract_config(TokenStream::empty(), item);
    assert_diagnostics(&result, &[]);
    assert_constructs_no_default(&format_output(&result));

    insta::assert_snapshot!(format_output(&result));
}

#[test]
fn fuzzer_wrapper_before_test_keeps_the_marker_on_the_test_function() {
    let item = quote!(
        #[test]
        fn returning_fn(a: u8) -> Result<NonZero<u8>, u8> {
            Result::Ok(a)
        }
    );

    let result = fuzzer_wrapper(TokenStream::empty(), item);
    assert_diagnostics(&result, &[]);

    let output = format_output(&result);
    assert_constructs_no_default(&output);
    assert!(
        output.contains("#[__snforge_extract_config]"),
        "the test function must keep the marker, otherwise its config is never lifted:\n{output}"
    );

    insta::assert_snapshot!(output);
}

/// Config attributes accumulate in the test body one by one, and only `#[__snforge_extract_config]`
/// lifts them out. Several of them in a row must still leave nothing behind that would need a
/// value of the return type.
#[test]
fn several_config_attributes_are_all_lifted_out() {
    let result = test(TokenStream::empty(), returning_function());
    assert_diagnostics(&result, &[]);

    let item = get_function(&result.token_stream, "returning_fn", false);
    let result = available_gas(quote!((l2_gas: 999)), item);
    assert_diagnostics(&result, &[]);

    let item = get_function(&result.token_stream, "returning_fn", false);
    let result = fork(quote!(("name")), item);
    assert_diagnostics(&result, &[]);

    let item = get_function(&result.token_stream, "returning_fn", true);
    let result = extract_config(TokenStream::empty(), item);
    assert_diagnostics(&result, &[]);
    assert_constructs_no_default(&format_output(&result));

    insta::assert_snapshot!(format_output(&result));
}

/// `is_config_run` is exported from `snforge_std::_internals`, so a test body may start with an
/// `if` that looks like the one we generate. Such a block is not ours: it has no trailing
/// `return`, and we must leave it in the test instead of taking it apart.
#[test]
fn user_written_config_if_is_left_alone() {
    let item = quote!(
        fn returning_fn() -> Result<NonZero<u8>, u8> {
            if snforge_std::_internals::is_config_run() {}
            Result::Ok(1)
        }
    );

    let result = extract_config(TokenStream::empty(), item);
    assert_diagnostics(&result, &[]);

    let output = format_output(&result);
    assert!(
        output.contains("fn returning_fn__snforge_internal_config() {}"),
        "the user's `if` is not our config block, so nothing should be lifted out of it:\n{output}"
    );
    assert!(
        output.contains("if snforge_std::_internals::is_config_run() {}"),
        "the user's `if` must stay in the test function:\n{output}"
    );

    insta::assert_snapshot!(output);
}
