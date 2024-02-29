use crate::state::{start_cheat, stop_cheat, CheatSpan, CheatTarget};
use crate::CheatnetState;
use cairo_felt::Felt252;

#[derive(Clone, Default, Debug)]
pub struct TxInfoMock {
    pub version: Option<Felt252>,
    pub account_contract_address: Option<Felt252>,
    pub max_fee: Option<Felt252>,
    pub signature: Option<Vec<Felt252>>,
    pub transaction_hash: Option<Felt252>,
    pub chain_id: Option<Felt252>,
    pub nonce: Option<Felt252>,
    pub resource_bounds: Option<Vec<Felt252>>,
    pub tip: Option<Felt252>,
    pub paymaster_data: Option<Vec<Felt252>>,
    pub nonce_data_availability_mode: Option<Felt252>,
    pub fee_data_availability_mode: Option<Felt252>,
    pub account_deployment_data: Option<Vec<Felt252>>,
}

impl CheatnetState {
    pub fn spoof(&mut self, target: CheatTarget, tx_info_mock: TxInfoMock, span: CheatSpan) {
        start_cheat(
            &mut self.global_spoof,
            &mut self.spoofed_contracts,
            target,
            tx_info_mock,
            span,
        );
    }

    pub fn start_spoof(&mut self, target: CheatTarget, tx_info_mock: TxInfoMock) {
        self.spoof(target, tx_info_mock, CheatSpan::Indefinite);
    }

    pub fn stop_spoof(&mut self, target: CheatTarget) {
        stop_cheat(&mut self.global_spoof, &mut self.spoofed_contracts, target);
    }
}
