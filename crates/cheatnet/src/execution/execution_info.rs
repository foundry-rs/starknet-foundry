use blockifier::execution::execution_utils::stark_felt_to_felt;
use cairo_vm::{
    types::relocatable::{MaybeRelocatable, Relocatable},
    vm::vm_core::VirtualMachine,
};
use starknet_api::core::ContractAddress;

use crate::{cheatcodes::spoof::TxInfoMock, state::CheatnetState};

fn get_cheated_block_info_ptr(
    cheatnet_state: &CheatnetState,
    vm: &mut VirtualMachine,
    original_block_info: &[MaybeRelocatable],
    contract_address: &ContractAddress,
) -> Relocatable {
    // create a new segment with replaced block info
    let ptr_cheated_block_info = vm.add_memory_segment();

    let mut new_block_info = original_block_info.to_owned();

    if let Some(rolled_number) = cheatnet_state.rolled_contracts.get(contract_address) {
        new_block_info[0] = MaybeRelocatable::Int(rolled_number.clone());
    };

    if let Some(warped_timestamp) = cheatnet_state.warped_contracts.get(contract_address) {
        new_block_info[1] = MaybeRelocatable::Int(warped_timestamp.clone());
    };

    vm.load_data(ptr_cheated_block_info, &new_block_info)
        .unwrap();
    ptr_cheated_block_info
}

fn get_cheated_tx_info_ptr(
    cheatnet_state: &CheatnetState,
    vm: &mut VirtualMachine,
    original_tx_info: &[MaybeRelocatable],
    contract_address: &ContractAddress,
) -> Relocatable {
    // create a new segment with replaced tx info
    let ptr_cheated_tx_info = vm.add_memory_segment();

    let mut new_tx_info = original_tx_info.to_owned();

    let tx_info_mock = cheatnet_state
        .spoofed_contracts
        .get(contract_address)
        .unwrap();
    let TxInfoMock {
        version,
        account_contract_address,
        max_fee,
        signature,
        transaction_hash,
        chain_id,
        nonce,
    } = tx_info_mock.to_owned();

    if let Some(version) = version {
        new_tx_info[0] = MaybeRelocatable::Int(version.clone());
    };
    if let Some(account_contract_address) = account_contract_address {
        new_tx_info[1] = MaybeRelocatable::Int(account_contract_address.clone());
    };
    if let Some(max_fee) = max_fee {
        new_tx_info[2] = MaybeRelocatable::Int(max_fee.clone());
    };

    if let Some(signature) = signature {
        let signature_len = signature.len();
        let signature_start_ptr = vm.add_memory_segment();
        let signature_end_ptr = (signature_start_ptr + signature_len).unwrap();
        let signature: Vec<MaybeRelocatable> =
            signature.iter().map(MaybeRelocatable::from).collect();
        vm.load_data(signature_start_ptr, &signature).unwrap();

        new_tx_info[3] = signature_start_ptr.into();
        new_tx_info[4] = signature_end_ptr.into();
    }

    if let Some(transaction_hash) = transaction_hash {
        new_tx_info[5] = MaybeRelocatable::Int(transaction_hash.clone());
    };
    if let Some(chain_id) = chain_id {
        new_tx_info[6] = MaybeRelocatable::Int(chain_id.clone());
    };
    if let Some(nonce) = nonce {
        new_tx_info[7] = MaybeRelocatable::Int(nonce.clone());
    };

    vm.load_data(ptr_cheated_tx_info, &new_tx_info).unwrap();
    ptr_cheated_tx_info
}

pub fn get_cheated_exec_info_ptr(
    cheatnet_state: &CheatnetState,
    vm: &mut VirtualMachine,
    execution_info_ptr: Relocatable,
    contract_address: &ContractAddress,
) -> Relocatable {
    // ExecutionInfo from corelib/src/starknet/info.cairo
    // block_info, tx_info, caller_address, contract_address, entry_point_selector

    let ptr_cheated_exec_info = vm.add_memory_segment();

    // Initialize as old exec_info
    let mut new_exec_info = vm.get_continuous_range(execution_info_ptr, 5).unwrap();

    if cheatnet_state.address_is_rolled(contract_address)
        || cheatnet_state.address_is_warped(contract_address)
    {
        let data = vm.get_range(execution_info_ptr, 1)[0].clone();
        if let MaybeRelocatable::RelocatableValue(block_info_ptr) = data.unwrap().into_owned() {
            let original_block_info = vm.get_continuous_range(block_info_ptr, 3).unwrap();

            let ptr_cheated_block_info = get_cheated_block_info_ptr(
                cheatnet_state,
                vm,
                &original_block_info,
                contract_address,
            );

            new_exec_info[0] = MaybeRelocatable::RelocatableValue(ptr_cheated_block_info);
        }
    }

    if cheatnet_state.address_is_spoofed(contract_address) {
        let data = vm.get_range(execution_info_ptr, 2)[1].clone();
        if let MaybeRelocatable::RelocatableValue(tx_info_ptr) = data.unwrap().into_owned() {
            let original_tx_info = vm.get_continuous_range(tx_info_ptr, 8).unwrap();

            let ptr_cheated_tx_info =
                get_cheated_tx_info_ptr(cheatnet_state, vm, &original_tx_info, contract_address);

            new_exec_info[1] = MaybeRelocatable::RelocatableValue(ptr_cheated_tx_info);
        }
    }

    if cheatnet_state.address_is_pranked(contract_address) {
        new_exec_info[2] = MaybeRelocatable::Int(stark_felt_to_felt(
            *cheatnet_state
                .pranked_contracts
                .get(contract_address)
                .expect("No caller address value found for the pranked contract address")
                .0
                .key(),
        ));
    }

    vm.load_data(ptr_cheated_exec_info, &new_exec_info).unwrap();

    ptr_cheated_exec_info
}
