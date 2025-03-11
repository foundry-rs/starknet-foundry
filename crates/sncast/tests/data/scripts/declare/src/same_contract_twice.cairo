use sncast_std::{
    get_nonce, declare, DeclareResult, DeclareResultTrait, ScriptCommandError, ProviderError,
    StarknetError, FeeSettings
};

fn main() {
    let fee_settings = FeeSettings {
        max_fee: Option::None,
        l1_gas: Option::Some(100000),
        l1_gas_price: Option::Some(10000000000000),
        l2_gas: Option::Some(1000000000),
        l2_gas_price: Option::Some(100000000000000000000),
        l1_data_gas: Option::Some(100000),
        l2_data_gas_price: Option::Some(10000000000000),
    };
    let declare_nonce = get_nonce('latest');
    let first_declare_result = declare("Mapa", fee_settings, Option::Some(declare_nonce))
        .expect('declare failed');
    println!("success");

    // Check if contract was declared successfully
    let class_hash = match first_declare_result {
        DeclareResult::Success(declare_transaction_result) => declare_transaction_result.class_hash,
        DeclareResult::AlreadyDeclared(_) => panic!("Should not be already declared"),
    };

    // Check declare result trait is implemented correctly for Success
    assert(*first_declare_result.class_hash() == class_hash, 'Class hashes must be equal');

    let declare_nonce = get_nonce('latest');
    let second_declare_result = declare("Mapa", fee_settings, Option::Some(declare_nonce))
        .expect('second declare failed');

    // Check if already declared contract was handled correctly
    match second_declare_result {
        DeclareResult::Success(_) => panic!("Should be already declared"),
        DeclareResult::AlreadyDeclared(already_declared_result) => assert!(
            already_declared_result.class_hash == class_hash
        ),
    }

    // Check declare result trait is implemented correctly for AlreadyDeclared
    assert(*second_declare_result.class_hash() == class_hash, 'Class hashes must be equal');

    println!("{:?}", first_declare_result);
    println!("{:?}", second_declare_result);
}
