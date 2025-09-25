use snforge_std::fs::{FileTrait, read_txt};

#[test]
#[should_panic(expected: "No such file or directory")]
fn catch_no_such_file() {
    let file = FileTrait::new("no_way_this_file_exists");
    let content = read_txt(@file);

    assert!(false);
}
