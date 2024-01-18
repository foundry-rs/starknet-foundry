#[starknet::interface]
trait IConstructorSpoofChecker<TContractState> {
    fn get_stored_tx_hash(self: @TContractState) -> felt252;
}

#[starknet::contract]
mod ConstructorSpoofChecker {
    use box::BoxTrait;
    #[storage]
    struct Storage {
        stored_tx_hash: felt252,
    }

    #[constructor]
    fn constructor(ref self: ContractState) {
        let tx_hash = starknet::get_tx_info().unbox().transaction_hash;
        self.stored_tx_hash.write(tx_hash);
    }

    #[abi(embed_v0)]
    impl IConstructorSpoofChecker of super::IConstructorSpoofChecker<ContractState> {
        fn get_stored_tx_hash(self: @ContractState) -> felt252 {
            self.stored_tx_hash.read()
        }
    }
}
