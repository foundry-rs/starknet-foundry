use starknet::{ContractAddress, testing::cheatcode, contract_address_const};
use starknet::info::v2::ResourceBounds;
use snforge_std::cheatcodes::{CheatSpan, CheatTarget, validate_cheat_target_and_span};
use super::super::_cheatcode::handle_cheatcode;

#[derive(Copy, Drop, Serde)]
struct TxInfoMock {
    version: Option<felt252>,
    account_contract_address: Option<ContractAddress>,
    max_fee: Option<u128>,
    signature: Option<Span<felt252>>,
    transaction_hash: Option<felt252>,
    chain_id: Option<felt252>,
    nonce: Option<felt252>,
    // starknet::info::v2::TxInfo fields
    resource_bounds: Option<Span<ResourceBounds>>,
    tip: Option<u128>,
    paymaster_data: Option<Span<felt252>>,
    nonce_data_availability_mode: Option<u32>,
    fee_data_availability_mode: Option<u32>,
    account_deployment_data: Option<Span<felt252>>,
}

trait TxInfoMockTrait {
    fn default() -> TxInfoMock;
}

impl TxInfoMockImpl of TxInfoMockTrait {
    fn default() -> TxInfoMock {
        TxInfoMock {
            version: Option::None(()),
            account_contract_address: Option::None(()),
            max_fee: Option::None(()),
            signature: Option::None(()),
            transaction_hash: Option::None(()),
            chain_id: Option::None(()),
            nonce: Option::None(()),
            resource_bounds: Option::None(()),
            tip: Option::None(()),
            paymaster_data: Option::None(()),
            nonce_data_availability_mode: Option::None(()),
            fee_data_availability_mode: Option::None(()),
            account_deployment_data: Option::None(()),
        }
    }
}

fn spoof(target: CheatTarget, tx_info_mock: TxInfoMock, span: CheatSpan) {
    validate_cheat_target_and_span(@target, @span);

    let mut inputs = array![];
    target.serialize(ref inputs);
    span.serialize(ref inputs);
    tx_info_mock.serialize(ref inputs);
    handle_cheatcode(cheatcode::<'spoof'>(inputs.span()));
}

fn start_spoof(target: CheatTarget, tx_info_mock: TxInfoMock) {
    spoof(target, tx_info_mock, CheatSpan::Indefinite);
}

fn stop_spoof(target: CheatTarget) {
    let mut inputs = array![];
    target.serialize(ref inputs);
    handle_cheatcode(cheatcode::<'stop_spoof'>(inputs.span()));
}

