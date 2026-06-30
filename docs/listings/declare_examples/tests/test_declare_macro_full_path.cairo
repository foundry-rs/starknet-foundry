use snforge_std::DeclareResultTrait;

#[test]
fn test_declare_full_path() {
    let contract = declare!(declare_examples::hello_starknet::HelloStarknet)
        .unwrap()
        .contract_class();
    // ...
}

