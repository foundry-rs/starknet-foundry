use indoc::indoc;
use num_bigint::BigUint;
use starknet_types_core::felt::Felt;
use test_utils::running_tests::run_test_case;
use test_utils::{
    runner::{assert_case_output_contains, assert_failed, assert_passed},
    test_case,
};

#[test]
fn read_short_string() {
    let mut test = test_case!(indoc!(
        r#"
        use snforge_std::env::var;

        #[test]
        fn read_short_string() {
            let result = var("MY_ENV_VAR");
            assert(result == array!['env_var_value'], 'failed reading env var');
        }
    "#
    ));
    test.set_env("MY_ENV_VAR", "'env_var_value'");

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
fn read_felt252() {
    let mut test = test_case!(indoc!(
        r#"
        use snforge_std::env::var;

        #[test]
        fn read_felt252() {
            let result = var("MY_ENV_VAR");
            assert(result == array![1234567], 'failed reading env var');
        }
    "#
    ));
    test.set_env("MY_ENV_VAR", "1234567");

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
fn read_bytearray() {
    let mut test = test_case!(indoc!(
        r#"
        use snforge_std::env::var;

        #[test]
        fn read_bytearray() {
            let mut result = var("MY_ENV_VAR").span();
            let result_bytearray = Serde::<ByteArray>::deserialize(ref result).unwrap();
            assert(result_bytearray == "very long string literal very very long very very long", 'failed reading env var');
        }
    "#
    ));
    test.set_env(
        "MY_ENV_VAR",
        r#""very long string literal very very long very very long""#,
    );

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
fn read_overflow_felt252() {
    let mut test = test_case!(indoc!(
        r#"
        use snforge_std::env::var;

        #[test]
        fn read_overflow_felt252() {
            let result = var("MY_ENV_VAR");
            assert(result == array![1], '');
        }
    "#
    ));

    let value = (Felt::prime() + BigUint::from(1_u32)).to_string();
    test.set_env("MY_ENV_VAR", &value);

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
fn read_invalid_short_string() {
    let mut test = test_case!(indoc!(
        r#"
        use snforge_std::env::var;

        #[test]
        fn read_invalid_short_string() {
            var("MY_ENV_VAR");
        }
    "#
    ));

    let value =
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    test.set_env("MY_ENV_VAR", value);

    let result = run_test_case(&test);

    assert_failed(&result);
    assert_case_output_contains(
        &result,
        "read_invalid_short_string",
        &format!("Failed to parse value = {value} to felt"),
    );
}

#[test]
fn read_non_existent() {
    let test = test_case!(indoc!(
        r#"
        use snforge_std::env::var;

        #[test]
        fn read_invalid_short_string() {
            var("MY_ENV_VAR");
        }
    "#
    ));
    let result = run_test_case(&test);

    assert_failed(&result);
    assert_case_output_contains(
        &result,
        "read_invalid_short_string",
        "Failed to read from env var = MY_ENV_VAR",
    );
}
