# `store/load` Cheatcodes

## Context
People might want to manipulate the storage of contracts from the test context.
For example: we might want to modify the `authority` of the contract, without actually having an exposed endpoint in the contract, 
since it might impose some security issues. Having the function, to modify the stored `authority` will allow to change it,
without re-deploying the contract.

Also - some specific storage variables may not be exposed directly to the user via the contracts' interface, since it 
would bloat the interface and would not be needed for anything but tests (which is generally an antipattern in programming).

## Existing solutions

The [store](https://book.getfoundry.sh/cheatcodes/store) and [load](https://book.getfoundry.sh/cheatcodes/load) cheatcodes known from foundry,
provide the functionality to store and load memory in the VM for the given contract. 
Having the correct format of data (conversion), is up to the user, since the interface accepts bytes, and returns bytes as well.


## Proposed solution

My proposal would be to use the generated `contract_state_for_testing` and leverage it's typed structure, to 
enable the users to get the address of the variable, via the `<var_name>.address(<params>)` function, and 
implement a `store/load` functions, which use the: 
- Contract address
- Calculated variable address
- Value (in case of `store`)

## Caveats & pitfalls of the approach

1. Constructing `contract_state_for_testing` to only calculate the address of the variable
2. Implementing & importing `storage_access` for the objects we want to write (needs to be implemented anyway, just a bit inconvenient)

## Example usage
### Contract
```cairo
#[starknet::contract]
mod HelloStarknet {
    #[derive(starknet::Store)]
    struct CustomStruct {
        a: felt252,
        b: felt252,
    }

    #[storage]
    struct Storage {
        balance: felt252, 
        map: LegacyMap<felt252, felt252>,
        custom_struct: CustomStruct,
    }
}
```
### Test

```cairo
use array::ArrayTrait;
use result::ResultTrait;
use option::OptionTrait;
use traits::TryInto;
use starknet::ContractAddress;
use starknet::Felt252TryIntoContractAddress;

use snforge_std::{ declare, ContractClassTrait, store, load };

use pkg::HelloStarknet;
use pkg::HelloStarknet::{balanceContractMemberStateTrait, CustomStruct};

fn deploy_hello_contract() -> ContractAddress {
    let contract_class = declare('HelloStarknet');
    contract_class.deploy(@array![]).unwrap()
}

#[test]
fn test_felt252() {
    let contract_address = deploy_hello_contract();
    let state = HelloStarknet::contract_state_for_testing();
    let variable_address = state.balance.address();
    
    let new_balance = 420;
    store::<felt252>(contract_address, variable_address, new_balance);
    let stored_value = load::<felt252>(contract_address, variable_address);
    
    assert(stored_value == 420, 'Wrong balance stored');    
}

#[test]
fn test_legacy_map() {
    let contract_address = deploy_hello_contract();
    let state = HelloStarknet::contract_state_for_testing();
    
    
    let key = 420;
    let value = 69;
    let variable_address = state.map.address(key);
    
    store::<felt252>(contract_address, variable_address, value);
    let stored_value = load::<felt252>(contract_address, variable_address);
    
    assert(stored_value == 69, 'Wrong k:v stored');   
}

#[test]
fn test_custom_struct() {
    let contract_address = deploy_hello_contract();
    let state = HelloStarknet::contract_state_for_testing();
    
    
    let a = 420;
    let b = 69;
    let value = CustomStruct {a, b};
    let variable_address = state.custom_struct.address();
    
    store::<CustomStruct>(contract_address, variable_address, value);
    let stored_value = load::<CustomStruct>(contract_address, variable_address);
    
    assert(stored_value.a == 420, 'Wrong custom_struct.a stored');
    assert(stored_value.b == 69, 'Wrong custom_struct.a stored');   
}
```
