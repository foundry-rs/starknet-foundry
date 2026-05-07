use crate::utils::{assert_diagnostics, empty_function, format_output};
use cairo_lang_macro::{Diagnostic, TextSpan, Token, TokenStream, TokenTree, quote};
use snforge_scarb_plugin::attributes::fuzzer::wrapper::fuzzer_wrapper;
use snforge_scarb_plugin::attributes::fuzzer::{fuzzer, fuzzer_config};

#[test]
fn work_without_args() {
    let args = TokenStream::empty();

    let result = fuzzer(args, empty_function());

    assert_diagnostics(&result, &[]);

    insta::assert_snapshot!(format_output(&result));
}

#[test]
fn work_with_args() {
    let args = quote!((runs: 655, seed: 32872357));

    let result = fuzzer(args, empty_function());

    assert_diagnostics(&result, &[]);

    insta::assert_snapshot!(format_output(&result));
}

#[test]
fn config_works_with_runs_only() {
    let args = quote!((runs: 655));

    let result = fuzzer_config(args, empty_function());

    assert_diagnostics(&result, &[]);

    insta::assert_snapshot!(format_output(&result));
}

#[test]
fn config_works_with_seed_only() {
    let args = quote!((seed: 655));

    let result = fuzzer_config(args, empty_function());

    assert_diagnostics(&result, &[]);

    insta::assert_snapshot!(format_output(&result));
}

#[test]
fn config_works_with_both_args() {
    let args = quote!((runs: 655, seed: 32872357));

    let result = fuzzer_config(args, empty_function());

    assert_diagnostics(&result, &[]);

    insta::assert_snapshot!(format_output(&result));
}

#[test]
fn config_wrapper_work_without_args() {
    let args = TokenStream::empty();

    let result = fuzzer_config(args, empty_function());

    assert_diagnostics(&result, &[]);

    insta::with_settings!({ snapshot_suffix => "after_fuzzer_config" }, {
        insta::assert_snapshot!(format_output(&result));
    });

    let item = result.token_stream;
    let args = TokenStream::empty();

    let result = fuzzer_wrapper(args, item);

    assert_diagnostics(&result, &[]);

    insta::with_settings!({ snapshot_suffix => "after_fuzzer_wrapper" }, {
        insta::assert_snapshot!(format_output(&result));
    });
}

#[test]
fn config_wrapper_work_with_both_args() {
    let args = quote!((runs: 655, seed: 32872357));

    let result = fuzzer_config(args, empty_function());

    assert_diagnostics(&result, &[]);

    insta::with_settings!({ snapshot_suffix => "after_fuzzer_config" }, {
        insta::assert_snapshot!(format_output(&result));
    });

    let item = result.token_stream;
    let args = TokenStream::empty();

    let result = fuzzer_wrapper(args, item);

    assert_diagnostics(&result, &[]);

    insta::with_settings!({ snapshot_suffix => "after_fuzzer_wrapper" }, {
        insta::assert_snapshot!(format_output(&result));
    });
}

#[test]
fn config_wrapper_work_with_fn_with_single_param() {
    let item = quote!(
        fn empty_fn(f: felt252) {}
    );
    let args = TokenStream::empty();

    let result = fuzzer_config(args, item);

    assert_diagnostics(&result, &[]);

    insta::with_settings!({ snapshot_suffix => "after_fuzzer_config" }, {
        insta::assert_snapshot!(format_output(&result));
    });

    let item = result.token_stream;
    let args = TokenStream::empty();

    let result = fuzzer_wrapper(args, item);

    assert_diagnostics(&result, &[]);

    insta::with_settings!({ snapshot_suffix => "after_fuzzer_wrapper" }, {
        insta::assert_snapshot!(format_output(&result));
    });
}

#[test]
fn config_wrapper_work_with_fn_with_params() {
    let item = quote!(
        fn empty_fn(f: felt252, u: u32) {}
    );
    let args = TokenStream::empty();

    let result = fuzzer_config(args, item);

    assert_diagnostics(&result, &[]);

    insta::with_settings!({ snapshot_suffix => "after_fuzzer_config" }, {
        insta::assert_snapshot!(format_output(&result));
    });

    let item = result.token_stream;
    let args = TokenStream::empty();

    let result = fuzzer_wrapper(args, item);

    assert_diagnostics(&result, &[]);

    insta::with_settings!({ snapshot_suffix => "after_fuzzer_wrapper" }, {
        insta::assert_snapshot!(format_output(&result));
    });
}

#[test]
fn wrapper_handle_attributes() {
    let item = quote!(
        #[available_gas(l2_gas: 40000)]
        #[test]
        fn empty_fn() {}
    );
    let args = TokenStream::empty();

    let result = fuzzer_wrapper(args, item);

    insta::assert_snapshot!(format_output(&result));
}

#[test]
fn fail_with_invalid_args() {
    let args = TokenStream::new(vec![TokenTree::Ident(Token::new(
        "(seed: '655')",
        TextSpan::call_site(),
    ))]);

    let result = fuzzer_config(args, empty_function());

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[fuzzer] <seed> should be number literal",
        )],
    );
}

#[test]
fn fail_with_unnamed_arg() {
    let args = quote!((123));

    let result = fuzzer_config(args, empty_function());

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[fuzzer] can be used with named arguments only",
        )],
    );
}

#[test]
fn is_used_once() {
    let item = quote!(
        #[fuzzer]
        fn empty_fn() {}
    );
    let args = TokenStream::empty();

    let result = fuzzer(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[fuzzer] can only be used once per item",
        )],
    );
}

#[test]
fn fails_with_unexpected_args() {
    let args = quote!((runs: 100, tomato: 123));

    let result = fuzzer_config(args, empty_function());

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[fuzzer] unexpected argument(s): <tomato>",
        )],
    );
}
