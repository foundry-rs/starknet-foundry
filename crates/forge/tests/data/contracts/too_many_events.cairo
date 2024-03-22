use starknet::ContractAddress;

#[starknet::interface]
trait ITooManyEvents<TContractState> {
    fn emit_too_many_events(self: @TContractState, count: felt252);
    fn emit_too_many_keys(self: @TContractState, count: felt252);
    fn emit_too_many_data(self: @TContractState, count: felt252);
}

#[starknet::contract]
mod TooManyEvents {
    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl TooManyEventsImpl of super::ITooManyEvents<ContractState> {
        fn emit_too_many_events(self: @ContractState, mut count: felt252) {
            while count != 0 {
                starknet::emit_event_syscall(array![0].span(), array![0].span()).unwrap();
                count -= 1;
            };
        }
        fn emit_too_many_keys(self: @ContractState, mut count: felt252) {
            let mut arr = array![];
            while count != 0 {
                arr.append(0);
                count -= 1;
            };

            starknet::emit_event_syscall(arr.span(), array![0].span()).unwrap();
        }
        fn emit_too_many_data(self: @ContractState, mut count: felt252) {
            let mut arr = array![];
            while count != 0 {
                arr.append(0);
                count -= 1;
            };

            starknet::emit_event_syscall(array![0].span(), arr.span()).unwrap();
        }
    }
}
