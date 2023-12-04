use core::clone::Clone;
use starknet::{ContractAddress, testing::cheatcode, contract_address_const};
use option::OptionTrait;
use array::ArrayTrait;
use array::SpanTrait;
use snforge_std::CheatTarget;
use serde::Serde;

#[derive(Copy, Drop, Serde)]
struct TxInfoMock {
    version: Option<felt252>,
    account_contract_address: Option<ContractAddress>,
    max_fee: Option<u128>,
    signature: Option<Span<felt252>>,
    transaction_hash: Option<felt252>,
    chain_id: Option<felt252>,
    nonce: Option<felt252>,
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
        }
    }
}

fn start_spoof(target: CheatTarget, tx_info_mock: TxInfoMock) {
    let mut cheat_target_serialized: Array<felt252> = array![];
    target.serialize(ref cheat_target_serialized);

    let mut tx_info_serialized = array![];
    tx_info_mock.serialize(ref tx_info_serialized);

    let mut inputs: Array<felt252> = array![];
    extend_array(ref inputs, cheat_target_serialized.span());
    extend_array(ref inputs, tx_info_serialized.span());

    cheatcode::<'start_spoof'>(inputs.span());
}

fn stop_spoof(target: CheatTarget) {
    let mut inputs = array![];
    target.serialize(ref inputs);
    cheatcode::<'stop_spoof'>(inputs.span());
}

fn extend_array(ref array: Array<felt252>, mut span: Span<felt252>) {
    loop {
        match span.pop_front() {
            Option::Some(x) => { array.append(x.clone()); },
            Option::None => { break; }
        };
    };
}
