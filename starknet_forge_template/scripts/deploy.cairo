use sncast_std::{
    declare, declare::DeclareResult, 
    deploy, deploy::DeployResult,
    get_nonce, 
    get_balance,
    ContractClassType,
};

fn main() {
    let nonce = get_nonce().unwrap();
    println!("Current nonce: {}", nonce);
   
    let balance = get_balance().unwrap();
    println!("Account balance: {}", balance);

    let declare_result = declare("{{ PROJECT_NAME }}", ContractClassType::V2).expect('Declaration failed');
    println!("Contract declared with class hash: {}", declare_result.class_hash);
    
    let constructor_calldata = array![]; 
    let salt = 0x1234;  // Use a unique salt or generate one

    let deploy_result = deploy(
        declare_result.class_hash, 
        constructor_calldata.span(),
        salt, 
        true  // This is set to true to make deployment address unique
    ).expect('Deployment failed');
    
    println!("Contract deployed at: {}", deploy_result.contract_address);
    println!("Transaction hash: {}", deploy_result.transaction_hash);
}