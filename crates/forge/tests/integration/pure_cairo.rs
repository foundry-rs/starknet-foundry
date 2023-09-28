use indoc::indoc;
use utils::running_tests::run_test_case;
use utils::{assert_failed, assert_passed, test_case};

#[test]
fn simple() {
    let test = test_case!(indoc!(
        r#"#[test]
        fn test_two_and_two() {
            assert(2 == 2, '2 == 2');
        }
    "#
    ));

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn failing() {
    let test = test_case!(indoc!(
        r#"#[test]
        fn test_two_and_three() {
            assert(2 == 3, '2 == 3');
        }
    "#
    ));

    let result = run_test_case(&test);

    assert_failed!(result);
}
