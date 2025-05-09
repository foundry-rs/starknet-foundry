use crate::utils::{assert_diagnostics, assert_output, EMPTY_FN, FN_WITH_SINGLE_FELT252_PARAM};
use cairo_lang_macro::{Diagnostic, TokenStream};
use indoc::formatdoc;
use snforge_scarb_plugin::attributes::test::test;

#[test]
fn appends_internal_config_and_executable() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new(String::new());

    let result = test(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            #[snforge_internal_test_executable]
            #[implicit_precedence(core::pedersen::Pedersen, core::RangeCheck, core::integer::Bitwise, core::ec::EcOp, core::poseidon::Poseidon, core::SegmentArena, core::circuit::RangeCheck96, core::circuit::AddMod, core::circuit::MulMod, core::gas::GasBuiltin, System)]
            fn empty_fn(mut _data: Span<felt252>) -> Span::<felt252> {
                core::internal::require_implicit::<System>();
                core::internal::revoke_ap_tracking();
                core::option::OptionTraitImpl::expect(core::gas::withdraw_gas(), 'Out of gas');

                core::option::OptionTraitImpl::expect(
                    core::gas::withdraw_gas_all(core::gas::get_builtin_costs()), 'Out of gas',
                );
                empty_fn_return_wrapper();

                let mut arr = ArrayTrait::new();
                core::array::ArrayTrait::span(@arr)
            }

            #[__internal_config_statement]
            fn empty_fn_return_wrapper() {}
        ",
    );
}

#[test]
fn fails_with_non_empty_args() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new("(123)".into());

    let result = test(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error("#[test] does not accept any arguments")],
    );
}

#[test]
fn is_used_once() {
    let item = TokenStream::new(formatdoc!(
        "
            #[test]
            {EMPTY_FN}
        "
    ));
    let args = TokenStream::new(String::new());

    let result = test(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error("#[test] can only be used once per item")],
    );
}

#[test]
fn fails_with_params() {
    let item = TokenStream::new(FN_WITH_SINGLE_FELT252_PARAM.into());
    let args = TokenStream::new(String::new());

    let result = test(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[test] function with parameters must have #[fuzzer] attribute",
        )],
    );
}
