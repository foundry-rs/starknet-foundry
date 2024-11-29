#[starknet::contract]
pub mod ConstructorContract {

    #[storage]
    struct Storage {}

     #[constructor]
    fn constructor(ref self: ContractState, x: felt252, y: felt252, z: felt252) {
       
    }
}
