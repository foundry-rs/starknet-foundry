use crate::common::corelib::corelib;
use crate::{assert_passed, test_case};
use camino::Utf8PathBuf;
use forge::run;
use indoc::indoc;
use std::collections::HashMap;

#[test]
fn simple() {
    let test = test_case!(indoc!(
        r#"#[test]
        fn test_two_and_two() {
            assert(2 == 2, '2 == 2');
        }
    "#
    ));

    let result = run(
        &test.path().unwrap(),
        Some(test.linked_libraries()),
        &Default::default(),
        Some(&Utf8PathBuf::from_path_buf(corelib().path().to_path_buf()).unwrap()),
        &HashMap::default(),
    )
    .unwrap();

    assert_passed!(result);
}
