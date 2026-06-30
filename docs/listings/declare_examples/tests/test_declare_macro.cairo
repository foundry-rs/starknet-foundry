use declare_examples::hello_starknet::HelloStarknet;
use snforge_std::DeclareResultTrait;

#[test]
fn test_declare() {
    let contract = declare!(HelloStarknet).unwrap().contract_class();
    // ...
}

