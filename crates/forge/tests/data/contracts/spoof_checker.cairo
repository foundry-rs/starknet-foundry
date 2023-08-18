use starknet::info::TxInfo;
use serde::Serde;
use option::OptionTrait;

#[starknet::interface]
trait ISpoofChecker<TContractState> {
    fn get_tx_hash(ref self: TContractState) -> TxInfo;
}

impl TxInfoSerde of serde::Serde::<TxInfo> {
    fn serialize(self: @TxInfo, ref output: array::Array<felt252>) {
        serde::Serde::serialize(self.version, ref output);
        serde::Serde::serialize(self.account_contract_address, ref output);
        serde::Serde::serialize(self.max_fee, ref output);
        serde::Serde::serialize(self.signature, ref output);
        serde::Serde::serialize(self.transaction_hash, ref output);
        serde::Serde::serialize(self.chain_id, ref output);
        serde::Serde::serialize(self.nonce, ref output)
    }
    fn deserialize(ref serialized: array::Span<felt252>) -> Option<TxInfo> {
        Option::Some(TxInfo {
            version: serde::Serde::deserialize(ref serialized)?,
            account_contract_address: serde::Serde::deserialize(ref serialized)?,
            max_fee: serde::Serde::deserialize(ref serialized)?,
            signature: serde::Serde::deserialize(ref serialized)?,
            transaction_hash: serde::Serde::deserialize(ref serialized)?,
            chain_id: serde::Serde::deserialize(ref serialized)?,
            nonce: serde::Serde::deserialize(ref serialized)?,
        })
    }
}

#[starknet::contract]
mod SpoofChecker {
    use serde::Serde;
    use starknet::info::TxInfo;
    use box::BoxTrait;
    use super::TxInfoSerde;

    #[storage]
    struct Storage {
        balance: felt252,
    }

    #[external(v0)]
    impl ISpoofChecker of super::ISpoofChecker<ContractState> {
        fn get_tx_hash(ref self: ContractState) -> TxInfo {
            starknet::get_tx_info().unbox()
        }
    }
}
