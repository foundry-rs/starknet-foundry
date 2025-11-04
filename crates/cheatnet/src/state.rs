use crate::constants::build_test_entry_point;
use crate::forking::state::ForkStateReader;
use crate::predeployment::erc20::eth::eth_predeployed_contract;
use crate::predeployment::erc20::strk::strk_predeployed_contract;
use crate::predeployment::predeployed_contract::PredeployedContract;
use crate::runtime_extensions::forge_runtime_extension::cheatcodes::cheat_execution_info::{
    ExecutionInfoMock, ResourceBounds,
};
use crate::runtime_extensions::forge_runtime_extension::cheatcodes::spy_events::Event;
use crate::runtime_extensions::forge_runtime_extension::cheatcodes::spy_messages_to_l1::MessageToL1;
use crate::trace_data::{CallTrace, NotEmptyCallStack, TraceData};
use blockifier::execution::contract_class::RunnableCompiledClass;
use blockifier::execution::syscalls::vm_syscall_utils::{
    SyscallSelector, SyscallUsage, SyscallUsageMap,
};
use blockifier::state::errors::StateError::UndeclaredClassHash;
use blockifier::state::state_api::{StateReader, StateResult};
use cairo_vm::Felt252;
use cairo_vm::vm::runners::cairo_runner::ExecutionResources;
use conversions::serde::deserialize::CairoDeserialize;
use conversions::string::TryFromHexStr;
use indexmap::IndexMap;
use runtime::starknet::constants::TEST_CONTRACT_CLASS_HASH;
use runtime::starknet::context::SerializableBlockInfo;
use runtime::starknet::state::DictStateReader;
use starknet_api::block::BlockInfo;
use starknet_api::core::{ChainId, EntryPointSelector};
use starknet_api::transaction::fields::ContractAddressSalt;
use starknet_api::{
    core::{ClassHash, CompiledClassHash, ContractAddress, Nonce},
    state::StorageKey,
};
use starknet_types_core::felt::Felt;
use std::cell::RefCell;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::rc::Rc;

// Specifies the duration of the cheat
#[derive(CairoDeserialize, Copy, Clone, Debug, PartialEq, Eq)]
pub enum CheatSpan {
    Indefinite,
    TargetCalls(NonZeroUsize),
}

#[derive(Debug)]
pub struct ExtendedStateReader {
    pub dict_state_reader: DictStateReader,
    pub fork_state_reader: Option<ForkStateReader>,
}

impl ExtendedStateReader {
    pub fn predeploy_contracts(&mut self) {
        // We consider contract as deployed solely based on the fact that the test used forking
        let is_fork = self.fork_state_reader.is_some();
        if !is_fork {
            let contracts = vec![strk_predeployed_contract(), eth_predeployed_contract()];
            for contract in contracts {
                self.predeploy_contract(contract);
            }
        }
    }

    fn predeploy_contract(&mut self, contract: PredeployedContract) {
        let PredeployedContract {
            contract_address,
            class_hash,
            contract_class,
            storage_kv_updates,
        } = contract;
        self.dict_state_reader
            .address_to_class_hash
            .insert(contract_address, class_hash);

        self.dict_state_reader
            .class_hash_to_class
            .insert(class_hash, contract_class);

        for (key, value) in &storage_kv_updates {
            let entry = (contract_address, *key);
            self.dict_state_reader.storage_view.insert(entry, *value);
        }
    }
}

pub trait BlockInfoReader {
    fn get_block_info(&mut self) -> StateResult<BlockInfo>;
}

impl BlockInfoReader for ExtendedStateReader {
    fn get_block_info(&mut self) -> StateResult<BlockInfo> {
        if let Some(ref mut fork_state_reader) = self.fork_state_reader {
            return fork_state_reader.get_block_info();
        }

        Ok(SerializableBlockInfo::default().into())
    }
}

impl StateReader for ExtendedStateReader {
    fn get_storage_at(
        &self,
        contract_address: ContractAddress,
        key: StorageKey,
    ) -> StateResult<Felt> {
        self.dict_state_reader
            .get_storage_at(contract_address, key)
            .or_else(|_| {
                self.fork_state_reader
                    .as_ref()
                    .map_or(Ok(Felt252::default()), {
                        |reader| reader.get_storage_at(contract_address, key)
                    })
            })
    }

    fn get_nonce_at(&self, contract_address: ContractAddress) -> StateResult<Nonce> {
        self.dict_state_reader
            .get_nonce_at(contract_address)
            .or_else(|_| {
                self.fork_state_reader
                    .as_ref()
                    .map_or(Ok(Nonce::default()), {
                        |reader| reader.get_nonce_at(contract_address)
                    })
            })
    }

    fn get_class_hash_at(&self, contract_address: ContractAddress) -> StateResult<ClassHash> {
        self.dict_state_reader
            .get_class_hash_at(contract_address)
            .or_else(|_| {
                self.fork_state_reader
                    .as_ref()
                    .map_or(Ok(ClassHash::default()), {
                        |reader| reader.get_class_hash_at(contract_address)
                    })
            })
    }

