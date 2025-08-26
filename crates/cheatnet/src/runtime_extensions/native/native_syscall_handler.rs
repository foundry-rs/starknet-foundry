use crate::runtime_extensions::forge_runtime_extension::cheatcodes::spy_events::Event;
use crate::runtime_extensions::forge_runtime_extension::cheatcodes::spy_messages_to_l1::MessageToL1;
use crate::runtime_extensions::native::call::execute_inner_call;
use crate::runtime_extensions::native::deploy::deploy;
use crate::state::CheatnetState;
use blockifier::execution::call_info::Retdata;
use blockifier::execution::common_hints::ExecutionMode;
use blockifier::execution::entry_point::{CallEntryPoint, CallType};
use blockifier::execution::errors::EntryPointExecutionError;
use blockifier::execution::native::syscall_handler::NativeSyscallHandler;
use blockifier::execution::syscalls::hint_processor::{OUT_OF_GAS_ERROR, SyscallExecutionError};
use blockifier::execution::syscalls::vm_syscall_utils::{
    SelfOrRevert, SyscallExecutorBaseError, SyscallSelector, TryExtractRevert,
};
use blockifier::utils::u64_from_usize;
use cairo_native::starknet::{
    BlockInfo, ExecutionInfo, ExecutionInfoV2, ResourceBounds, Secp256k1Point, Secp256r1Point,
    StarknetSyscallHandler, SyscallResult, TxV2Info, U256,
};
use num_traits::ToPrimitive;
use starknet_api::contract_class::EntryPointType;
use starknet_api::core::{ClassHash, ContractAddress, EntryPointSelector};
use starknet_api::execution_resources::GasAmount;
use starknet_api::transaction::fields::{Calldata, ContractAddressSalt};
use starknet_types_core::felt::Felt;
use std::sync::Arc;

pub struct CheatableNativeSyscallHandler<'a> {
    pub native_syscall_handler: &'a mut NativeSyscallHandler<'a>,
    pub cheatnet_state: &'a mut CheatnetState,
}

pub type BaseSyscallResult<T> = Result<T, SyscallExecutionError>;

