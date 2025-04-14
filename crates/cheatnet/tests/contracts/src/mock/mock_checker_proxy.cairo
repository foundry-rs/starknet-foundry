use starknet::ContractAddress;

#[starknet::interface]
trait IMockChecker<TContractState> {
    fn get_thing(ref self: TContractState) -> felt252;
}


#[starknet::interface]
trait IMockCheckerProxy<TContractState> {
    fn get_thing_from_contract(ref self: TContractState, address: ContractAddress) -> felt252;
    fn get_thing_from_contract_and_emit_event(
        ref self: TContractState, address: ContractAddress,
    ) -> felt252;
}

#[starknet::contract]
mod MockCheckerProxy {
    use starknet::ContractAddress;
    use super::IMockCheckerDispatcherTrait;
    use super::IMockCheckerDispatcher;

    #[storage]
    struct Storage {}

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        ThingEmitted: ThingEmitted,
    }

    #[derive(Drop, starknet::Event)]
    struct ThingEmitted {
        thing: felt252,
    }

    #[abi(embed_v0)]
    impl IMockCheckerProxy of super::IMockCheckerProxy<ContractState> {
        fn get_thing_from_contract(ref self: ContractState, address: ContractAddress) -> felt252 {
            let dispatcher = IMockCheckerDispatcher { contract_address: address };
            dispatcher.get_thing()
        }

        fn get_thing_from_contract_and_emit_event(
            ref self: ContractState, address: ContractAddress,
        ) -> felt252 {
            let dispatcher = IMockCheckerDispatcher { contract_address: address };
            let thing = dispatcher.get_thing();
            self.emit(Event::ThingEmitted(ThingEmitted { thing }));
            thing
        }
    }
}
