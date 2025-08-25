use crate::utils::{assert_diagnostics, assert_output, empty_function};
use cairo_lang_macro::{Diagnostic, TokenStream, quote};
use snforge_scarb_plugin::attributes::test_case::test_case;

pub fn function_with_params() -> TokenStream {
    quote!(
        fn test_add(x: i128, y: i128, expected: i128) {
            let actual = x + y;
            assert!(actual == expected);
        }
    )
}

#[test]
fn works_with_args() {
    let args = quote!((1, 2, 3));

    let result = test_case(args, function_with_params());

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            #[implicit_precedence(core::pedersen::Pedersen, core::RangeCheck, core::integer::Bitwise, core::ec::EcOp, core::poseidon::Poseidon, core::SegmentArena, core::circuit::RangeCheck96, core::circuit::AddMod, core::circuit::MulMod, core::gas::GasBuiltin, System)]
            #[snforge_internal_test_executable]
            fn test_add_1_2_3(mut _data: Span<felt252>) -> Span<felt252> {
                core::internal::require_implicit::<System>();
                core::internal::revoke_ap_tracking();
                core::option::OptionTraitImpl::expect(core::gas::withdraw_gas(), 'Out of gas');
                core::option::OptionTraitImpl::expect(
                    core::gas::withdraw_gas_all(core::gas::get_builtin_costs()), 'Out of gas',
                );
                test_add(1, 2, 3);
                let mut arr = ArrayTrait::new();
                core::array::ArrayTrait::span(@arr)
            }
            fn test_add(x: i128, y: i128, expected: i128) {
                let actual = x + y;
                assert!(actual == expected);
            }
        ",
    );
}

#[test]
fn works_with_name_and_args() {
    let args = quote!((name: "one_and_two", 1, 2, 3));

    let result = test_case(args, function_with_params());

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            #[implicit_precedence(core::pedersen::Pedersen, core::RangeCheck, core::integer::Bitwise, core::ec::EcOp, core::poseidon::Poseidon, core::SegmentArena, core::circuit::RangeCheck96, core::circuit::AddMod, core::circuit::MulMod, core::gas::GasBuiltin, System)]
            #[snforge_internal_test_executable]
            fn test_add_one_and_two(mut _data: Span<felt252>) -> Span<felt252> {
                core::internal::require_implicit::<System>();
                core::internal::revoke_ap_tracking();
                core::option::OptionTraitImpl::expect(core::gas::withdraw_gas(), 'Out of gas');
                core::option::OptionTraitImpl::expect(
                    core::gas::withdraw_gas_all(core::gas::get_builtin_costs()), 'Out of gas',
                );
                test_add(1, 2, 3);
                let mut arr = ArrayTrait::new();
                core::array::ArrayTrait::span(@arr)
            }
            fn test_add(x: i128, y: i128, expected: i128) {
                let actual = x + y;
                assert!(actual == expected);
            }
        ",
    );
}

#[test]
fn invalid_args_number() {
    let args = quote!((1, 2));

    let result = test_case(args, function_with_params());

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[test_case] Expected 3 parameters, but got 2",
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
