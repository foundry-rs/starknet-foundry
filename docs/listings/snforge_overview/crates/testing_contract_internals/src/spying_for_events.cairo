#[starknet::interface]
pub trait IEmitter<TContractState> {
    fn emit_event(ref self: TContractState);
}

#[starknet::contract]
pub mod Emitter {
    use core::result::ResultTrait;
    use starknet::ClassHash;

    #[event]
    #[derive(Drop, starknet::Event)]
    pub enum Event {
        ThingEmitted: ThingEmitted
    }

    #[derive(Drop, starknet::Event)]
    pub struct ThingEmitted {
        pub thing: felt252
    }

    #[storage]
    struct Storage {}

    #[external(v0)]
    pub fn emit_event(ref self: ContractState) {
        self.emit(Event::ThingEmitted(ThingEmitted { thing: 420 }));
    }
}
