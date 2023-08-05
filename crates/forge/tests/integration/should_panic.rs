use crate::integration::common::corelib::{corelib_path, predeployed_contracts};

use crate::{assert_passed, test_case};
use camino::Utf8PathBuf;
use forge::run;
use indoc::indoc;

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

    let result = run(
        &test.path().unwrap(),
        &test.path().unwrap().join("src/lib.cairo"),
        &Some(test.linked_libraries()),
        &Default::default(),
        &corelib_path(),
        &test.contracts(&corelib_path()).unwrap(),
        &Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
    )
    .unwrap();

    assert_passed!(result);
}
