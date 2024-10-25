use sncast_std::{
    get_nonce, declare, DeclareResult, ScriptCommandError, ProviderError, StarknetError,
    FeeSettings, EthFeeSettings
};

fn main() {
    let max_fee = 99999999999999999;

    let declare_nonce = get_nonce('latest');
    let first_declare_result = declare(
        "Mapa",
        FeeSettings::Eth(EthFeeSettings { max_fee: Option::Some(max_fee) }),
        Option::Some(declare_nonce)
    )
        .expect('declare failed');
    println!("success");
    
    let class_hash = match first_declare_result {
        DeclareResult::Success(declare_transaction_result) => declare_transaction_result.class_hash,
        DeclareResult::AlreadyDeclared(_) => panic!("Should not be already declared"),
    };

    let declare_nonce = get_nonce('latest');
    let second_declare_result = declare(
        "Mapa",
        FeeSettings::Eth(EthFeeSettings { max_fee: Option::Some(max_fee) }),
        Option::Some(declare_nonce)
    )
        .expect('second declare failed');

    match second_declare_result {
        DeclareResult::Success(_) => panic!("Should be already declare"),
        DeclareResult::AlreadyDeclared(already_declared_result) => assert!(already_declared_result.class_hash == class_hash),
    }


    println!("{:?}", first_declare_result);
    println!("{:?}", second_declare_result);
}
