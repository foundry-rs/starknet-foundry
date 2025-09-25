use snforge_std::fs::FileTrait;

#[test]
fn file_trait_example() {
    // Create an instance of `File` to be used later
    let _file = FileTrait::new("data/hello_starknet.txt");
}