    fn get_compiled_class(&self, class_hash: ClassHash) -> StateResult<RunnableCompiledClass> {
        self.dict_state_reader
            .get_compiled_class(class_hash)
            .or_else(|_| {
                self.fork_state_reader
                    .as_ref()
                    .map_or(Err(UndeclaredClassHash(class_hash)), |reader| {
                        reader.get_compiled_class(class_hash)
                    })
            })
    }

    fn get_compiled_class_hash(&self, class_hash: ClassHash) -> StateResult<CompiledClassHash> {
        Ok(self
            .dict_state_reader
            .get_compiled_class_hash(class_hash)
            .unwrap_or_default())
    }
}

impl ExtendedStateReader {
    pub fn get_chain_id(&self) -> anyhow::Result<Option<ChainId>> {
        self.fork_state_reader
            .as_ref()
            .map(ForkStateReader::chain_id)
            .transpose()
    }
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub enum CheatStatus<T> {
    Cheated(T, CheatSpan),
    #[default]
    Uncheated,
}

impl<T> CheatStatus<T> {
    pub fn decrement_cheat_span(&mut self) {
        if let CheatStatus::Cheated(_, CheatSpan::TargetCalls(n)) = self {
            let calls_number = n.get() - 1;

            if calls_number == 0 {
                *self = CheatStatus::Uncheated;
            } else {
                *n = NonZeroUsize::new(calls_number)
                    .expect("`NonZeroUsize` should not be zero after decrement");
            }
        }
    }

    pub fn as_value(&self) -> Option<T>
    where
        T: Clone,
    {
        match self {
            Self::Cheated(value, _span) => Some(value.clone()),
            Self::Uncheated => None,
        }
    }
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct CheatedTxInfo {
    pub version: Option<Felt>,
    pub account_contract_address: Option<Felt>,
    pub max_fee: Option<Felt>,
    pub signature: Option<Vec<Felt>>,
    pub transaction_hash: Option<Felt>,
    pub chain_id: Option<Felt>,
    pub nonce: Option<Felt>,
    pub resource_bounds: Option<Vec<ResourceBounds>>,
    pub tip: Option<Felt>,
    pub paymaster_data: Option<Vec<Felt>>,
    pub nonce_data_availability_mode: Option<Felt>,
    pub fee_data_availability_mode: Option<Felt>,
    pub account_deployment_data: Option<Vec<Felt>>,
}

impl CheatedTxInfo {
    #[must_use]
    pub fn is_mocked(&self) -> bool {
        self != &CheatedTxInfo::default()
    }
}

#[derive(Clone, Default, Debug)]
pub struct CheatedData {
    pub block_number: Option<u64>,
    pub block_timestamp: Option<u64>,
    pub caller_address: Option<ContractAddress>,
    pub contract_address: Option<ContractAddress>,
    pub sequencer_address: Option<ContractAddress>,
    pub tx_info: CheatedTxInfo,
}

pub struct CheatnetState {
    pub cheated_execution_info_contracts: HashMap<ContractAddress, ExecutionInfoMock>,
    pub global_cheated_execution_info: ExecutionInfoMock,

