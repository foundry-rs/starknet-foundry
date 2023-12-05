#[derive(Serde, Drop)]
struct StructThing {
    item_one: felt252,
    item_two: felt252,
}

#[starknet::interface]
trait IMockChecker<TContractState> {
    fn get_thing(ref self: TContractState) -> felt252;
    fn get_thing_wrapper(ref self: TContractState) -> felt252;
    fn get_constant_thing(ref self: TContractState) -> felt252;
    fn get_struct_thing(ref self: TContractState) -> StructThing;
    fn get_arr_thing(ref self: TContractState) -> Array<StructThing>;
}

#[starknet::contract]
mod MockChecker {
    use super::IMockChecker;
    use super::StructThing;
    use array::ArrayTrait;

    #[storage]
    struct Storage {
        stored_thing: felt252
    }

    #[constructor]
    fn constructor(ref self: ContractState, arg1: felt252) {
        self.stored_thing.write(arg1)
    }

    #[abi(embed_v0)]
    impl IMockCheckerImpl of super::IMockChecker<ContractState> {
        fn get_thing(ref self: ContractState) -> felt252 {
            self.stored_thing.read()
        }

        fn get_thing_wrapper(ref self: ContractState) -> felt252 {
            self.get_thing()
        }

        fn get_constant_thing(ref self: ContractState) -> felt252 {
            13
        }

        fn get_struct_thing(ref self: ContractState) -> StructThing {
            StructThing {item_one: 12, item_two: 21}
        }

        fn get_arr_thing(ref self: ContractState) -> Array<StructThing> {
            array![StructThing {item_one: 12, item_two: 21}]
        }
    }
}