impl CheatableNativeSyscallHandler<'_> {
    // TODO consider modifying this so it doesn't use take
    pub fn unrecoverable_error(&mut self) -> Option<SyscallExecutionError> {
        self.native_syscall_handler.unrecoverable_error.take()
    }

    /// Handles all gas-related logics, syscall usage counting and perform additional checks. In
    /// native, we need to explicitly call this method at the beginning of each syscall.
    #[allow(clippy::result_large_err)]
    fn pre_execute_syscall(
        &mut self,
        remaining_gas: &mut u64,
        total_gas_cost: u64,
        selector: SyscallSelector,
    ) -> SyscallResult<()> {
        if self.native_syscall_handler.unrecoverable_error.is_some() {
            // An unrecoverable error was found in a previous syscall, we return immediately to
            // accelerate the end of the execution. The returned data is not important
            return Err(vec![]);
        }

        // Keccak syscall usages' increments are handled inside its implementation.
        if !matches!(selector, SyscallSelector::Keccak) {
            self.native_syscall_handler
                .base
                .increment_syscall_count_by(selector, 1);
        }

        // Refund `SYSCALL_BASE_GAS_COST` as it was pre-charged.
        let required_gas = total_gas_cost
            - self
                .native_syscall_handler
                .gas_costs()
                .base
                .syscall_base_gas_cost;

        if *remaining_gas < required_gas {
            // Out of gas failure.
            return Err(vec![
                Felt::from_hex(OUT_OF_GAS_ERROR)
                    .expect("Failed to parse OUT_OF_GAS_ERROR hex string"),
            ]);
        }

        *remaining_gas -= required_gas;

        // To support sierra gas charge for blockifier revert flow, we track the remaining gas left
        // before executing a syscall if the current tracked resource is gas.
        // 1. If the syscall does not run Cairo code (i.e. not library call, not call contract, and
        //    not a deploy), any failure will not run in the OS, so no need to charge - the value
        //    before entering the callback is good enough to charge.
        // 2. If the syscall runs Cairo code, but the tracked resource is steps (and not gas), the
        //    additional charge of reverted cairo steps will cover the inner cost, and the outer
        //    cost we track here will be the additional reverted gas.
        // 3. If the syscall runs Cairo code and the tracked resource is gas, either the inner
        //    failure will be a Cairo1 revert (and the gas consumed on the call info will override
        //    the current tracked value), or we will pass through another syscall before failing -
        //    and by induction (we will reach this point again), the gas will be charged correctly.
        self.native_syscall_handler
            .base
            .context
            .update_revert_gas_with_next_remaining_gas(GasAmount(*remaining_gas));

        Ok(())
    }

    // Copied from blockifer/src/exection/native/syscall_handler.rs
    #[allow(clippy::result_large_err)]
    fn execute_inner_call(
        &mut self,
        entry_point: &mut CallEntryPoint,
        remaining_gas: &mut u64,
        class_hash: ClassHash,
        error_wrapper_fn: impl Fn(
            SyscallExecutionError,
            ClassHash,
            ContractAddress,
            EntryPointSelector,
        ) -> SyscallExecutionError,
    ) -> SyscallResult<Retdata> {
        let entry_point_clone = entry_point.clone();
        // region: Modified blockifier code
        let raw_data = execute_inner_call(
            &mut self.native_syscall_handler.base,
            self.cheatnet_state,
            entry_point,
            remaining_gas,
        )
        .map_err(|e| {
            self.handle_error(
                remaining_gas,
                SyscallExecutionError::from_self_or_revert(e.try_extract_revert().map_original(
                    |error| {
                        error_wrapper_fn(
                            error,
                            class_hash,
                            entry_point_clone.storage_address,
                            entry_point_clone.entry_point_selector,
                        )
                    },
                )),
            )
        })?;
        // endregion
        Ok(Retdata(raw_data))
    }

    fn handle_error(&mut self, remaining_gas: &mut u64, error: SyscallExecutionError) -> Vec<Felt> {
        // In case of more than one inner call and because each inner call has their own
        // syscall handler, if there is an unrecoverable error at call `n` it will create a
        // `NativeExecutionError`. When rolling back, each call from `n-1` to `1` will also
        // store the result of a previous `NativeExecutionError` in a `NativeExecutionError`
        // creating multiple wraps around the same error. This function is meant to prevent that.
        fn unwrap_native_error(error: SyscallExecutionError) -> SyscallExecutionError {
            match error {
                SyscallExecutionError::EntryPointExecutionError(
                    EntryPointExecutionError::NativeUnrecoverableError(e),
                ) => *e,
                _ => error,
            }
        }

        match error.try_extract_revert() {
            SelfOrRevert::Revert(revert_error) => revert_error.error_data,
            SelfOrRevert::Original(error) => {
                assert!(
                    self.native_syscall_handler.unrecoverable_error.is_none(),
                    "Trying to set an unrecoverable error twice in Native Syscall Handler"
                );
                self.native_syscall_handler.unrecoverable_error = Some(unwrap_native_error(error));
                *remaining_gas = 0;
                vec![]
            }
        }
    }
}

