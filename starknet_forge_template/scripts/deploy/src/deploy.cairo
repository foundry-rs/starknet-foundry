// // Necessary importations
use starknet::ContractAddress;
use array::ArrayTrait;
use core::result::ResultTrait;
use core::option::OptionTrait;
use core::debug::PrintTrait;
use snforge_std::{declare, ContractClass, ContractClassTrait, DeclareResult};
use starknet::class_hash::ClassHash;
use starknet::{deploy_syscall, SyscallResultTrait};

// Script for deploying contracts
fn main() {
    // Contract declaration
    let contract = declare("ContractName"); // Simple generic name

    match contract {
        Result::Ok(declared) => {
            'Class hash: '.print();
            match declared {
                DeclareResult::Success(contract_class) => {
                    let hash: felt252 = contract_class.class_hash.into();
                    hash.print();

                    // Contract deployment
                    let mut constructor_calldata = ArrayTrait::new();
                    let salt = 0x1234; // Unique salt for deployment

                    let deployment = deploy_syscall(
                        contract_class.class_hash, salt, constructor_calldata.span(), false
                    );

                    let (contract_address, _) = deployment.unwrap_syscall();
                    'Contract deployed successfully'.print();
                    'Contract address: '.print();
                    contract_address.print();
                },
                _ => { panic_with_felt252('Declaration failed'); }
            }
        },
        Result::Err(_) => {
            'Declaration failed'.print();
            panic_with_felt252('Declaration failed');
        },
    }
}
