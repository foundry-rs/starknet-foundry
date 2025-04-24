use blockifier::execution::contract_class::{CompiledClassV1, RunnableCompiledClass};
use blockifier::execution::syscalls::hint_processor::SyscallHintProcessor;
use blockifier::state::cached_state::CachedState;
use cheatnet::constants::{STRK_CLASS_HASH, STRK_CONTRACT_ADDRESS, strk_constructor_calldata};
use cheatnet::data::STRK_ERC20_CASM;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::declare::declare_with_contract_class;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::deploy::deploy_at;
use cheatnet::state::{CheatnetState, ExtendedStateReader};
use conversions::string::TryFromHexStr;
use starknet_api::contract_class::SierraVersion;
use starknet_api::core::{ClassHash, ContractAddress};
use starknet_types_core::felt::Felt;

fn declare_token(
    cached_state: &mut CachedState<ExtendedStateReader>,
    class_hash: ClassHash,
    casm: &str,
) {
    let contract_class = RunnableCompiledClass::V1(
        CompiledClassV1::try_from_json_string(casm, SierraVersion::LATEST).unwrap(),
    );
    declare_with_contract_class(cached_state, contract_class, class_hash)
        .expect("Failed to declare class");
}

fn deploy_token(
    syscall_handler: &mut SyscallHintProcessor,
    cheatnet_state: &mut CheatnetState,
    class_hash: ClassHash,
    contract_address: ContractAddress,
    constructor_calldata: &[Felt],
) -> bool {
    let deploy_result = deploy_at(
        syscall_handler,
        cheatnet_state,
        &class_hash,
        constructor_calldata,
        contract_address,
        true,
    );

    // It's possible that token can be already deployed (forking)
    deploy_result.is_ok()
}

pub fn declare_token_strk(cached_state: &mut CachedState<ExtendedStateReader>) {
    let class_hash = ClassHash::try_from_hex_str(STRK_CLASS_HASH).unwrap();
    declare_token(cached_state, class_hash, STRK_ERC20_CASM);
}

pub fn deploy_token_strk(
    syscall_handler: &mut SyscallHintProcessor,
    cheatnet_state: &mut CheatnetState,
) -> bool {
    let class_hash = ClassHash::try_from_hex_str(STRK_CLASS_HASH).unwrap();
    let contract_address = ContractAddress::try_from_hex_str(STRK_CONTRACT_ADDRESS).unwrap();
    let constructor_calldata = strk_constructor_calldata();
    deploy_token(
        syscall_handler,
        cheatnet_state,
        class_hash,
        contract_address,
        &constructor_calldata,
    )
}
