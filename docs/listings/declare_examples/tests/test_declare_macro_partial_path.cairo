use declare_examples::hello_starknet;
use snforge_std::DeclareResultTrait;

#[test]
fn test_declare_partial_path() {
    let contract = declare!(hello_starknet::HelloStarknet).unwrap().contract_class();
    // ...
}
