#[starknet::contract]
mod ConstructorSimple2 {
    #[storage]
    struct Storage {
        number: felt252
    }

    #[constructor]
    fn constructor(ref self: ContractState, number: felt252, number2: felt252) {
        self.number.write(number + number2);
    }

    #[abi(embed_v0)]
    fn add_to_number(ref self: ContractState, number: felt252) -> felt252 {
        let new_number = self.number.read() + number;
        self.number.write(new_number);
        new_number
    }

    #[abi(embed_v0)]
    fn get_number(self: @ContractState) -> felt252 {
        self.number.read()
    }
}
