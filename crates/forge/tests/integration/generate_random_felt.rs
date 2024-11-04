use indoc::indoc;
use test_utils::runner::assert_passed;
use test_utils::running_tests::run_test_case;
use test_utils::test_case;

#[test]
fn simple_generate_random_felt() {
    let test = test_case!(indoc!(
        r#"
        use result::ResultTrait;
        use snforge_std::generate_random_felt;

        #[test]
        fn simple_generate_random_felt() {
            assert(generate_random_felt() != 0, 'simple check');
        }
        "#
    ),);

    let result = run_test_case(&test);

    assert_passed(&result);
}
