#[starknet::contract]
mod ConstructorSpyEventsChecker {
    #[storage]
    struct Storage {}

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        FirstEvent: FirstEvent,
    }

    #[derive(Drop, starknet::Event)]
    struct FirstEvent {
        some_data: felt252,
    }

    #[constructor]
    fn constructor(ref self: ContractState, data: felt252) {
        self.emit(Event::FirstEvent(FirstEvent { some_data: data }));
    }
}
