use std::env;
use crate::integration::common::running_tests::run_test_case;
use crate::{assert_passed, test_case};
use indoc::indoc;

#[test]
fn read_short_string() {
    env::set_var("MY_ENV_VAR", "'env_var_value'");

    let test = test_case!(indoc!(
        r#"
        use snforge_std::read_env_var;

        #[test]
        fn test_read_short_string() {
            let result = read_env_var('MY_ENV_VAR');
            assert(result == 'env_var_value', 'failed reading env var');
        }
    "#
    ));

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn read_felt252() {
    env::set_var("MY_ENV_VAR", "1234567");

    let test = test_case!(indoc!(
        r#"
        use snforge_std::read_env_var;

        #[test]
        fn test_read_short_string() {
            let result = read_env_var('MY_ENV_VAR');
            assert(result == 1234567, 'failed reading env var');
        }
    "#
    ));

    let result = run_test_case(&test);

    assert_passed!(result);
}