impl StarknetSyscallHandler for &mut CheatableNativeSyscallHandler<'_> {
    fn get_block_hash(
        &mut self,
        block_number: u64,
        remaining_gas: &mut u64,
    ) -> SyscallResult<Felt> {
        self.pre_execute_syscall(
            remaining_gas,
            self.native_syscall_handler
                .gas_costs()
                .syscalls
                .get_block_hash
                .base_syscall_cost(),
            SyscallSelector::GetBlockHash,
        )?;

        let block_hash = self.cheatnet_state.get_cheated_block_hash_for_contract(
            self.native_syscall_handler.base.call.storage_address,
            block_number,
        );

        if let Some(block_hash) = block_hash {
            Ok(block_hash.0)
        } else {
            match self
                .native_syscall_handler
                .base
                .get_block_hash(block_number)
            {
                Ok(value) => Ok(value),
                Err(e) => Err(self.handle_error(remaining_gas, e)),
            }
        }
    }

    fn get_execution_info(&mut self, remaining_gas: &mut u64) -> SyscallResult<ExecutionInfo> {
        self.native_syscall_handler
            .get_execution_info(remaining_gas)
    }

    #[expect(clippy::too_many_lines)]
    fn get_execution_info_v2(&mut self, remaining_gas: &mut u64) -> SyscallResult<ExecutionInfoV2> {
        self.pre_execute_syscall(
            remaining_gas,
            self.native_syscall_handler
                .gas_costs()
                .syscalls
                .get_execution_info
                .base_syscall_cost(),
            SyscallSelector::GetBlockHash,
        )?;

        let original_data = self
            .native_syscall_handler
            .get_execution_info_v2(remaining_gas)?;

        let cheated_data = self
            .cheatnet_state
            .get_cheated_data(self.native_syscall_handler.base.call.storage_address);

        let block_number = cheated_data
            .block_number
            .unwrap_or(original_data.block_info.block_number);
        let block_timestamp = cheated_data
            .block_timestamp
            .unwrap_or(original_data.block_info.block_timestamp);
        let sequencer_address = cheated_data.sequencer_address.map_or(
            original_data.block_info.sequencer_address,
            std::convert::Into::into,
        );

        let version = cheated_data
            .tx_info
            .version
            .unwrap_or(original_data.tx_info.version);
        let account_contract_address = cheated_data
            .tx_info
            .account_contract_address
            .unwrap_or(original_data.tx_info.account_contract_address);
        let max_fee = cheated_data
            .tx_info
            .max_fee
            .map_or(original_data.tx_info.max_fee, |max_fee| {
                max_fee.to_u128().unwrap()
            });
        let signature = cheated_data
            .tx_info
            .signature
            .unwrap_or(original_data.tx_info.signature);
        let transaction_hash = cheated_data
            .tx_info
            .transaction_hash
            .unwrap_or(original_data.tx_info.transaction_hash);
        let chain_id = cheated_data
            .tx_info
            .chain_id
            .unwrap_or(original_data.tx_info.chain_id);
        let nonce = cheated_data
            .tx_info
            .nonce
            .unwrap_or(original_data.tx_info.nonce);
        // TODO impl conversions
        let resource_bounds = cheated_data.tx_info.resource_bounds.map_or(
            original_data.tx_info.resource_bounds,
            |rb| {
                rb.iter()
                    .map(|item| ResourceBounds {
                        resource: item.resource,
                        max_amount: item.max_amount,
                        max_price_per_unit: item.max_price_per_unit,
                    })
                    .collect()
            },
        );
        let tip = cheated_data
            .tx_info
            .tip
            .map_or(original_data.tx_info.tip, |tip| tip.to_u128().unwrap());
        let paymaster_data = cheated_data
            .tx_info
            .paymaster_data
            .unwrap_or(original_data.tx_info.paymaster_data);
        let nonce_data_availability_mode = cheated_data
            .tx_info
            .nonce_data_availability_mode
            .map_or(original_data.tx_info.nonce_data_availability_mode, |mode| {
                mode.to_u32().unwrap()
            });
        let fee_data_availability_mode = cheated_data
            .tx_info
            .fee_data_availability_mode
            .map_or(original_data.tx_info.fee_data_availability_mode, |mode| {
                mode.to_u32().unwrap()
            });
        let account_deployment_data = cheated_data
            .tx_info
            .account_deployment_data
            .unwrap_or(original_data.tx_info.account_deployment_data);

        let caller_address = cheated_data
            .caller_address
            .map_or(original_data.caller_address, std::convert::Into::into);
        let contract_address = cheated_data
            .contract_address
            .map_or(original_data.contract_address, std::convert::Into::into);
        let entry_point_selector = original_data.entry_point_selector;

        Ok(ExecutionInfoV2 {
            block_info: BlockInfo {
                block_number,
                block_timestamp,
                sequencer_address,
            },
            tx_info: TxV2Info {
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
            },
            caller_address,
            contract_address,
            entry_point_selector,
        })
    }

    fn deploy(
        &mut self,
        class_hash: Felt,
        contract_address_salt: Felt,
        calldata: &[Felt],
        deploy_from_zero: bool,
        remaining_gas: &mut u64,
    ) -> SyscallResult<(Felt, Vec<Felt>)> {
        // The cost of deploying a contract is the base cost plus the linear cost of the calldata
        // len.
        let total_gas_cost = self
            .native_syscall_handler
            .gas_costs()
            .syscalls
            .deploy
            .get_syscall_cost(u64_from_usize(calldata.len()));

        self.pre_execute_syscall(remaining_gas, total_gas_cost, SyscallSelector::Deploy)?;

        let (deployed_contract_address, call_info) = deploy(
            &mut self.native_syscall_handler.base,
            self.cheatnet_state,
            ClassHash(class_hash),
            ContractAddressSalt(contract_address_salt),
            Calldata(Arc::new(calldata.to_vec())),
            deploy_from_zero,
            remaining_gas,
        )
        .map_err(|err| self.handle_error(remaining_gas, err))?;

        let constructor_retdata = call_info.execution.retdata.0[..].to_vec();
        self.native_syscall_handler.base.inner_calls.push(call_info);

        Ok((Felt::from(deployed_contract_address), constructor_retdata))
    }

    fn replace_class(&mut self, class_hash: Felt, remaining_gas: &mut u64) -> SyscallResult<()> {
        self.native_syscall_handler
            .replace_class(class_hash, remaining_gas)
    }

    // Copied from blockifier/src/execution/native/syscall_handler.rs
    fn library_call(
        &mut self,
        class_hash: Felt,
        function_selector: Felt,
        calldata: &[Felt],
        remaining_gas: &mut u64,
    ) -> SyscallResult<Vec<Felt>> {
        self.pre_execute_syscall(
            remaining_gas,
            self.native_syscall_handler
                .gas_costs()
                .syscalls
                .library_call
                .base_syscall_cost(),
            SyscallSelector::LibraryCall,
        )?;

        let class_hash = ClassHash(class_hash);

        let wrapper_calldata = Calldata(Arc::new(calldata.to_vec()));

        let selector = EntryPointSelector(function_selector);

        let mut entry_point = CallEntryPoint {
            class_hash: Some(class_hash),
            code_address: None,
            entry_point_type: EntryPointType::External,
            entry_point_selector: selector,
            calldata: wrapper_calldata,
            // The call context remains the same in a library call.
            storage_address: self.native_syscall_handler.base.call.storage_address,
            caller_address: self.native_syscall_handler.base.call.caller_address,
            call_type: CallType::Delegate,
            initial_gas: *remaining_gas,
        };

        let error_wrapper_function =
            |e: SyscallExecutionError,
             class_hash: ClassHash,
             storage_address: ContractAddress,
             selector: EntryPointSelector| {
                e.as_lib_call_execution_error(class_hash, storage_address, selector)
            };

        Ok(self
            .execute_inner_call(
                &mut entry_point,
                remaining_gas,
                class_hash,
                error_wrapper_function,
            )?
            .0)
    }

    // Copied from blockifier/src/execution/native/syscall_handler.rs
    fn call_contract(
        &mut self,
        address: Felt,
        entry_point_selector: Felt,
        calldata: &[Felt],
        remaining_gas: &mut u64,
    ) -> SyscallResult<Vec<Felt>> {
        self.pre_execute_syscall(
            remaining_gas,
            self.native_syscall_handler
                .gas_costs()
                .syscalls
                .call_contract
                .base_syscall_cost(),
            SyscallSelector::CallContract,
        )?;

        let contract_address = ContractAddress::try_from(address)
            .map_err(|error| self.handle_error(remaining_gas, error.into()))?;

        let class_hash = self
            .native_syscall_handler
            .base
            .state
            .get_class_hash_at(contract_address)
            .map_err(|e| self.handle_error(remaining_gas, e.into()))?;
        if self.native_syscall_handler.base.context.execution_mode == ExecutionMode::Validate
            && self.native_syscall_handler.base.call.storage_address != contract_address
        {
            let err = SyscallExecutorBaseError::InvalidSyscallInExecutionMode {
                syscall_name: "call_contract".to_string(),
                execution_mode: self.native_syscall_handler.base.context.execution_mode,
            };
            return Err(self.handle_error(remaining_gas, err.into()));
        }
        let selector = EntryPointSelector(entry_point_selector);
        // TODO restore blocking
        // self
        //     .native_syscall_handler
        //     .base
        //     .maybe_block_direct_execute_call(selector)
        //     .map_err(|e| self.handle_error(remaining_gas, e))?;

        let wrapper_calldata = Calldata(Arc::new(calldata.to_vec()));

        let mut entry_point = CallEntryPoint {
            class_hash: None,
            code_address: Some(contract_address),
            entry_point_type: EntryPointType::External,
            entry_point_selector: selector,
            calldata: wrapper_calldata,
            storage_address: contract_address,
            caller_address: self.native_syscall_handler.base.call.storage_address,
            call_type: CallType::Call,
            initial_gas: *remaining_gas,
        };

        let error_wrapper_function =
            |e: SyscallExecutionError,
             class_hash: ClassHash,
             storage_address: ContractAddress,
             selector: EntryPointSelector| {
                e.as_call_contract_execution_error(class_hash, storage_address, selector)
            };

        Ok(self
            .execute_inner_call(
                &mut entry_point,
                remaining_gas,
                class_hash,
                error_wrapper_function,
            )?
            .0)
    }

    fn storage_read(
        &mut self,
        address_domain: u32,
        address: Felt,
        remaining_gas: &mut u64,
    ) -> SyscallResult<Felt> {
        self.native_syscall_handler
            .storage_read(address_domain, address, remaining_gas)
    }

    fn storage_write(
        &mut self,
        address_domain: u32,
        address: Felt,
        value: Felt,
        remaining_gas: &mut u64,
    ) -> SyscallResult<()> {
        self.native_syscall_handler
            .storage_write(address_domain, address, value, remaining_gas)
    }

    fn emit_event(
        &mut self,
        keys: &[Felt],
        data: &[Felt],
        remaining_gas: &mut u64,
    ) -> SyscallResult<()> {
        let syscall_result = self
            .native_syscall_handler
            .emit_event(keys, data, remaining_gas);

        if syscall_result.is_ok() {
            let contract_address = self
                .native_syscall_handler
                .base
                .call
                // TODO why we default to code_address??
                .code_address
                .unwrap_or(self.native_syscall_handler.base.call.storage_address);
            let event = self
                .native_syscall_handler
                .base
                .events
                .last()
                .expect("Event must have been emitted");
            self.cheatnet_state
                .detected_events
                .push(Event::from_ordered_event(event, contract_address));
        }

        syscall_result
    }

    fn send_message_to_l1(
        &mut self,
        to_address: Felt,
        payload: &[Felt],
        remaining_gas: &mut u64,
    ) -> SyscallResult<()> {
        let syscall_result =
            self.native_syscall_handler
                .send_message_to_l1(to_address, payload, remaining_gas);

        if syscall_result.is_ok() {
            let contract_address = self
                .native_syscall_handler
                .base
                .call
                // TODO why we default to code_address??
                .code_address
                .unwrap_or(self.native_syscall_handler.base.call.storage_address);
            let message = self
                .native_syscall_handler
                .base
                .l2_to_l1_messages
                .last()
                .expect("Message must have been sent");
            self.cheatnet_state
                .detected_messages_to_l1
                .push(MessageToL1::from_ordered_message(message, contract_address));
        }

        syscall_result
    }

    fn keccak(&mut self, input: &[u64], remaining_gas: &mut u64) -> SyscallResult<U256> {
        self.native_syscall_handler.keccak(input, remaining_gas)
    }

    fn secp256k1_new(
        &mut self,
        x: U256,
        y: U256,
        remaining_gas: &mut u64,
    ) -> SyscallResult<Option<Secp256k1Point>> {
        self.native_syscall_handler
            .secp256k1_new(x, y, remaining_gas)
    }

    fn secp256k1_add(
        &mut self,
        p0: Secp256k1Point,
        p1: Secp256k1Point,
        remaining_gas: &mut u64,
    ) -> SyscallResult<Secp256k1Point> {
        self.native_syscall_handler
            .secp256k1_add(p0, p1, remaining_gas)
    }

    fn secp256k1_mul(
        &mut self,
        p: Secp256k1Point,
        m: U256,
        remaining_gas: &mut u64,
    ) -> SyscallResult<Secp256k1Point> {
        self.native_syscall_handler
            .secp256k1_mul(p, m, remaining_gas)
    }

    fn secp256k1_get_point_from_x(
        &mut self,
        x: U256,
        y_parity: bool,
        remaining_gas: &mut u64,
    ) -> SyscallResult<Option<Secp256k1Point>> {
        self.native_syscall_handler
            .secp256k1_get_point_from_x(x, y_parity, remaining_gas)
    }

    fn secp256k1_get_xy(
        &mut self,
        p: Secp256k1Point,
        remaining_gas: &mut u64,
    ) -> SyscallResult<(U256, U256)> {
        self.native_syscall_handler
            .secp256k1_get_xy(p, remaining_gas)
    }

    fn secp256r1_new(
        &mut self,
        x: U256,
        y: U256,
        remaining_gas: &mut u64,
    ) -> SyscallResult<Option<Secp256r1Point>> {
        self.native_syscall_handler
            .secp256r1_new(x, y, remaining_gas)
    }

    fn secp256r1_add(
        &mut self,
        p0: Secp256r1Point,
        p1: Secp256r1Point,
        remaining_gas: &mut u64,
    ) -> SyscallResult<Secp256r1Point> {
        self.native_syscall_handler
            .secp256r1_add(p0, p1, remaining_gas)
    }

    fn secp256r1_mul(
        &mut self,
        p: Secp256r1Point,
        m: U256,
        remaining_gas: &mut u64,
    ) -> SyscallResult<Secp256r1Point> {
        self.native_syscall_handler
            .secp256r1_mul(p, m, remaining_gas)
    }

    fn secp256r1_get_point_from_x(
        &mut self,
        x: U256,
        y_parity: bool,
        remaining_gas: &mut u64,
    ) -> SyscallResult<Option<Secp256r1Point>> {
        self.native_syscall_handler
            .secp256r1_get_point_from_x(x, y_parity, remaining_gas)
    }

    fn secp256r1_get_xy(
        &mut self,
        p: Secp256r1Point,
        remaining_gas: &mut u64,
    ) -> SyscallResult<(U256, U256)> {
        self.native_syscall_handler
            .secp256r1_get_xy(p, remaining_gas)
    }

    fn sha256_process_block(
        &mut self,
        state: &mut [u32; 8],
        block: &[u32; 16],
        remaining_gas: &mut u64,
    ) -> SyscallResult<()> {
        self.native_syscall_handler
            .sha256_process_block(state, block, remaining_gas)
    }

    fn get_class_hash_at(
        &mut self,
        contract_address: Felt,
        remaining_gas: &mut u64,
    ) -> SyscallResult<Felt> {
        self.native_syscall_handler
            .get_class_hash_at(contract_address, remaining_gas)
    }

    fn meta_tx_v0(
        &mut self,
        address: Felt,
        entry_point_selector: Felt,
        calldata: &[Felt],
        signature: &[Felt],
        remaining_gas: &mut u64,
    ) -> SyscallResult<Vec<Felt>> {
        self.native_syscall_handler.meta_tx_v0(
            address,
            entry_point_selector,
            calldata,
            signature,
            remaining_gas,
        )
    }
}
