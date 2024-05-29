use crate::utils::{assert_diagnostics, assert_output, EMPTY_FN};
use cairo_lang_macro::TokenStream;
use snforge_scarb_plugin::attributes::{available_gas::available_gas, fork::fork, test::test};

#[test]
fn works_with_few_attributes() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new(String::new());

    let result = test(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            #[snforge_internal_test_executable]
            #[__internal_config_statement]
            fn empty_fn(){}
        ",
    );

    let item = result.token_stream;
    let args = TokenStream::new("(123)".into());

    let result = available_gas(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            #[snforge_internal_test_executable]
            #[__internal_config_statement]
            fn empty_fn() {
                if *starknet::testing::cheatcode::<'is_config_mode'>(array![].span()).at(0) == 1 {
                    let mut data = array![];

                    snforge_std::_config_types::AvailableGasConfig {
                        gas: 0x7b
                    }
                    .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_available_gas'>(data);

                    return; 
                }
            }
        ",
    );

    let item = result.token_stream;
    let args = TokenStream::new(r#"("test")"#.into());

    let result = fork(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        r#"
            #[snforge_internal_test_executable]
            #[__internal_config_statement]
            fn empty_fn() {
                if *starknet::testing::cheatcode::<'is_config_mode'>(array![].span()).at(0) == 1 {
                    let mut data = array![];

                    snforge_std::_config_types::AvailableGasConfig {
                        gas: 0x7b
                    }
                    .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_available_gas'>(data);

                    let mut data = array![];

                    snforge_std::_config_types::ForkConfig::Named("test")
                        .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_fork'>(data);

                    return; 
                }
            }
        "#,
    );
}