    pub mocked_functions:
        HashMap<ContractAddress, HashMap<EntryPointSelector, CheatStatus<Vec<Felt>>>>,
    pub replaced_bytecode_contracts: HashMap<ContractAddress, ClassHash>,
    pub detected_events: Vec<Event>,
    pub detected_messages_to_l1: Vec<MessageToL1>,
    pub deploy_salt_base: u32,
    pub next_deploy_at_address: Option<ContractAddress>,
    pub block_info: BlockInfo,
    pub trace_data: TraceData,
    pub encountered_errors: EncounteredErrors,
    pub fuzzer_args: Vec<String>,
    pub block_hash_contracts: HashMap<(ContractAddress, u64), (CheatSpan, Felt)>,
    pub global_block_hash: HashMap<u64, (Felt, Vec<ContractAddress>)>,
    pub used_resources: ExecutionResources,
    pub used_syscalls: SyscallUsageMap,
}

pub type EncounteredErrors = IndexMap<ClassHash, Vec<usize>>;

impl Default for CheatnetState {
    fn default() -> Self {
        let mut test_code_entry_point = build_test_entry_point();
        test_code_entry_point.class_hash =
            ClassHash(TryFromHexStr::try_from_hex_str(TEST_CONTRACT_CLASS_HASH).unwrap());
        let test_call = Rc::new(RefCell::new(CallTrace {
            entry_point: test_code_entry_point.into(),
            ..CallTrace::default_successful_call()
        }));
        Self {
            cheated_execution_info_contracts: HashMap::default(),
            global_cheated_execution_info: ExecutionInfoMock::default(),
            mocked_functions: HashMap::default(),
            replaced_bytecode_contracts: HashMap::default(),
            detected_events: vec![],
            detected_messages_to_l1: vec![],
            deploy_salt_base: 0,
            next_deploy_at_address: None,
            block_info: SerializableBlockInfo::default().into(),
            trace_data: TraceData {
                current_call_stack: NotEmptyCallStack::from(test_call),
                is_vm_trace_needed: false,
            },
            encountered_errors: IndexMap::default(),
            fuzzer_args: Vec::default(),
            block_hash_contracts: HashMap::default(),
            global_block_hash: HashMap::default(),
            used_resources: ExecutionResources::default(),
            used_syscalls: SyscallUsageMap::default(),
        }
    }
}

impl CheatnetState {
    #[must_use]
    pub fn create_cheated_data(&mut self, contract_address: ContractAddress) -> CheatedData {
        let execution_info = self.get_cheated_execution_info_for_contract(contract_address);

        CheatedData {
            block_number: execution_info.block_info.block_number.as_value(),
            block_timestamp: execution_info.block_info.block_timestamp.as_value(),
            caller_address: execution_info.caller_address.as_value(),
            contract_address: execution_info.contract_address.as_value(),
            sequencer_address: execution_info.block_info.sequencer_address.as_value(),
            tx_info: CheatedTxInfo {
                version: execution_info.tx_info.version.as_value(),
                account_contract_address: execution_info
                    .tx_info
                    .account_contract_address
                    .as_value(),
                max_fee: execution_info.tx_info.max_fee.as_value(),
                signature: execution_info.tx_info.signature.as_value(),
                transaction_hash: execution_info.tx_info.transaction_hash.as_value(),
                chain_id: execution_info.tx_info.chain_id.as_value(),
                nonce: execution_info.tx_info.nonce.as_value(),
                resource_bounds: execution_info.tx_info.resource_bounds.as_value(),
                tip: execution_info.tx_info.tip.as_value(),
                paymaster_data: execution_info.tx_info.paymaster_data.as_value(),
                nonce_data_availability_mode: execution_info
                    .tx_info
                    .nonce_data_availability_mode
                    .as_value(),
                fee_data_availability_mode: execution_info
                    .tx_info
                    .fee_data_availability_mode
                    .as_value(),
                account_deployment_data: execution_info.tx_info.account_deployment_data.as_value(),
            },
        }
    }

    pub fn get_cheated_data(&mut self, contract_address: ContractAddress) -> CheatedData {
        let current_call_stack = &mut self.trace_data.current_call_stack;

        // case of cheating the test address itself
        if current_call_stack.size() == 1 {
            self.create_cheated_data(contract_address)
            // do not update the cheats, as the test address cannot be called from the outside
        } else {
            current_call_stack.top_cheated_data()
        }
    }

    pub fn increment_deploy_salt_base(&mut self) {
        self.deploy_salt_base += 1;
    }

    pub fn set_next_deploy_at_address(&mut self, address: ContractAddress) {
        self.next_deploy_at_address = Some(address);
    }

    pub fn next_address_for_deployment(&mut self) -> Option<ContractAddress> {
        self.next_deploy_at_address.take()
    }

    #[must_use]
    pub fn get_salt(&self) -> ContractAddressSalt {
        ContractAddressSalt(Felt::from(self.deploy_salt_base))
    }

    #[must_use]
    pub fn get_cheated_block_number(&mut self, address: ContractAddress) -> Option<u64> {
        self.get_cheated_execution_info_for_contract(address)
            .block_info
            .block_number
            .as_value()
    }

    #[must_use]
    pub fn get_cheated_block_timestamp(&mut self, address: ContractAddress) -> Option<u64> {
        self.get_cheated_execution_info_for_contract(address)
            .block_info
            .block_timestamp
            .as_value()
    }

    #[must_use]
    pub fn get_cheated_sequencer_address(
        &mut self,
        address: ContractAddress,
    ) -> Option<ContractAddress> {
        self.get_cheated_execution_info_for_contract(address)
            .block_info
            .sequencer_address
            .as_value()
    }

    #[must_use]
    pub fn get_cheated_caller_address(
        &mut self,
        address: ContractAddress,
    ) -> Option<ContractAddress> {
        self.get_cheated_execution_info_for_contract(address)
            .caller_address
            .as_value()
    }

    pub fn update_cheats(&mut self, address: &ContractAddress) {
        self.progress_cheated_execution_info(*address);
    }

    pub fn update_fuzzer_args(&mut self, arg: String) {
        self.fuzzer_args.push(arg);
    }

    pub fn register_error(&mut self, class_hash: ClassHash, pcs: Vec<usize>) {
        self.encountered_errors.insert(class_hash, pcs);
    }

    pub fn clear_error(&mut self, class_hash: ClassHash) {
        self.encountered_errors.shift_remove(&class_hash);
    }

    pub fn add_used_resources(&mut self, resources: &ExecutionResources) {
        self.used_resources += resources;
    }

    pub fn add_used_syscall(&mut self, syscall: &SyscallSelector) {
        self.used_syscalls
            .entry(*syscall)
            .and_modify(|existing_usage| {
                existing_usage.call_count += 1;
            })
            .or_insert(SyscallUsage {
                call_count: 1,
                linear_factor: 0,
            });
    }
}
