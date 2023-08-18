use crate::CheatnetState;
use starknet_api::core::ContractAddress;
use cairo_felt::Felt252;

pub struct TxInfoMock {
    version: Felt252,
    account_contract_address: Felt252,
    max_fee: Felt252,
    signature: Vec<Felt252>,
    transaction_hash: Felt252,
    chain_id: Felt252,
    nonce: Felt252,
}

impl CheatnetState {
    pub fn start_spoof(
        &mut self,
        contract_address: ContractAddress,
        version: Felt252,
        account_contract_address: Felt252,
        max_fee: Felt252,
        signature: Vec<Felt252>,
        transaction_hash: Felt252,
        chain_id: Felt252,
        nonce: Felt252,
    ) {
        let tx_info = TxInfoMock {version, account_contract_address, max_fee, signature, transaction_hash, chain_id, nonce};

        self.cheatcode_state
            .spoofed_contracts
            .insert(contract_address, tx_info);
    }

    pub fn stop_spoof(
        &mut self,
        contract_address: ContractAddress,
    ) {
        self.cheatcode_state
            .spoofed_contracts
            .remove(&contract_address);
    }
}
