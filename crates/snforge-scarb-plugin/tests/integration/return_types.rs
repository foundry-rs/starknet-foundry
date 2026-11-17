use crate::multiple_attributes::get_function;
use crate::utils::{assert_diagnostics, format_output};
use cairo_lang_macro::{quote, TokenStream};
use snforge_scarb_plugin::attributes::available_gas::available_gas;
use snforge_scarb_plugin::attributes::extract_config::{extract_config, ExtractConfigCollector};
use snforge_scarb_plugin::attributes::fork::fork;
use snforge_scarb_plugin::attributes::fuzzer::wrapper::fuzzer_wrapper;
use snforge_scarb_plugin::attributes::test::test;
use snforge_scarb_plugin::attributes::AttributeInfo;

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
fn test_leaves_body_untouched() {
    let result = test(TokenStream::empty(), returning_function());
    assert_diagnostics(&result, &[]);
    assert_constructs_no_default(&format_output(&result));

    insta::with_settings!({ snapshot_suffix => "after_test" }, {
        insta::assert_snapshot!(format_output(&result));
    });

    let item = get_function(&result.token_stream, "returning_fn", true);
    let result = extract_config(TokenStream::empty(), item);
    assert_diagnostics(&result, &[]);
    assert_constructs_no_default(&format_output(&result));

    insta::with_settings!({ snapshot_suffix => "after_extract_config" }, {
        insta::assert_snapshot!(format_output(&result));
    });
}

#[test]
fn lifts_config_with_fuzzer_before_test() {
    let item = quote!(
        #[test]
        fn returning_fn(a: u8) -> Result<NonZero<u8>, u8> {
            Result::Ok(a)
        }
    );

    let result = available_gas(quote!((l2_gas: 999)), item);
    assert_diagnostics(&result, &[]);

    let item = get_function(&result.token_stream, "returning_fn", false);
    let result = fuzzer_wrapper(TokenStream::empty(), item);
    assert_diagnostics(&result, &[]);

    let output = format_output(&result);
    let marker = format!("#[{}]", ExtractConfigCollector::ATTR_NAME);
    assert!(
        output.contains(&marker),
        "the test function must keep the marker, otherwise its config is never lifted:\n{output}"
    );

    insta::with_settings!({ snapshot_suffix => "after_fuzzer_wrapper" }, {
        insta::assert_snapshot!(output);
    });

    let item = get_function(&result.token_stream, "returning_fn", true);
    let result = extract_config(TokenStream::empty(), item);
    assert_diagnostics(&result, &[]);
    assert_constructs_no_default(&format_output(&result));

    insta::with_settings!({ snapshot_suffix => "after_extract_config" }, {
        insta::assert_snapshot!(format_output(&result));
    });
}

#[test]
fn lifts_config_from_several_attributes() {
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
