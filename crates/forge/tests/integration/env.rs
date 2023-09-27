use crate::integration::common::running_tests::run_test_case;
use crate::{assert_case_output_contains, assert_failed, assert_passed, test_case};
use cairo_felt::Felt252;
use indoc::indoc;
use num_bigint::BigUint;

#[test]
fn read_short_string() {
    let mut test = test_case!(indoc!(
        r#"
        use snforge_std::env::var;

        #[test]
        fn test_read_short_string() {
            let result = var('MY_ENV_VAR');
            assert(result == 'env_var_value', 'failed reading env var');
        }
    "#
    ));
    test.set_env("MY_ENV_VAR", "'env_var_value'");

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn read_felt252() {
    let mut test = test_case!(indoc!(
        r#"
        use snforge_std::env::var;

        #[test]
        fn test_read_felt252() {
            let result = var('MY_ENV_VAR');
            assert(result == 1234567, 'failed reading env var');
        }
    "#
    ));
    test.set_env("MY_ENV_VAR", "1234567");

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn read_invalid_felt252() {
    let mut test = test_case!(indoc!(
        r#"
        use snforge_std::env::var;

        #[test]
        fn test_read_invalid_felt252() {
            let result = var('MY_ENV_VAR');
        }
    "#
    ));

    let value = (Felt252::prime() + BigUint::from(1_u32)).to_string();
    test.set_env("MY_ENV_VAR", &value);

    let result = run_test_case(&test);

    assert_failed!(result);
    assert_case_output_contains!(
        result,
        "test_read_invalid_felt252",
        &format!("Failed to parse value = {value} to felt")
    );
}

#[test]
fn read_invalid_short_string() {
    let mut test = test_case!(indoc!(
        r#"
        use snforge_std::env::var;

        #[test]
        fn test_read_invalid_short_string() {
            let result = var('MY_ENV_VAR');
        }
    "#
    ));

    let value =
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    test.set_env("MY_ENV_VAR", value);

    let result = run_test_case(&test);

    assert_failed!(result);
    assert_case_output_contains!(
        result,
        "test_read_invalid_short_string",
        &format!("Failed to parse value = {value} to felt")
    );
}

#[test]
fn read_non_existent() {
    let test = test_case!(indoc!(
        r#"
        use snforge_std::env::var;

        #[test]
        fn test_read_invalid_short_string() {
            let result = var('MY_ENV_VAR');
        }
    "#
    ));
    let result = run_test_case(&test);

    assert_failed!(result);
    assert_case_output_contains!(
        result,
        "test_read_invalid_short_string",
        "Failed to read from env var = MY_ENV_VAR"
    );
}
