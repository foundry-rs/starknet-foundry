use snforge_std::fs::{FileTrait, read_json};

#[test]
fn read_json_example() {
    let file = FileTrait::new("data/user.json");
    let content = read_json(@file);

    let expected_serialized_json = array![
        30,
        0,
        28391512738467412385612170632190008583538,
        17,
        0,
        5649052288429290091,
        8,
        0,
        5591873,
        3,
        0,
        1248815214,
        4,
        0,
        4484965,
        3,
    ];
    let mut i = 0;

    while i != content.len() {
        println!("0x{:x}", *content[i]);
        assert!(*content[i] == *expected_serialized_json[i]);
        i += 1;
    };
}
