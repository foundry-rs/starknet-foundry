# Cheatcodes Reference


- [`mock_call`](cheatcodes/mock_call.md#mock_call) - mocks a number of contract calls to an entry point
- [`start_mock_call`](cheatcodes/mock_call.md#start_mock_call) - mocks contract call to an entry point
- [`stop_mock_call`](cheatcodes/mock_call.md#stop_mock_call) - cancels the `mock_call` / `start_mock_call` for an entry point
- [`get_class_hash`](cheatcodes/get_class_hash.md) - retrieves a class hash of a contract
- [`replace_bytecode`](cheatcodes/replace_bytecode.md) - replace the class hash of a contract
- [`l1_handler`](cheatcodes/l1_handler.md) - executes a `#[l1_handler]` function to mock a message arriving from Ethereum
- [`spy_events`](cheatcodes/spy_events.md) - creates `EventSpy` instance which spies on events emitted by contracts
- [`spy_messages_to_l1`](cheatcodes/spy_messages_to_l1.md) - creates `L1MessageSpy` instance which spies on messages to L1 sent by contracts
- [`store`](cheatcodes/store.md) - stores values in targeted contact's storage
- [`load`](cheatcodes/load.md) - loads values directly from targeted contact's storage
- [`set_balance`](cheatcodes/set_balance.md) - sets new balance of ERC20 token for target contract

- [`CheatSpan`](cheatcodes/cheat_span.md) - enum for specifying the number of target calls for a cheat
- [`Token`](cheatcodes/token.md) - enum for specifying ERC20 token for a cheat
- [`interact_with_state`](cheatcodes/interact_with_state.md) - allows interacting with a contract's state in tests

## Execution Info

### Caller Address

- [`cheat_caller_address`](cheatcodes/caller_address.md#cheat_caller_address) - changes the caller address for contracts, for a number of calls
- [`start_cheat_caller_address_global`](cheatcodes/caller_address.md#start_cheat_caller_address_global) - changes the caller address for all contracts
- [`start_cheat_caller_address`](cheatcodes/caller_address.md#start_cheat_caller_address) - changes the caller address for contracts
- [`stop_cheat_caller_address`](cheatcodes/caller_address.md#stop_cheat_caller_address) - cancels the `cheat_caller_address` / `start_cheat_caller_address` for contracts
- [`stop_cheat_caller_address_global`](cheatcodes/caller_address.md#stop_cheat_caller_address_global) - cancels the `start_cheat_caller_address_global`

## Block Info

### Block Number

- [`cheat_block_number`](cheatcodes/block_number.md#cheat_block_number) - changes the block number for contracts, for a number of calls
- [`start_cheat_block_number_global`](cheatcodes/block_number.md#start_cheat_block_number_global) - changes the block number for all contracts
- [`start_cheat_block_number`](cheatcodes/block_number.md#start_cheat_block_number) - changes the block number for contracts
- [`stop_cheat_block_number`](cheatcodes/block_number.md#stop_cheat_block_number) - cancels the `cheat_block_number` / `start_cheat_block_number` for contracts
- [`stop_cheat_block_number_global`](cheatcodes/block_number.md#stop_cheat_block_number_global) - cancels the `start_cheat_block_number_global`

### Block Timestamp

- [`cheat_block_timestamp`](cheatcodes/block_timestamp.md#cheat_block_timestamp) - changes the block timestamp for contracts, for a number of calls
- [`start_cheat_block_timestamp_global`](cheatcodes/block_timestamp.md#start_cheat_block_timestamp_global) - changes the block timestamp for all contracts
- [`start_cheat_block_timestamp`](cheatcodes/block_timestamp.md#start_cheat_block_timestamp) - changes the block timestamp for contracts
- [`stop_cheat_block_timestamp`](cheatcodes/block_timestamp.md#stop_cheat_block_timestamp) - cancels the `cheat_block_timestamp` / `start_cheat_block_timestamp` for contracts
- [`stop_cheat_block_timestamp_global`](cheatcodes/block_timestamp.md#stop_cheat_block_timestamp_global) - cancels the `start_cheat_block_timestamp_global`

### Sequencer Address

- [`cheat_sequencer_address`](cheatcodes/sequencer_address.md#cheat_sequencer_address) - changes the sequencer address for contracts, for a number of calls
- [`start_cheat_sequencer_address_global`](cheatcodes/sequencer_address.md#start_cheat_sequencer_address_global) - changes the sequencer address for all contracts
- [`start_cheat_sequencer_address`](cheatcodes/sequencer_address.md#start_cheat_sequencer_address) - changes the sequencer address for contracts
- [`stop_cheat_sequencer_address`](cheatcodes/sequencer_address.md#stop_cheat_sequencer_address) - cancels the `cheat_sequencer_address` / `start_cheat_sequencer_address` for contracts
- [`stop_cheat_sequencer_address_global`](cheatcodes/sequencer_address.md#stop_cheat_sequencer_address_global) - cancels the `start_cheat_sequencer_address_global`

## Transaction Info

### Transaction Version

- [`cheat_transaction_version`](cheatcodes/transaction_version.md#cheat_transaction_version) - changes the transaction version for contracts, for a number of calls
- [`start_cheat_transaction_version_global`](cheatcodes/transaction_version.md#start_cheat_transaction_version_global) - changes the transaction version for all contracts
- [`start_cheat_transaction_version`](cheatcodes/transaction_version.md#start_cheat_transaction_version) - changes the transaction version for contracts
- [`stop_cheat_transaction_version`](cheatcodes/transaction_version.md#stop_cheat_transaction_version) - cancels the `cheat_transaction_version` / `start_cheat_transaction_version` for contracts
- [`stop_cheat_transaction_version_global`](cheatcodes/transaction_version.md#stop_cheat_transaction_version_global) - cancels the `start_cheat_transaction_version_global`

### Transaction Max Fee

- [`cheat_max_fee`](cheatcodes/max_fee.md#cheat_max_fee) - changes the transaction max fee for contracts, for a number of calls
- [`start_cheat_max_fee_global`](cheatcodes/max_fee.md#start_cheat_max_fee_global) - changes the transaction max fee for all contracts
- [`start_cheat_max_fee`](cheatcodes/max_fee.md#start_cheat_max_fee) - changes the transaction max fee for contracts
- [`stop_cheat_max_fee`](cheatcodes/max_fee.md#stop_cheat_max_fee) - cancels the `cheat_max_fee` / `start_cheat_max_fee` for contracts
- [`stop_cheat_max_fee_global`](cheatcodes/max_fee.md#stop_cheat_max_fee_global) - cancels the `start_cheat_max_fee_global`

### Transaction Signature

- [`cheat_signature`](cheatcodes/signature.md#cheat_signature) - changes the transaction signature for contracts, for a number of calls
- [`start_cheat_signature_global`](cheatcodes/signature.md#start_cheat_signature_global) - changes the transaction signature for all contracts
- [`start_cheat_signature`](cheatcodes/signature.md#start_cheat_signature) - changes the transaction signature for contracts
- [`stop_cheat_signature`](cheatcodes/signature.md#stop_cheat_signature) - cancels the `cheat_signature` / `start_cheat_signature` for contracts
- [`stop_cheat_signature_global`](cheatcodes/signature.md#stop_cheat_signature_global) - cancels the `start_cheat_signature_global`

### Transaction Hash

- [`cheat_transaction_hash`](cheatcodes/transaction_hash.md#cheat_transaction_hash) - changes the transaction hash for contracts, for a number of calls
- [`start_cheat_transaction_hash_global`](cheatcodes/transaction_hash.md#start_cheat_transaction_hash_global) - changes the transaction hash for all contracts
- [`start_cheat_transaction_hash`](cheatcodes/transaction_hash.md#start_cheat_transaction_hash) - changes the transaction hash for contracts
- [`stop_cheat_transaction_hash`](cheatcodes/transaction_hash.md#stop_cheat_transaction_hash) - cancels the `cheat_transaction_hash` / `start_cheat_transaction_hash` for contracts
- [`stop_cheat_transaction_hash_global`](cheatcodes/transaction_hash.md#stop_cheat_transaction_hash_global) - cancels the `start_cheat_transaction_hash_global`

### Transaction Chain ID

- [`cheat_chain_id`](cheatcodes/chain_id.md#cheat_chain_id) - changes the transaction chain_id for contracts, for a number of calls
- [`start_cheat_chain_id_global`](cheatcodes/chain_id.md#start_cheat_chain_id_global) - changes the transaction chain_id for all contracts
- [`start_cheat_chain_id`](cheatcodes/chain_id.md#start_cheat_chain_id) - changes the transaction chain_id for contracts
- [`stop_cheat_chain_id`](cheatcodes/chain_id.md#stop_cheat_chain_id) - cancels the `cheat_chain_id` / `start_cheat_chain_id` for contracts
- [`stop_cheat_chain_id_global`](cheatcodes/chain_id.md#stop_cheat_chain_id_global) - cancels the `start_cheat_chain_id_global`

### Transaction Nonce

- [`cheat_nonce`](cheatcodes/nonce.md#cheat_nonce) - changes the transaction nonce for contracts, for a number of calls
- [`start_cheat_nonce_global`](cheatcodes/nonce.md#start_cheat_nonce_global) - changes the transaction nonce for all contracts
- [`start_cheat_nonce`](cheatcodes/nonce.md#start_cheat_nonce) - changes the transaction nonce for contracts
- [`stop_cheat_nonce`](cheatcodes/nonce.md#stop_cheat_nonce) - cancels the `cheat_nonce` / `start_cheat_nonce` for contracts
- [`stop_cheat_nonce_global`](cheatcodes/nonce.md#stop_cheat_nonce_global) - cancels the `start_cheat_nonce_global`

### Transaction Resource Bounds

- [`cheat_resource_bounds`](cheatcodes/resource_bounds.md#cheat_resource_bounds) - changes the transaction resource bounds for contracts, for a number of calls
- [`start_cheat_resource_bounds_global`](cheatcodes/resource_bounds.md#start_cheat_resource_bounds_global) - changes the transaction resource bounds for all contracts
- [`start_cheat_resource_bounds`](cheatcodes/resource_bounds.md#start_cheat_resource_bounds) - changes the transaction resource bounds for contracts
- [`stop_cheat_resource_bounds`](cheatcodes/resource_bounds.md#stop_cheat_resource_bounds) - cancels the `cheat_resource_bounds` / `start_cheat_resource_bounds` for contracts
- [`stop_cheat_resource_bounds_global`](cheatcodes/resource_bounds.md#stop_cheat_resource_bounds_global) - cancels the `start_cheat_resource_bounds_global`

### Transaction Tip

- [`cheat_tip`](cheatcodes/tip.md#cheat_tip) - changes the transaction tip for contracts, for a number of calls
- [`start_cheat_tip_global`](cheatcodes/tip.md#start_cheat_tip_global) - changes the transaction tip for all contracts
- [`start_cheat_tip`](cheatcodes/tip.md#start_cheat_tip) - changes the transaction tip for contracts
- [`stop_cheat_tip`](cheatcodes/tip.md#stop_cheat_tip) - cancels the `cheat_tip` / `start_cheat_tip` for contracts
- [`stop_cheat_tip_global`](cheatcodes/tip.md#stop_cheat_tip_global) - cancels the `start_cheat_tip_global`

### Transaction Paymaster Data

- [`cheat_paymaster_data`](cheatcodes/paymaster_data.md#cheat_paymaster_data) - changes the transaction paymaster data for contracts, for a number of calls
- [`start_cheat_paymaster_data_global`](cheatcodes/paymaster_data.md#start_cheat_paymaster_data_global) - changes the transaction paymaster data for all contracts
- [`start_cheat_paymaster_data`](cheatcodes/paymaster_data.md#start_cheat_paymaster_data) - changes the transaction paymaster data for contracts
- [`stop_cheat_paymaster_data`](cheatcodes/paymaster_data.md#stop_cheat_paymaster_data) - cancels the `cheat_paymaster_data` / `start_cheat_paymaster_data` for contracts
- [`stop_cheat_paymaster_data_global`](cheatcodes/paymaster_data.md#stop_cheat_paymaster_data_global) - cancels the `start_cheat_paymaster_data_global`

### Transaction Nonce Data Availability Mode

- [`cheat_nonce_data_availability_mode`](cheatcodes/nonce_data_availability_mode.md#cheat_nonce_data_availability_mode) - changes the transaction nonce data availability mode for contracts, for a number of calls
- [`start_cheat_nonce_data_availability_mode_global`](cheatcodes/nonce_data_availability_mode.md#start_cheat_nonce_data_availability_mode_global) - changes the transaction nonce data availability mode for all contracts
- [`start_cheat_nonce_data_availability_mode`](cheatcodes/nonce_data_availability_mode.md#start_cheat_nonce_data_availability_mode) - changes the transaction nonce data availability mode for contracts
- [`stop_cheat_nonce_data_availability_mode`](cheatcodes/nonce_data_availability_mode.md#stop_cheat_nonce_data_availability_mode) - cancels the `cheat_nonce_data_availability_mode` / `start_cheat_nonce_data_availability_mode` for contracts
- [`stop_cheat_nonce_data_availability_mode_global`](cheatcodes/nonce_data_availability_mode.md#stop_cheat_nonce_data_availability_mode_global) - cancels the `start_cheat_nonce_data_availability_mode_global`

### Transaction Fee Data Availability Mode

- [`cheat_fee_data_availability_mode`](cheatcodes/fee_data_availability_mode.md#cheat_fee_data_availability_mode) - changes the transaction fee data availability mode for contracts, for a number of calls
- [`start_cheat_fee_data_availability_mode_global`](cheatcodes/fee_data_availability_mode.md#start_cheat_fee_data_availability_mode_global) - changes the transaction fee data availability mode for all contracts
- [`start_cheat_fee_data_availability_mode`](cheatcodes/fee_data_availability_mode.md#start_cheat_fee_data_availability_mode) - changes the transaction fee data availability mode for contracts
- [`stop_cheat_fee_data_availability_mode`](cheatcodes/fee_data_availability_mode.md#stop_cheat_fee_data_availability_mode) - cancels the `cheat_fee_data_availability_mode` / `start_cheat_fee_data_availability_mode` for contracts
- [`stop_cheat_fee_data_availability_mode_global`](cheatcodes/fee_data_availability_mode.md#stop_cheat_fee_data_availability_mode_global) - cancels the `start_cheat_fee_data_availability_mode_global`

### Transaction Account Deployment

- [`cheat_account_deployment_data`](cheatcodes/account_deployment_data.md#cheat_account_deployment_data) - changes the transaction account deployment data for contracts, for a number of calls
- [`start_cheat_account_deployment_data_global`](cheatcodes/account_deployment_data.md#start_cheat_account_deployment_data_global) - changes the transaction account deployment data for all contracts
- [`start_cheat_account_deployment_data`](cheatcodes/account_deployment_data.md#start_cheat_account_deployment_data) - changes the transaction account deployment data for contracts
- [`stop_cheat_account_deployment_data`](cheatcodes/account_deployment_data.md#stop_cheat_account_deployment_data) - cancels the `cheat_account_deployment_data` / `start_cheat_account_deployment_data` for contracts
- [`stop_cheat_account_deployment_data_global`](cheatcodes/account_deployment_data.md#stop_cheat_account_deployment_data_global) - cancels the `start_cheat_account_deployment_data_global`

## Account Contract Address

- [`cheat_account_contract_address`](cheatcodes/account_contract_address.md#cheat_account_contract_address) - changes the address of an account which the transaction originates from, for the given target and span
- [`start_cheat_account_contract_address_global`](cheatcodes/account_contract_address.md#start_cheat_account_contract_address_global) - changes the address of an account which the transaction originates from, for all targets
- [`start_cheat_account_contract_address`](cheatcodes/account_contract_address.md#start_cheat_account_contract_address) - changes the address of an account which the transaction originates from, for the given target 
- [`stop_cheat_account_contract_address`](cheatcodes/account_deployment_data.md#stop_cheat_account_contract_address) - cancels the `cheat_account_deployment_data` / `start_cheat_account_deployment_data` for the given target
- [`stop_cheat_account_contract_address_global`](cheatcodes/account_deployment_data.md#stop_cheat_account_contract_address_global) - cancels the `start_cheat_account_contract_address_global`

> ℹ️ **Info**
> To use cheatcodes you need to add `snforge_std` package as a development dependency in
> your [`Scarb.toml`](https://docs.swmansion.com/scarb/docs/guides/dependencies.html#development-dependencies)
> using the appropriate version.
>
> ```toml
> [dev-dependencies]
> snforge_std = "0.33.0"
> ```
