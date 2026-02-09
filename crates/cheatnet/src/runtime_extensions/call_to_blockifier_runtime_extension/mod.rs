use std::marker::PhantomData;

use crate::state::CheatnetState;
use blockifier::execution::entry_point::{CallEntryPoint, CallType};
use blockifier::execution::syscalls::hint_processor::{
    OUT_OF_GAS_ERROR, SyscallHintProcessor, create_retdata_segment,
};
use blockifier::execution::syscalls::syscall_base::SyscallResult;
use blockifier::execution::syscalls::syscall_executor::SyscallExecutor;
use blockifier::execution::syscalls::vm_syscall_utils::{
    CallContractRequest, CallContractResponse, LibraryCallRequest, LibraryCallResponse, RevertData,
    SelfOrRevert, SingleSegmentResponse, SyscallExecutorBaseError, SyscallRequestWrapper,
    SyscallSelector,
};
use blockifier::execution::syscalls::vm_syscall_utils::{
    SyscallRequest, SyscallResponse, SyscallResponseWrapper,
};
use blockifier::utils::u64_from_usize;
use cairo_vm::vm::{errors::hint_errors::HintError, vm_core::VirtualMachine};
use runtime::{ExtendedRuntime, ExtensionLogic, SyscallHandlingResult};
use starknet_api::contract_class::EntryPointType;
use starknet_api::core::ContractAddress;
use starknet_api::execution_resources::GasAmount;
use starknet_types_core::felt::Felt;

use crate::runtime_extensions::call_to_blockifier_runtime_extension::rpc::{
    AddressOrClassHash, call_entry_point,
};

use super::cheatable_starknet_runtime_extension::{
    CheatableStarknetRuntime, CheatableStarknetRuntimeError,
};
use conversions::string::TryFromHexStr;
use runtime::starknet::constants::TEST_ADDRESS;

pub mod execution;
pub mod panic_parser;
pub mod rpc;

pub struct CallToBlockifierExtension<'a> {
    pub lifetime: &'a PhantomData<()>,
}

pub type CallToBlockifierRuntime<'a> = ExtendedRuntime<CallToBlockifierExtension<'a>>;

impl<'a> ExtensionLogic for CallToBlockifierExtension<'a> {
    type Runtime = CheatableStarknetRuntime<'a>;

    fn override_system_call(
        &mut self,
        selector: SyscallSelector,
        vm: &mut VirtualMachine,
        extended_runtime: &mut Self::Runtime,
    ) -> Result<SyscallHandlingResult, HintError> {
        // Warning: Do not add a default (`_`) arm here.
        // This match must remain exhaustive so that if a new syscall is introduced,
        // we will explicitly add support for it.
        match selector {
            // We execute contract calls and library calls with modified blockifier
            // This is redirected to drop ForgeRuntimeExtension
            // and to enable executing outer calls in tests as non-revertible.
            SyscallSelector::CallContract => {
                self.execute_syscall(vm, call_contract_syscall, selector, extended_runtime)?;

                Ok(SyscallHandlingResult::Handled)
            }
            SyscallSelector::LibraryCall => {
                self.execute_syscall(vm, library_call_syscall, selector, extended_runtime)?;

                Ok(SyscallHandlingResult::Handled)
            }
            SyscallSelector::DelegateCall
            | SyscallSelector::DelegateL1Handler
            | SyscallSelector::Deploy
            | SyscallSelector::EmitEvent
            | SyscallSelector::GetBlockHash
            | SyscallSelector::GetBlockNumber
            | SyscallSelector::GetBlockTimestamp
            | SyscallSelector::GetCallerAddress
            | SyscallSelector::GetClassHashAt
            | SyscallSelector::GetContractAddress
            | SyscallSelector::GetExecutionInfo
            | SyscallSelector::GetSequencerAddress
            | SyscallSelector::GetTxInfo
            | SyscallSelector::GetTxSignature
            | SyscallSelector::Keccak
            | SyscallSelector::KeccakRound
            | SyscallSelector::Sha256ProcessBlock
            | SyscallSelector::LibraryCallL1Handler
            | SyscallSelector::MetaTxV0
            | SyscallSelector::ReplaceClass
            | SyscallSelector::Secp256k1Add
            | SyscallSelector::Secp256k1GetPointFromX
            | SyscallSelector::Secp256k1GetXy
            | SyscallSelector::Secp256k1Mul
            | SyscallSelector::Secp256k1New
            | SyscallSelector::Secp256r1Add
            | SyscallSelector::Secp256r1GetPointFromX
            | SyscallSelector::Secp256r1GetXy
            | SyscallSelector::Secp256r1Mul
            | SyscallSelector::Secp256r1New
            | SyscallSelector::SendMessageToL1
            | SyscallSelector::StorageRead
            | SyscallSelector::StorageWrite => Ok(SyscallHandlingResult::Forwarded),
        }
    }
}

fn call_contract_syscall(
    request: CallContractRequest,
    vm: &mut VirtualMachine,
    syscall_handler: &mut SyscallHintProcessor,
    cheatnet_state: &mut CheatnetState,
    remaining_gas: &mut u64,
) -> SyscallResult<CallContractResponse> {
    let contract_address = request.contract_address;

    let entry_point = CallEntryPoint {
        class_hash: None,
        code_address: Some(contract_address),
        entry_point_type: EntryPointType::External,
        entry_point_selector: request.function_selector,
        calldata: request.calldata,
        storage_address: contract_address,
        caller_address: TryFromHexStr::try_from_hex_str(TEST_ADDRESS).unwrap(),
        call_type: CallType::Call,
        initial_gas: *remaining_gas,
    };

    let res = call_entry_point(
        syscall_handler,
        cheatnet_state,
        entry_point,
        &AddressOrClassHash::ContractAddress(contract_address),
        remaining_gas,
    )?;

    let segment = create_retdata_segment(vm, syscall_handler, &res.ret_data)?;
    Ok(CallContractResponse { segment })
}

