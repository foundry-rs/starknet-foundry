# Testing smart contracts 

Using unit testing as much as possible is a good practice, as it makes your test suites run faster. However, when writing smart contracts you often want your test to cover the on-chain state and interactions between multiple contracts.

In this section, you will learn how to deploy and interact with a smart contract in Protostar for testing purposes.
How to test a contract

To test a contract, you need to use an important Protostar feature: cheatcodes. Cheatcodes are additional library functions that Protostar exposes to help you with testing.

Let's write a test which deploys and calls a contract. First let's define our contract in the file src/lib.cairo
Deployed contract

#[contract]
mod MinimalContract {
    #[external]
    fn hello() {
        assert(5 == 5, 'always true');
    }
}

You need to define contract in protostar.toml configuration file. Add it to the [contracts] section
Configuration file

[contracts]
minimal = ["your_project_name"]

We can write a test that deploys and calls this contract. Let's create a file test_contract.cairo:
Example

use array::ArrayTrait;
use result::ResultTrait;

#[test]
fn test_deploy() {
    let deployed_contract_address = deploy_contract('minimal', @ArrayTrait::new()).unwrap();
    invoke(deployed_contract_address, 'hello', @ArrayTrait::new()).unwrap();
}

deploy_contract will declare and deploy the given contract. invoke will invoke hello method.
Transaction reverts

Cheatcodes deploy, invoke and call execute code on chain which can be reverted. In such case, they return RevertedTransaction structure. You can use it, for example, to verify if your contract reverts the transaction in a certain scenario.

Here's how the structure looks:

struct RevertedTransaction {
    panic_data: Array::<felt252>, 
}

trait RevertedTransactionTrait {
    fn first(self: @RevertedTransaction) -> felt252; // Gets the first felt of the panic data
}

Example usage
Deployed contract

#[contract]
mod MinimalContract {
    #[external]
    fn panic_with(panic_data: Array::<felt252>) {
        panic(panic_data);
    }
}

Test

use cheatcodes::RevertedTransactionTrait;
use array::ArrayTrait;
use result::ResultTrait;

#[test]
fn test_invoke_errors() {
    let deployed_contract_address = deploy_contract('minimal', @ArrayTrait::new()).unwrap();
    let mut panic_data = ArrayTrait::new();
    panic_data.append(2); // Array length
    panic_data.append('error');
    panic_data.append('data');
    
    match invoke(deployed_contract_address, 'panic_with', @panic_data) {
        Result::Ok(x) => assert(false, 'Shouldnt have succeeded'),
        Result::Err(x) => {
            assert(x.first() == 'error', 'first datum doesnt match');
            assert(*x.panic_data.at(1_u32) == 'data', 'second datum doesnt match');
        }
    }
}

Cheatcodes in contract constructors

If you ever want to use prank, roll, warp or any of the environment-modifying cheatcodes in the constructor code, just split the deploy_contract into declare, prepare and deploy - so that you have a contract address (from prepare call) just before the deployment. Then you can use the cheatcode of your choice on the obtained address, and it will work in the constructor as well!
Example:
with_constructor.cairo

#[contract]
mod WithConstructor {
    use starknet::get_caller_address;
    use starknet::ContractAddress;
    use starknet::ContractAddressIntoFelt252;
    use traits::Into;


    struct Storage {
        owner: ContractAddress,
    }

    #[constructor]
    fn constructor() {
        let caller_address = get_caller_address();
        owner::write(caller_address);
    }

    #[view]
    fn get_owner() -> felt252 {
        owner::read().into()
    }
}

test_with_constructor.cairo

#[test]
fn test_prank_constructor() {
    let class_hash = declare('with_constructor').unwrap();
    let prepared = prepare(class_hash, @ArrayTrait::new()).unwrap();
    let owner_address = 123;

    start_prank(owner_address, prepared.contract_address).unwrap(); // <-- Prank before the deploy call

    let deployed_contract_address = deploy(prepared).unwrap();

    let return_data = call(deployed_contract_address, 'get_owner', @ArrayTrait::new()).unwrap();
    assert(*return_data.at(0_u32) == owner_address, 'check call result');
}