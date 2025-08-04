use snforge_std::fs::{FileTrait, read_txt};

#[test]
fn read_txt_example() {
    // Create an instance of `File` to be used later
    let file = FileTrait::new("data/hello_starknet.txt");

    // Read the content of the file
    let content = read_txt(@file);

    let expected = array![
        'Hello Starknet!',
        'Let\'s code in Cairo!',
        // Below is serialized byte array "Example byte array"
        0,
        6051711116678136165665715375637410673222009,
        18,
    ];
    let mut i = 0;

    // Iterate through the content and compare with expected values
    while i != content.len() {
        println!("0x{:x}", *content[i]);
        assert(*content[i] == *expected[i], 'unexpected content');
        i += 1;
    };
}
