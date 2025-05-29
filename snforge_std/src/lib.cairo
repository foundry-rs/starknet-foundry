pub mod cheatcodes;

pub use cheatcodes::CheatSpan;

pub use cheatcodes::block_hash::cheat_block_hash;
pub use cheatcodes::block_hash::{
    start_cheat_block_hash, start_cheat_block_hash_global, stop_cheat_block_hash,
    stop_cheat_block_hash_global,
};

pub use cheatcodes::contract_class::declare;
pub use cheatcodes::contract_class::{
    ContractClass, ContractClassTrait, DeclareResult, DeclareResultTrait, get_class_hash,
};

pub use cheatcodes::erc20::set_balance;
pub use cheatcodes::erc20::{CustomToken, Token, TokenImpl, TokenTrait};

pub use cheatcodes::events::Event;
pub use cheatcodes::events::{
    EventSpy, EventSpyAssertionsTrait, EventSpyTrait, EventsFilterTrait, IsEmitted, spy_events,
};
pub use cheatcodes::execution_info::account_contract_address::{
    cheat_account_contract_address, start_cheat_account_contract_address,
    start_cheat_account_contract_address_global, stop_cheat_account_contract_address,
    stop_cheat_account_contract_address_global,
};
pub use cheatcodes::execution_info::account_deployment_data::{
    cheat_account_deployment_data, start_cheat_account_deployment_data,
    start_cheat_account_deployment_data_global, stop_cheat_account_deployment_data,
    stop_cheat_account_deployment_data_global,
};
pub use cheatcodes::execution_info::block_number::{
    cheat_block_number, start_cheat_block_number, start_cheat_block_number_global,
    stop_cheat_block_number, stop_cheat_block_number_global,
};
pub use cheatcodes::execution_info::block_timestamp::{
    cheat_block_timestamp, start_cheat_block_timestamp, start_cheat_block_timestamp_global,
    stop_cheat_block_timestamp, stop_cheat_block_timestamp_global,
};

pub use cheatcodes::execution_info::caller_address::cheat_caller_address;
pub use cheatcodes::execution_info::caller_address::{
    start_cheat_caller_address, start_cheat_caller_address_global, stop_cheat_caller_address,
    stop_cheat_caller_address_global,
};
pub use cheatcodes::execution_info::chain_id::{
    cheat_chain_id, start_cheat_chain_id, start_cheat_chain_id_global, stop_cheat_chain_id,
    stop_cheat_chain_id_global,
};
pub use cheatcodes::execution_info::fee_data_availability_mode::{
    cheat_fee_data_availability_mode, start_cheat_fee_data_availability_mode,
    start_cheat_fee_data_availability_mode_global, stop_cheat_fee_data_availability_mode,
    stop_cheat_fee_data_availability_mode_global,
};
pub use cheatcodes::execution_info::max_fee::{
    cheat_max_fee, start_cheat_max_fee, start_cheat_max_fee_global, stop_cheat_max_fee,
    stop_cheat_max_fee_global,
};
pub use cheatcodes::execution_info::nonce::{
    cheat_nonce, start_cheat_nonce, start_cheat_nonce_global, stop_cheat_nonce,
    stop_cheat_nonce_global,
};
pub use cheatcodes::execution_info::nonce_data_availability_mode::{
    cheat_nonce_data_availability_mode, start_cheat_nonce_data_availability_mode,
    start_cheat_nonce_data_availability_mode_global, stop_cheat_nonce_data_availability_mode,
    stop_cheat_nonce_data_availability_mode_global,
};
pub use cheatcodes::execution_info::paymaster_data::{
    cheat_paymaster_data, start_cheat_paymaster_data, start_cheat_paymaster_data_global,
    stop_cheat_paymaster_data, stop_cheat_paymaster_data_global,
};
pub use cheatcodes::execution_info::resource_bounds::{
    cheat_resource_bounds, start_cheat_resource_bounds, start_cheat_resource_bounds_global,
    stop_cheat_resource_bounds, stop_cheat_resource_bounds_global,
};
pub use cheatcodes::execution_info::sequencer_address::{
    cheat_sequencer_address, start_cheat_sequencer_address, start_cheat_sequencer_address_global,
    stop_cheat_sequencer_address, stop_cheat_sequencer_address_global,
};
pub use cheatcodes::execution_info::signature::{
    cheat_signature, start_cheat_signature, start_cheat_signature_global, stop_cheat_signature,
    stop_cheat_signature_global,
};
pub use cheatcodes::execution_info::tip::{
    cheat_tip, start_cheat_tip, start_cheat_tip_global, stop_cheat_tip, stop_cheat_tip_global,
};
pub use cheatcodes::execution_info::transaction_hash::{
    cheat_transaction_hash, start_cheat_transaction_hash, start_cheat_transaction_hash_global,
    stop_cheat_transaction_hash, stop_cheat_transaction_hash_global,
};
pub use cheatcodes::execution_info::version::{
    cheat_transaction_version, start_cheat_transaction_version,
    start_cheat_transaction_version_global, stop_cheat_transaction_version,
    stop_cheat_transaction_version_global,
};

pub use cheatcodes::generate_random_felt::generate_random_felt;

pub use cheatcodes::l1_handler::L1Handler;
pub use cheatcodes::l1_handler::L1HandlerTrait;

pub use cheatcodes::message_to_l1::{
    MessageToL1, MessageToL1FilterTrait, MessageToL1Spy, MessageToL1SpyAssertionsTrait,
    MessageToL1SpyTrait, spy_messages_to_l1,
};

pub use cheatcodes::storage::store;
pub use cheatcodes::storage::{load, map_entry_address};
pub use cheatcodes::{
    ReplaceBytecodeError, mock_call, replace_bytecode, start_mock_call, stop_mock_call,
    test_address, test_selector,
};

pub mod byte_array;

mod cheatcode;

mod config_types;

pub mod env;

pub mod fs;

pub mod fuzzable;

pub mod signature;

pub mod trace;

#[doc(hidden)]
pub mod _internals {
    pub use cheatcode::is_config_run;
    pub use cheatcode::save_fuzzer_arg;
    use super::cheatcode;

    pub use super::config_types;
}
