use crate::CheatnetState;
use cairo_felt::Felt252;
use starknet_api::core::ContractAddress;

pub struct TxInfoMock {
    pub version: Option<Felt252>,
    pub account_contract_address: Option<Felt252>,
    pub max_fee: Option<Felt252>,
    pub signature: Option<Vec<Felt252>>,
    pub transaction_hash: Option<Felt252>,
    pub chain_id: Option<Felt252>,
    pub nonce: Option<Felt252>,
}

impl CheatnetState {
    pub fn start_spoof(&mut self, contract_address: ContractAddress, tx_info: TxInfoMock) {
        self.cheatcode_state
            .spoofed_contracts
            .insert(contract_address, tx_info);
    }

    pub fn stop_spoof(&mut self, contract_address: ContractAddress) {
        self.cheatcode_state
            .spoofed_contracts
            .remove(&contract_address);
    }
}
