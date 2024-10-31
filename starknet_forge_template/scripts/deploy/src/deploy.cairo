use sncast_std::{
    declare, declare::DeclareResult,    // For contract declaration functionality
    deploy, deploy::DeployResult,       // For contract deployment functionality
    get_nonce,                          // To fetch current nonce
    get_balance,                        // To check account balance
    ContractClassType,                  // Enum for contract class versions
};

fn main() {
    // Getting the current nonce for deployment
    let nonce = get_nonce().unwrap();
    println!("Current nonce: {}", nonce);
    
    // Checking account balance before deployment
    let balance = get_balance().unwrap();
    println!("Account balance: {}", balance);

    let declare_result = declare("HelloStarknet", ContractClassType::V2).expect('Declaration failed');
    println!("Contract declared with class hash: {}", declare_result.class_hash);
    
    // Preparing to deploy your contract
    let constructor_calldata = array![];    // empty array, in case of contracts without a constructor. You can add constructor arguments if needed
    let salt = 0x1234;                      // Unique value to determine contract address 'unique salt'

    // Deploying the declared contract
    let deploy_result = deploy(
        declare_result.class_hash,       // Use hash from declaration
        constructor_calldata.span(),     // Constructor arguments
        salt,                            // Unique salt value
        true                             // set to true to make deployment address unique
    ).expect('Deployment failed');
    
    println!("Contract deployed at: {}", deploy_result.contract_address);   // Print contract address
    println!("Transaction hash: {}", deploy_result.transaction_hash);       // Print transaction hash
}

