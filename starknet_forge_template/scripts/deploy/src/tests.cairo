use starknet::ContractAddress;
use array::ArrayTrait;
use core::result::ResultTrait;
use core::option::OptionTrait;
use core::debug::PrintTrait;
use snforge_std::{declare, ContractClass, ContractClassTrait, DeclareResult, spy_events,
EventSpy,};
use starknet::{deploy_syscall, SyscallResultTrait};

// start_prank, stop_prank, CheatTarget, SpyOn, EventFetcher

#[test]
fn test_contract_declaration() {
    // Test contract declaration
    let contract = snforge_std::declare("ContractName");

    match contract {
        Result::Ok(declared) => {
            match declared {
                DeclareResult::Success(contract_class) => {
                    // Verify class hash is not zero
                    let hash: felt252 = contract_class.class_hash.into();
                    assert(hash != 0, 'Invalid class hash');
                },
                _ => { panic_with_felt252('Declaration failed'); }
            }
        },
        Result::Err(_) => { panic_with_felt252('Declaration failed'); }
    }
}

#[test]
fn test_contract_deployment() {
    // First declare the contract
    let contract = snforge_std::declare("{{MyContract}}");

    match contract {
        Result::Ok(declared) => {
            match declared {
                DeclareResult::Success(contract_class) => {
                    // Prepare deployment parameters
                    let mut constructor_calldata = ArrayTrait::new();
                    let salt = 0x1234;

                    // Deploy the contract
                    let deployment = deploy_syscall(
                        contract_class.class_hash, salt, constructor_calldata.span(), false
                    );

                    // Verify deployment success
                    match deployment {
                        Result::Ok((
                            contract_address, _
                        )) => {
                            // Verify contract address is not zero
                            let address_felt: felt252 = contract_address.into();
                            assert(address_felt != 0, 'Invalid contract address');
                        },
                        Result::Err(_) => { panic_with_felt252('Deployment failed'); }
                    }
                },
                _ => { panic_with_felt252('Declaration failed'); }
            }
        },
        Result::Err(_) => { panic_with_felt252('Declaration failed'); }
    }
}

#[test]
#[should_panic(expected: ('Declaration failed',))]
fn test_invalid_contract_declaration() {
    // Try to declare a non-existent contract
    let contract = snforge_std::declare("NonExistentContract");

    match contract {
        Result::Ok(_) => { panic_with_felt252('Should not succeed'); },
        Result::Err(_) => { panic_with_felt252('Declaration failed'); }
    }
}
