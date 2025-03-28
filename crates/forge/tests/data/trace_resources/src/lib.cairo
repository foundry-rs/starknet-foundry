mod empty;
mod trace_dummy;
mod trace_info_checker;
mod trace_info_proxy;

use starknet::{
    ContractAddress, ClassHash, get_block_hash_syscall, get_contract_address, emit_event_syscall,
    send_message_to_l1_syscall, SyscallResultTrait,
};

fn use_builtins_and_syscalls(empty_hash: ClassHash, salt: felt252) -> ContractAddress {
    1_u8 >= 1_u8;
    1_u8 & 1_u8;
    core::pedersen::pedersen(1, 2);
    core::poseidon::hades_permutation(0, 0, 0);
    let ec_point = core::ec::EcPointTrait::new_from_x(1).unwrap();
    core::ec::EcPointTrait::mul(ec_point, 2);

    keccak::keccak_u256s_le_inputs(array![1].span());

    get_block_hash_syscall(1).unwrap_syscall();
    starknet::deploy_syscall(empty_hash, salt, array![].span(), false).unwrap_syscall();
    emit_event_syscall(array![1].span(), array![2].span()).unwrap_syscall();
    send_message_to_l1_syscall(10, array![20, 30].span()).unwrap_syscall();
    let x = starknet::storage_read_syscall(0, 10.try_into().unwrap()).unwrap_syscall();
    starknet::storage_write_syscall(0, 10.try_into().unwrap(), x).unwrap_syscall();

    get_contract_address()
}
