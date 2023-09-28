use indoc::indoc;
use utils::running_tests::run_test_case;
use utils::{assert_passed, test_case};

#[test]
fn should_panic() {
    let test = test_case!(indoc!(
        r#"
            use array::ArrayTrait;

            #[test]
            #[should_panic]
            fn should_panic_with_no_expected_data() {
                panic_with_felt252(0);
            }

            #[test]
            #[should_panic(expected: ('panic message', ))]
            fn should_panic_check_data() {
                panic_with_felt252('panic message');
            }

            #[test]
            #[should_panic(expected: ('panic message', 'second message',))]
            fn should_panic_multiple_messages(){
                let mut arr = ArrayTrait::new();
                arr.append('panic message');
                arr.append('second message');
                panic(arr);
            }
        "#
    ));

    let result = run_test_case(&test);

    assert_passed!(result);
}
