#[starknet::interface]
pub trait ISpyEventsChecker<TContractState> {
    fn emit_one_event(ref self: TContractState, some_data: felt252);
}

#[starknet::contract]
pub mod SpyEventsChecker {
    #[storage]
    struct Storage {}

    #[event]
    #[derive(Drop, starknet::Event)]
    pub enum Event {
        FirstEvent: FirstEvent
    }

    #[derive(Drop, starknet::Event)]
    pub struct FirstEvent {
        pub some_data: felt252
    }

    #[external(v0)]
    pub fn emit_one_event(ref self: ContractState, some_data: felt252) {
        self.emit(FirstEvent { some_data });
    }
}