fn library_call_syscall(
    request: LibraryCallRequest,
    vm: &mut VirtualMachine,
    syscall_handler: &mut SyscallHintProcessor,
    cheatnet_state: &mut CheatnetState,
    remaining_gas: &mut u64,
) -> SyscallResult<LibraryCallResponse> {
    let class_hash = request.class_hash;

    let entry_point = CallEntryPoint {
        class_hash: Some(class_hash),
        code_address: None,
        entry_point_type: EntryPointType::External,
        entry_point_selector: request.function_selector,
        calldata: request.calldata,
        storage_address: TryFromHexStr::try_from_hex_str(TEST_ADDRESS).unwrap(),
        caller_address: ContractAddress::default(),
        call_type: CallType::Delegate,
        initial_gas: *remaining_gas,
    };

    let res = call_entry_point(
        syscall_handler,
        cheatnet_state,
        entry_point,
        &AddressOrClassHash::ClassHash(class_hash),
        remaining_gas,
    )?;

    let segment = create_retdata_segment(vm, syscall_handler, &res.ret_data)?;
    Ok(LibraryCallResponse { segment })
}

impl CallToBlockifierExtension<'_> {
    // crates/blockifier/src/execution/syscalls/vm_syscall_utils.rs:677 (execute_syscall)
    #[expect(clippy::unused_self)]
    fn execute_syscall<Request, Response, ExecuteCallback, Error>(
        &mut self,
        vm: &mut VirtualMachine,
        execute_callback: ExecuteCallback,
        selector: SyscallSelector,
        extended_runtime: &mut CheatableStarknetRuntime,
    ) -> Result<(), Error>
    where
        Request: SyscallRequest + std::fmt::Debug,
        Response: SyscallResponse + std::fmt::Debug,
        Error: CheatableStarknetRuntimeError,
        ExecuteCallback: FnOnce(
            Request,
            &mut VirtualMachine,
            &mut SyscallHintProcessor<'_>,
            &mut CheatnetState,
            &mut u64, // Remaining gas.
        ) -> Result<Response, Error>,
    {
        // region: Modified blockifier code
        let syscall_handler = &mut extended_runtime.extended_runtime.hint_handler;
        let cheatnet_state = &mut *extended_runtime.extension.cheatnet_state;

        // Increment, since the selector was peeked into before
        syscall_handler.syscall_ptr += 1;
        syscall_handler.increment_syscall_count_by(&selector, 1);
        // endregion

        let syscall_gas_cost = syscall_handler
            .get_gas_cost_from_selector(&selector)
            .map_err(|error| SyscallExecutorBaseError::GasCost { error, selector })?;

        let SyscallRequestWrapper {
            gas_counter,
            request,
        } = SyscallRequestWrapper::<Request>::read(vm, syscall_handler.get_mut_syscall_ptr())?;

        let syscall_gas_cost =
            syscall_gas_cost.get_syscall_cost(u64_from_usize(request.get_linear_factor_length()));
        let syscall_base_cost = syscall_handler.get_syscall_base_gas_cost();

        // Sanity check for preventing underflow.
        assert!(
            syscall_gas_cost >= syscall_base_cost,
            "Syscall gas cost must be greater than base syscall gas cost"
        );

        // Refund `SYSCALL_BASE_GAS_COST` as it was pre-charged.
        // Note: It is pre-charged by the compiler: https://github.com/starkware-libs/sequencer/blob/v0.15.0-rc.2/crates/blockifier/src/blockifier_versioned_constants.rs#L1057
        let required_gas = syscall_gas_cost - syscall_base_cost;

        if gas_counter < required_gas {
            let out_of_gas_error =
                Felt::from_hex(OUT_OF_GAS_ERROR).map_err(SyscallExecutorBaseError::from)?;
            let response: SyscallResponseWrapper<SingleSegmentResponse> =
                SyscallResponseWrapper::Failure {
                    gas_counter,
                    revert_data: RevertData::new_normal(vec![out_of_gas_error]),
                };
            response.write(vm, syscall_handler.get_mut_syscall_ptr())?;

            return Ok(());
        }

        let mut remaining_gas = gas_counter - required_gas;

        // TODO(#3681)
        syscall_handler.update_revert_gas_with_next_remaining_gas(GasAmount(remaining_gas));

        let original_response = execute_callback(
            request,
            vm,
            syscall_handler,
            cheatnet_state,
            &mut remaining_gas,
        );

        let response = match original_response {
            Ok(response) => SyscallResponseWrapper::Success {
                gas_counter: remaining_gas,
                response,
            },
            Err(error) => match error.try_extract_revert() {
                SelfOrRevert::Revert(data) => SyscallResponseWrapper::Failure {
                    gas_counter: remaining_gas,
                    revert_data: data,
                },
                SelfOrRevert::Original(err) => return Err(err),
            },
        };

        response.write(vm, &mut syscall_handler.syscall_ptr)?;

        Ok(())
    }
}
