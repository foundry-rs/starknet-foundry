#[starknet::interface]
trait IReplaceBytecode<TContractState> {
    fn get(self: @TContractState) -> felt252;
    fn emit_event(ref self: TContractState);
    fn libcall(self: @TContractState, class_hash: starknet::ClassHash) -> felt252;
}

#[starknet::interface]
trait ILib<TContractState> {
    fn get(self: @TContractState) -> felt252;
}

#[starknet::contract]
mod Lib {
    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl ILib of super::ILib<ContractState> {
        fn get(self: @ContractState) -> felt252 {
            123456789
        }
    }
}

#[starknet::contract]
mod ReplaceBytecodeA {
    use super::{ILibLibraryDispatcher, ILibDispatcherTrait};
    #[storage]
    struct Storage {}

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        ValueChanged: ValueChanged,
    }

    #[derive(Drop, starknet::Event)]
    struct ValueChanged {
        value: felt252,
    }

    #[abi(embed_v0)]
    impl IReplaceBytecodeA of super::IReplaceBytecode<ContractState> {
        fn get(self: @ContractState) -> felt252 {
            2137
        }
        fn emit_event(ref self: ContractState) {
            self.emit(Event::ValueChanged(ValueChanged { value: 2137 }));
        }
        fn libcall(self: @ContractState, class_hash: starknet::ClassHash) -> felt252 {
            let dispatcher = ILibLibraryDispatcher { class_hash };
            dispatcher.get()
        }
    }
}

#[starknet::contract]
mod ReplaceBytecodeB {
    use super::{ILibLibraryDispatcher, ILibDispatcherTrait};
    #[storage]
    struct Storage {}

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        ValueChanged: ValueChanged,
    }

    #[derive(Drop, starknet::Event)]
    struct ValueChanged {
        value: felt252,
    }

    #[abi(embed_v0)]
    impl IReplaceBytecodeB of super::IReplaceBytecode<ContractState> {
        fn get(self: @ContractState) -> felt252 {
            420
        }
        fn emit_event(ref self: ContractState) {
            self.emit(Event::ValueChanged(ValueChanged { value: 420 }));
        }
        fn libcall(self: @ContractState, class_hash: starknet::ClassHash) -> felt252 {
            let dispatcher = ILibLibraryDispatcher { class_hash };
            dispatcher.get()
        }
    }
}
