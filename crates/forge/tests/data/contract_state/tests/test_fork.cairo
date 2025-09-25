#[starknet::interface]
trait IMap<TMapState> {
    fn put(ref self: TMapState, key: felt252, value: felt252);
    fn get(self: @TMapState, key: felt252) -> felt252;
}

#[starknet::contract]
mod Map {
    use starknet::storage::{Map, StorageMapReadAccess, StoragePathEntry, StoragePointerWriteAccess};
    #[storage]
    pub struct Storage {
        pub storage: Map<felt252, felt252>,
    }

    #[abi(embed_v0)]
    impl MapImpl of super::IMap<ContractState> {
        fn put(ref self: ContractState, key: felt252, value: felt252) {
            self.storage.entry(key).write(value);
        }

        fn get(self: @ContractState, key: felt252) -> felt252 {
            self.storage.read(key)
        }
    }
}
use snforge_std::interact_with_state;
use starknet::ContractAddress;
use starknet::storage::StorageMapWriteAccess;

#[test]
#[fork(url: "{{ NODE_RPC_URL }}", block_number: 900_000)]
fn test_fork_contract() {
    let contract_address: ContractAddress =
        0x00cd8f9ab31324bb93251837e4efb4223ee195454f6304fcfcb277e277653008
        .try_into()
        .unwrap();
    let dispatcher = IMapDispatcher { contract_address };

    assert(dispatcher.get(1) == 2, 'Wrong value');

    interact_with_state(
        contract_address,
        || {
            let mut state = Map::contract_state_for_testing();
            state.storage.write(1, 13579)
        },
    );

    assert(dispatcher.get(1) == 13579, 'Wrong value');
}
