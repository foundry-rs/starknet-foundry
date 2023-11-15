use core::clone::Clone;
use starknet::{ContractAddress, testing::cheatcode, contract_address_const};
use option::OptionTrait;
use array::ArrayTrait;
use array::SpanTrait;
use snforge_std::CheatTarget;


#[derive(Copy, Drop)]
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

fn option_as_tuple<T, impl TDrop: Drop<T>>(option: Option<T>, default: T) -> (bool, T) {
    match option {
        Option::Some(x) => (true, x),
        Option::None => (false, default),
    }
}

fn join_arrays(ref a: Array<felt252>, ref b: Span<felt252>) {
    loop {
        match b.pop_front() {
              Option::Some(x) => {
                  a.append(x.clone());
              },
              Option::None => {break ();}
        };
    };
}

fn start_spoof(target: CheatTarget, tx_info_mock: TxInfoMock) {
    let TxInfoMock{version,
    account_contract_address,
    max_fee,
    signature,
    transaction_hash,
    chain_id,
    nonce } =
        tx_info_mock;

    let (is_version_set, version) = option_as_tuple(version, 0);

    let (is_acc_address_set, account_contract_address) = option_as_tuple(
        account_contract_address, contract_address_const::<0>()
    );
    let (is_max_fee_set, max_fee) = option_as_tuple(max_fee, 0_u128);
    let (is_tx_hash_set, transaction_hash) = option_as_tuple(transaction_hash, 0);
    let (is_chain_id_set, chain_id) = option_as_tuple(chain_id, 0);
    let (is_nonce_set, nonce) = option_as_tuple(nonce, 0);
    let (is_signature_set, mut signature) = option_as_tuple(signature, ArrayTrait::new().span());

    let mut cheat_target_serialized: Array<felt252> = array![];
    target.serialize(ref cheat_target_serialized);
    let mut cheat_target_serialized = cheat_target_serialized.span();

    let mut txn_info_serialized = array![
        is_version_set.into(),
        version,
        is_acc_address_set.into(),
        account_contract_address.into(),
        is_max_fee_set.into(),
        max_fee.into(),
        is_tx_hash_set.into(),
        transaction_hash,
        is_chain_id_set.into(),
        chain_id,
        is_nonce_set.into(),
        nonce,
        is_signature_set.into()
    ];
    let mut txn_info_serialized = txn_info_serialized.span();

    let mut inputs: Array<felt252> = array![];
    join_arrays(ref inputs, ref cheat_target_serialized);
    join_arrays(ref inputs, ref txn_info_serialized);
    let signature_len = signature.len();
    inputs.append(signature_len.into());
    join_arrays(ref inputs, ref signature);

    cheatcode::<'start_spoof'>(inputs.span());
}

fn stop_spoof(target: CheatTarget) {
    let mut inputs = array![];
    target.serialize(ref inputs);
    cheatcode::<'stop_spoof'>(inputs.span());
}
