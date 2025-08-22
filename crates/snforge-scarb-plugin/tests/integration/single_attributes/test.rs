use crate::utils::{assert_diagnostics, assert_output, empty_function};
use cairo_lang_macro::{Diagnostic, TokenStream, quote};
use snforge_scarb_plugin::attributes::test::test;

#[test]
fn appends_internal_config_and_executable() {
    let args = TokenStream::empty();

    let result = test(args, empty_function());

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            #[implicit_precedence(core::pedersen::Pedersen, core::RangeCheck, core::integer::Bitwise, core::ec::EcOp, core::poseidon::Poseidon, core::SegmentArena, core::circuit::RangeCheck96, core::circuit::AddMod, core::circuit::MulMod, core::gas::GasBuiltin, System)]
            #[snforge_internal_test_executable]
            fn empty_fn_return_wrapper(mut _data: Span<felt252>) -> Span::<felt252> {
                core::internal::require_implicit::<System>();
                core::internal::revoke_ap_tracking();
                core::option::OptionTraitImpl::expect(core::gas::withdraw_gas(), 'Out of gas');

                core::option::OptionTraitImpl::expect(
                    core::gas::withdraw_gas_all(core::gas::get_builtin_costs()), 'Out of gas',
                );
                empty_fn();

                let mut arr = ArrayTrait::new();
                core::array::ArrayTrait::span(@arr)
            }

            #[__internal_config_statement]
            fn empty_fn() {}
        ",
    );
}

#[test]
fn fails_with_non_empty_args() {
    let args = quote!((123));

    let result = test(args, empty_function());

    assert_diagnostics(
        &result,
        &[Diagnostic::error("#[test] does not accept any arguments")],
    );
}

#[test]
fn is_used_once() {
    let item = quote!(
        #[test]
        fn empty_fn() {}
    );
    let args = TokenStream::empty();

    let result = test(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error("#[test] can only be used once per item")],
    );
}

#[test]
fn fails_with_params() {
    let item = quote!(
        fn empty_fn(f: felt252) {}
    );
    let args = TokenStream::empty();

    let result = test(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[test] function with parameters must have #[fuzzer] or #[test_case] attribute",
        )],
    );
}
