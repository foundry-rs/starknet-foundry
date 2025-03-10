use forge_runner::forge_config::ForgeTrackedResource;
use indoc::indoc;
use test_utils::runner::assert_passed;
use test_utils::running_tests::run_test_case;
use test_utils::test_case;

#[test]
fn simple_generate_random_felt() {
    let test = test_case!(indoc!(
        r"
        use snforge_std::generate_random_felt;

        #[test]
        fn simple_generate_random_felt() {
            let mut random_values = array![];
            let mut unique_values = array![];
            let mut i = 10;

        while i != 0 {
            let random_value = generate_random_felt();
            random_values.append(random_value);
            i -= 1;
        };

        for element in random_values.span() {
            let mut k = 0; 

            while k != random_values.len() {
                if element != random_values.at(k) {
                    unique_values.append(element);
                };
                k += 1;
            };
        };

        assert(unique_values.len() > 1, 'Identical values');
        }
        "
    ),);

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}
