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
    #[allow(clippy::too_many_arguments)]
    pub fn start_spoof(
        &mut self,
        contract_address: ContractAddress,
        version: Option<Felt252>,
        account_contract_address: Option<Felt252>,
        max_fee: Option<Felt252>,
        signature: Option<Vec<Felt252>>,
        transaction_hash: Option<Felt252>,
        chain_id: Option<Felt252>,
        nonce: Option<Felt252>,
    ) {
        let tx_info = TxInfoMock {
            version,
            account_contract_address,
            max_fee,
            signature,
            transaction_hash,
            chain_id,
            nonce,
        };
        self.spoofed_contracts.insert(contract_address, tx_info);
    }

    pub fn stop_spoof(&mut self, contract_address: ContractAddress) {
        self.spoofed_contracts.remove(&contract_address);
    }
}
