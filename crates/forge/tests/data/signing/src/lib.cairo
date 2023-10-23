use core::result::ResultTrait;
use snforge_std::{declare, ContractClass, ContractClassTrait, start_spoof, TxInfoMockTrait};
use snforge_std::signature::{StarkCurveKeyPair, StarkCurveKeyPairTrait};

use starknet::account::Call;

use openzeppelin::account::interface::ISRC6Dispatcher;
use openzeppelin::account::interface::ISRC6DispatcherTrait;


#[test]
fn test() {
    let mut key_pair = StarkCurveKeyPairTrait::generate();

    let contract = declare('Account');
    let account_address = contract.deploy(@array![key_pair.public_key]).unwrap();

    let message_hash = 123456;
    let (r, s) = key_pair.sign(message_hash).unwrap();

    let mut tx_info = TxInfoMockTrait::default();
    tx_info.transaction_hash = Option::Some(message_hash);
    tx_info.account_contract_address = Option::Some(account_address);
    tx_info.signature = Option::Some(array![r, s].span());

    start_spoof(account_address, tx_info);

    let account = ISRC6Dispatcher { contract_address: account_address };

    let valid = account.__validate__(array![]);

    assert(valid == 'VALID', 'Invalid signature');
}
