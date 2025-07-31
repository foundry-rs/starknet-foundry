use snforge_std::fs::{FileTrait};

#[test]
fn file_trait_example() {
    let _file = FileTrait::new("data/hello_starknet.txt");
    // Later we can use the file instance to read data from it
}
