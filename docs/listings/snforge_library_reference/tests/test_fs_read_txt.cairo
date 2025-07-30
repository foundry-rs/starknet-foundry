use snforge_std::fs::{FileTrait, read_txt};

#[test]
fn read_txt_example() {
    let file = FileTrait::new("data/hello_starknet.txt");
    let content = read_txt(@file);

    let expected = array!['Hello Starknet!', 'Let\'s code in Cairo!'];
    let mut i = 0;

    while i != content.len() {
        println!("{}", content[i]);
        assert(*content[i] == *expected[i], 'unexpected content');
        i += 1;
    };
}
