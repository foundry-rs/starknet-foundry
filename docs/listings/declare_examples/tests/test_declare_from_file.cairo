use snforge_std::{DeclareResultTrait, declare_from_file};

#[test]
fn test_declare_from_file() {
    let contract = declare_from_file(
        "target/dev/declare_examples_HelloStarknet.contract_class.json",
    )
        .unwrap()
        .contract_class();
    // ...
}
