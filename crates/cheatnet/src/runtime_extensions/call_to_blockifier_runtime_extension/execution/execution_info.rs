use crate::state::{CheatedData, CheatedTxInfo};
use cairo_vm::{
    types::relocatable::{MaybeRelocatable, Relocatable},
    vm::vm_core::VirtualMachine,
};
use conversions::{
    IntoConv,
    serde::serialize::{SerializeToFeltVec, raw::RawFeltVec},
};
use starknet_types_core::felt::Felt;

fn get_cheated_block_info_ptr(
    vm: &mut VirtualMachine,
    original_block_info: &[MaybeRelocatable],
    cheated_data: &CheatedData,
) -> Relocatable {
    // create a new segment with replaced block info
    let ptr_cheated_block_info = vm.add_memory_segment();

    let mut new_block_info = original_block_info.to_owned();

    if let Some(block_number) = cheated_data.block_number {
        new_block_info[0] = MaybeRelocatable::Int(block_number.into());
    }

    if let Some(block_timestamp) = cheated_data.block_timestamp {
        new_block_info[1] = MaybeRelocatable::Int(block_timestamp.into());
    }

    if let Some(sequencer_address) = cheated_data.sequencer_address {
        new_block_info[2] = MaybeRelocatable::Int(sequencer_address.into_());
    }

    vm.load_data(ptr_cheated_block_info, &new_block_info)
        .unwrap();
    ptr_cheated_block_info
}

fn get_cheated_tx_info_ptr(
    vm: &mut VirtualMachine,
    original_tx_info: &[MaybeRelocatable],
    cheated_data: &CheatedData,
) -> Relocatable {
    // create a new segment with replaced tx info
    let ptr_cheated_tx_info = vm.add_memory_segment();

    let mut new_tx_info = original_tx_info.to_owned();

    let tx_info_mock = cheated_data.tx_info.clone();

    let CheatedTxInfo {
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
    }
    if let Some(account_contract_address) = account_contract_address {
        new_tx_info[1] = MaybeRelocatable::Int(account_contract_address);
    }
    if let Some(max_fee) = max_fee {
        new_tx_info[2] = MaybeRelocatable::Int(max_fee);
    }

    if let Some(signature) = signature {
        let (signature_start_ptr, signature_end_ptr) = add_vec_memory_segment(&signature, vm);
        new_tx_info[3] = signature_start_ptr.into();
        new_tx_info[4] = signature_end_ptr.into();
    }

    if let Some(transaction_hash) = transaction_hash {
        new_tx_info[5] = MaybeRelocatable::Int(transaction_hash);
    }
    if let Some(chain_id) = chain_id {
        new_tx_info[6] = MaybeRelocatable::Int(chain_id);
    }
    if let Some(nonce) = nonce {
        new_tx_info[7] = MaybeRelocatable::Int(nonce);
    }
    if let Some(resource_bounds) = resource_bounds {
        let (resource_bounds_start_ptr, resource_bounds_end_ptr) =
            add_vec_memory_segment(&RawFeltVec::new(resource_bounds).serialize_to_vec(), vm);
        new_tx_info[8] = resource_bounds_start_ptr.into();
        new_tx_info[9] = resource_bounds_end_ptr.into();
    }
    if let Some(tip) = tip {
        new_tx_info[10] = MaybeRelocatable::Int(tip);
    }
    if let Some(paymaster_data) = paymaster_data {
        let (paymaster_data_start_ptr, paymaster_data_end_ptr) =
            add_vec_memory_segment(&paymaster_data, vm);
        new_tx_info[11] = paymaster_data_start_ptr.into();
        new_tx_info[12] = paymaster_data_end_ptr.into();
    }
    if let Some(nonce_data_availability_mode) = nonce_data_availability_mode {
        new_tx_info[13] = MaybeRelocatable::Int(nonce_data_availability_mode);
    }
    if let Some(fee_data_availability_mode) = fee_data_availability_mode {
        new_tx_info[14] = MaybeRelocatable::Int(fee_data_availability_mode);
    }
    if let Some(account_deployment_data) = account_deployment_data {
        let (account_deployment_data_start_ptr, account_deployment_data_end_ptr) =
            add_vec_memory_segment(&account_deployment_data, vm);
        new_tx_info[15] = account_deployment_data_start_ptr.into();
        new_tx_info[16] = account_deployment_data_end_ptr.into();
    }

    vm.load_data(ptr_cheated_tx_info, &new_tx_info).unwrap();
    ptr_cheated_tx_info
}

pub fn get_cheated_exec_info_ptr(
    vm: &mut VirtualMachine,
    execution_info_ptr: Relocatable,
    cheated_data: &CheatedData,
) -> Relocatable {
    let ptr_cheated_exec_info = vm.add_memory_segment();

    // Initialize as old exec_info
    let mut new_exec_info = vm.get_continuous_range(execution_info_ptr, 5).unwrap();
    if cheated_data.block_number.is_some()
        || cheated_data.block_timestamp.is_some()
        || cheated_data.sequencer_address.is_some()
    {
        let data = vm.get_range(execution_info_ptr, 1)[0].clone();
        if let MaybeRelocatable::RelocatableValue(block_info_ptr) = data.unwrap().into_owned() {
            let original_block_info = vm.get_continuous_range(block_info_ptr, 3).unwrap();

            let ptr_cheated_block_info =
                get_cheated_block_info_ptr(vm, &original_block_info, cheated_data);
            new_exec_info[0] = MaybeRelocatable::RelocatableValue(ptr_cheated_block_info);
        }
    }

    if cheated_data.tx_info.is_mocked() {
        let data = vm.get_range(execution_info_ptr, 2)[1].clone();
        if let MaybeRelocatable::RelocatableValue(tx_info_ptr) = data.unwrap().into_owned() {
            let original_tx_info = vm.get_continuous_range(tx_info_ptr, 17).unwrap();

            let ptr_cheated_tx_info = get_cheated_tx_info_ptr(vm, &original_tx_info, cheated_data);

            new_exec_info[1] = MaybeRelocatable::RelocatableValue(ptr_cheated_tx_info);
        }
    }

    if let Some(caller_address) = cheated_data.caller_address {
        new_exec_info[2] = MaybeRelocatable::Int(caller_address.into_());
    }

    if let Some(contract_address) = cheated_data.contract_address {
        new_exec_info[3] = MaybeRelocatable::Int(contract_address.into_());
    }

    vm.load_data(ptr_cheated_exec_info, &new_exec_info).unwrap();

    ptr_cheated_exec_info
}

fn add_vec_memory_segment(vector: &[Felt], vm: &mut VirtualMachine) -> (Relocatable, Relocatable) {
    let vector_len = vector.len();
    let vector_start_ptr = vm.add_memory_segment();
    let vector_end_ptr = (vector_start_ptr + vector_len).unwrap();

    let vector: Vec<MaybeRelocatable> = vector.iter().map(MaybeRelocatable::from).collect();
    vm.load_data(vector_start_ptr, &vector).unwrap();

    (vector_start_ptr, vector_end_ptr)
}
