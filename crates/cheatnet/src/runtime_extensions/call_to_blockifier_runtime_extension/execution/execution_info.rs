use blockifier::execution::execution_utils::stark_felt_to_felt;
use cairo_felt::Felt252;
use cairo_vm::{
    types::relocatable::{MaybeRelocatable, Relocatable},
    vm::vm_core::VirtualMachine,
};
use conversions::FromConv;
use starknet_api::core::ContractAddress;

use crate::{
    runtime_extensions::forge_runtime_extension::cheatcodes::spoof::TxInfoMock,
    state::CheatnetState,
};

fn get_cheated_block_info_ptr(
    cheatnet_state: &CheatnetState,
    vm: &mut VirtualMachine,
    original_block_info: &[MaybeRelocatable],
    contract_address: &ContractAddress,
) -> Relocatable {
    // create a new segment with replaced block info
    let ptr_cheated_block_info = vm.add_memory_segment();

    let mut new_block_info = original_block_info.to_owned();

    if let Some(rolled_number) = cheatnet_state.get_cheated_block_number(contract_address) {
        new_block_info[0] = MaybeRelocatable::Int(rolled_number);
    };

    if let Some(warped_timestamp) = cheatnet_state.get_cheated_block_timestamp(contract_address) {
        new_block_info[1] = MaybeRelocatable::Int(warped_timestamp);
    }

    if let Some(elected_address) = cheatnet_state.get_cheated_sequencer_address(contract_address) {
        new_block_info[2] = MaybeRelocatable::Int(Felt252::from_(elected_address));
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
        .get_cheated_tx_info(contract_address)
        .unwrap();

    let TxInfoMock {
        version,
        account_contract_address,
        max_fee,
        signature,
        transaction_hash,
        chain_id,
        nonce,
        resource_bounds,
        tip,
        paymaster_data,
        nonce_data_availability_mode,
        fee_data_availability_mode,
        account_deployment_data,
    } = tx_info_mock;

    if let Some(version) = version {
        new_tx_info[0] = MaybeRelocatable::Int(version);
    };
    if let Some(account_contract_address) = account_contract_address {
        new_tx_info[1] = MaybeRelocatable::Int(account_contract_address);
    };
    if let Some(max_fee) = max_fee {
        new_tx_info[2] = MaybeRelocatable::Int(max_fee);
    };

    if let Some(signature) = signature {
        let (signature_start_ptr, signature_end_ptr) = add_vec_memory_segment(&signature, vm);
        new_tx_info[3] = signature_start_ptr.into();
        new_tx_info[4] = signature_end_ptr.into();
    }

    if let Some(transaction_hash) = transaction_hash {
        new_tx_info[5] = MaybeRelocatable::Int(transaction_hash);
    };
    if let Some(chain_id) = chain_id {
        new_tx_info[6] = MaybeRelocatable::Int(chain_id);
    };
    if let Some(nonce) = nonce {
        new_tx_info[7] = MaybeRelocatable::Int(nonce);
    };
    if let Some(resource_bounds) = resource_bounds {
        let (resource_bounds_start_ptr, resource_bounds_end_ptr) =
            add_vec_memory_segment(&resource_bounds, vm);
        new_tx_info[8] = resource_bounds_start_ptr.into();
        new_tx_info[9] = resource_bounds_end_ptr.into();
    }
    if let Some(tip) = tip {
        new_tx_info[10] = MaybeRelocatable::Int(tip);
    };
    if let Some(paymaster_data) = paymaster_data {
        let (paymaster_data_start_ptr, paymaster_data_end_ptr) =
            add_vec_memory_segment(&paymaster_data, vm);
        new_tx_info[11] = paymaster_data_start_ptr.into();
        new_tx_info[12] = paymaster_data_end_ptr.into();
    };
    if let Some(nonce_data_availability_mode) = nonce_data_availability_mode {
        new_tx_info[13] = MaybeRelocatable::Int(nonce_data_availability_mode);
    };
    if let Some(fee_data_availability_mode) = fee_data_availability_mode {
        new_tx_info[14] = MaybeRelocatable::Int(fee_data_availability_mode);
    };
    if let Some(account_deployment_data) = account_deployment_data {
        let (account_deployment_data_start_ptr, account_deployment_data_end_ptr) =
            add_vec_memory_segment(&account_deployment_data, vm);
        new_tx_info[15] = account_deployment_data_start_ptr.into();
        new_tx_info[16] = account_deployment_data_end_ptr.into();
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
    let ptr_cheated_exec_info = vm.add_memory_segment();

    // Initialize as old exec_info
    let mut new_exec_info = vm.get_continuous_range(execution_info_ptr, 5).unwrap();
    if cheatnet_state.address_is_rolled(contract_address)
        || cheatnet_state.address_is_warped(contract_address)
        || cheatnet_state.address_is_elected(contract_address)
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
            let original_tx_info = vm.get_continuous_range(tx_info_ptr, 17).unwrap();

            let ptr_cheated_tx_info =
                get_cheated_tx_info_ptr(cheatnet_state, vm, &original_tx_info, contract_address);

            new_exec_info[1] = MaybeRelocatable::RelocatableValue(ptr_cheated_tx_info);
        }
    }

    if cheatnet_state.address_is_pranked(contract_address) {
        new_exec_info[2] = MaybeRelocatable::Int(stark_felt_to_felt(
            *cheatnet_state
                .get_cheated_caller_address(contract_address)
                .expect("No caller address value found for the pranked contract address")
                .0
                .key(),
        ));
    }

    vm.load_data(ptr_cheated_exec_info, &new_exec_info).unwrap();

    ptr_cheated_exec_info
}

fn add_vec_memory_segment(
    vector: &Vec<Felt252>,
    vm: &mut VirtualMachine,
) -> (Relocatable, Relocatable) {
    let vector_len = vector.len();
    let vector_start_ptr = vm.add_memory_segment();
    let vector_end_ptr = (vector_start_ptr + vector_len).unwrap();

    let vector: Vec<MaybeRelocatable> = vector.iter().map(MaybeRelocatable::from).collect();
    vm.load_data(vector_start_ptr, &vector).unwrap();

    (vector_start_ptr, vector_end_ptr)
}
