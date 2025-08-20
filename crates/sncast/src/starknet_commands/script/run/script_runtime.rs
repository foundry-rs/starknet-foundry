use blockifier::execution::syscalls::vm_syscall_utils::SyscallSelector;
use cairo_lang_casm::hints::{ExternalHint, Hint};
use cairo_lang_runner::Arg;
use cairo_lang_runner::casm_run::{cell_ref_to_relocatable, extract_relocatable, get_val};
use cairo_native::starknet::StarknetSyscallHandler;
use cairo_vm::hint_processor::hint_processor_definition::{HintProcessorLogic, HintReference};
use cairo_vm::serde::deserialize_program::ApTracking;
use cairo_vm::types::exec_scope::ExecutionScopes;
use cairo_vm::types::relocatable::Relocatable;
use cairo_vm::vm::errors::hint_errors::HintError;
use cairo_vm::vm::errors::vm_errors::VirtualMachineError;
use cairo_vm::vm::runners::cairo_runner::{ResourceTracker, RunResources};
use cairo_vm::vm::vm_core::VirtualMachine;
use num_traits::ToPrimitive;
use runtime::{SignalPropagator, StarknetRuntime, SyscallPtrAccess};
use starknet_types_core::felt::Felt;
use std::any::Any;
use std::collections::HashMap;

pub struct CastScriptRuntime<'a> {
    pub starknet_runtime: StarknetRuntime<'a>,
    pub user_args: Vec<Vec<Arg>>,
}

impl SyscallPtrAccess for CastScriptRuntime<'_> {
    fn get_mut_syscall_ptr(&mut self) -> &mut Relocatable {
        self.starknet_runtime.get_mut_syscall_ptr()
    }
}

impl ResourceTracker for CastScriptRuntime<'_> {
    fn consumed(&self) -> bool {
        self.starknet_runtime.consumed()
    }

    fn consume_step(&mut self) {
        self.starknet_runtime.consume_step();
    }

    fn get_n_steps(&self) -> Option<usize> {
        self.starknet_runtime.get_n_steps()
    }

    fn run_resources(&self) -> &RunResources {
        self.starknet_runtime.run_resources()
    }
}

impl SignalPropagator for CastScriptRuntime<'_> {
    fn propagate_system_call_signal(&mut self, selector: SyscallSelector, vm: &mut VirtualMachine) {
        self.starknet_runtime
            .propagate_system_call_signal(selector, vm);
    }

    fn propagate_cheatcode_signal(&mut self, selector: &str, inputs: &[Felt]) {
        self.starknet_runtime
            .propagate_cheatcode_signal(selector, inputs);
    }
}

impl HintProcessorLogic for CastScriptRuntime<'_> {
    fn execute_hint(
        &mut self,
        vm: &mut VirtualMachine,
        exec_scopes: &mut ExecutionScopes,
        hint_data: &Box<dyn Any>,
        constants: &HashMap<String, Felt>,
    ) -> Result<(), HintError> {
        let maybe_extended_hint = hint_data.downcast_ref::<Hint>();

        match maybe_extended_hint {
            // Copied from https://github.com/starkware-libs/cairo/blob/ada0450439a36d756223fba88dfd1f266f428f0c/crates/cairo-lang-runner/src/casm_run/mod.rs#L1321
            Some(Hint::External(ExternalHint::AddRelocationRule { src, dst })) => {
                vm.add_relocation_rule(
                    extract_relocatable(vm, src)?,
                    extract_relocatable(vm, dst)?,
                )?;
                Ok(())
            }
            // Copied from https://github.com/starkware-libs/cairo/blob/ada0450439a36d756223fba88dfd1f266f428f0c/crates/cairo-lang-runner/src/casm_run/mod.rs#L1330
            Some(Hint::External(ExternalHint::WriteRunParam { index, dst })) => {
                let index = get_val(vm, index)?.to_usize().expect("Got a bad index.");
                let mut stack = vec![(cell_ref_to_relocatable(dst, vm), &self.user_args[index])];
                while let Some((mut buffer, values)) = stack.pop() {
                    for value in values {
                        match value {
                            Arg::Value(v) => {
                                vm.insert_value(buffer, v)?;
                                buffer += 1;
                            }
                            Arg::Array(arr) => {
                                let arr_buffer = vm.add_memory_segment();
                                stack.push((arr_buffer, arr));
                                vm.insert_value(buffer, arr_buffer)?;
                                buffer += 1;
                                vm.insert_value(buffer, (arr_buffer + args_size(arr))?)?;
                                buffer += 1;
                            }
                        }
                    }
                }
                Ok(())
            }
            _ => self
                .starknet_runtime
                .execute_hint(vm, exec_scopes, hint_data, constants),
        }
    }

    fn compile_hint(
        &self,
        hint_code: &str,
        ap_tracking_data: &ApTracking,
        reference_ids: &HashMap<String, usize>,
        references: &[HintReference],
    ) -> Result<Box<dyn Any>, VirtualMachineError> {
        self.starknet_runtime
            .compile_hint(hint_code, ap_tracking_data, reference_ids, references)
    }
}

impl<'a> StarknetSyscallHandler for CastScriptRuntime<'a> {
    fn get_block_hash(
        &mut self,
        _block_number: u64,
        _remaining_gas: &mut u64,
    ) -> cairo_native::starknet::SyscallResult<Felt> {
        todo!()
    }

    fn get_execution_info(
        &mut self,
        _remaining_gas: &mut u64,
    ) -> cairo_native::starknet::SyscallResult<cairo_native::starknet::ExecutionInfo> {
        todo!()
    }

    fn get_execution_info_v2(
        &mut self,
        _remaining_gas: &mut u64,
    ) -> cairo_native::starknet::SyscallResult<cairo_native::starknet::ExecutionInfoV2> {
        todo!()
    }

    fn deploy(
        &mut self,
        _class_hash: Felt,
        _contract_address_salt: Felt,
        _calldata: &[Felt],
        _deploy_from_zero: bool,
        _remaining_gas: &mut u64,
    ) -> cairo_native::starknet::SyscallResult<(Felt, Vec<Felt>)> {
        todo!()
    }

    fn replace_class(
        &mut self,
        _class_hash: Felt,
        _remaining_gas: &mut u64,
    ) -> cairo_native::starknet::SyscallResult<()> {
        todo!()
    }

    fn library_call(
        &mut self,
        _class_hash: Felt,
        _function_selector: Felt,
        _calldata: &[Felt],
        _remaining_gas: &mut u64,
    ) -> cairo_native::starknet::SyscallResult<Vec<Felt>> {
        todo!()
    }

    fn call_contract(
        &mut self,
        _address: Felt,
        _entry_point_selector: Felt,
        _calldata: &[Felt],
        _remaining_gas: &mut u64,
    ) -> cairo_native::starknet::SyscallResult<Vec<Felt>> {
        todo!()
    }

    fn storage_read(
        &mut self,
        _address_domain: u32,
        _address: Felt,
        _remaining_gas: &mut u64,
    ) -> cairo_native::starknet::SyscallResult<Felt> {
        todo!()
    }

    fn storage_write(
        &mut self,
        _address_domain: u32,
        _address: Felt,
        _value: Felt,
        _remaining_gas: &mut u64,
    ) -> cairo_native::starknet::SyscallResult<()> {
        todo!()
    }

    fn emit_event(
        &mut self,
        _keys: &[Felt],
        _data: &[Felt],
        _remaining_gas: &mut u64,
    ) -> cairo_native::starknet::SyscallResult<()> {
        todo!()
    }

    fn send_message_to_l1(
        &mut self,
        _to_address: Felt,
        _payload: &[Felt],
        _remaining_gas: &mut u64,
    ) -> cairo_native::starknet::SyscallResult<()> {
        todo!()
    }

    fn keccak(
        &mut self,
        _input: &[u64],
        _remaining_gas: &mut u64,
    ) -> cairo_native::starknet::SyscallResult<cairo_native::starknet::U256> {
        todo!()
    }

    fn secp256k1_new(
        &mut self,
        _x: cairo_native::starknet::U256,
        _y: cairo_native::starknet::U256,
        _remaining_gas: &mut u64,
    ) -> cairo_native::starknet::SyscallResult<Option<cairo_native::starknet::Secp256k1Point>> {
        todo!()
    }

    fn secp256k1_add(
        &mut self,
        _p0: cairo_native::starknet::Secp256k1Point,
        _p1: cairo_native::starknet::Secp256k1Point,
        _remaining_gas: &mut u64,
    ) -> cairo_native::starknet::SyscallResult<cairo_native::starknet::Secp256k1Point> {
        todo!()
    }

    fn secp256k1_mul(
        &mut self,
        _p: cairo_native::starknet::Secp256k1Point,
        _m: cairo_native::starknet::U256,
        _remaining_gas: &mut u64,
    ) -> cairo_native::starknet::SyscallResult<cairo_native::starknet::Secp256k1Point> {
        todo!()
    }

    fn secp256k1_get_point_from_x(
        &mut self,
        _x: cairo_native::starknet::U256,
        _y_parity: bool,
        _remaining_gas: &mut u64,
    ) -> cairo_native::starknet::SyscallResult<Option<cairo_native::starknet::Secp256k1Point>> {
        todo!()
    }

    fn secp256k1_get_xy(
        &mut self,
        _p: cairo_native::starknet::Secp256k1Point,
        _remaining_gas: &mut u64,
    ) -> cairo_native::starknet::SyscallResult<(
        cairo_native::starknet::U256,
        cairo_native::starknet::U256,
    )> {
        todo!()
    }

    fn secp256r1_new(
        &mut self,
        _x: cairo_native::starknet::U256,
        _y: cairo_native::starknet::U256,
        _remaining_gas: &mut u64,
    ) -> cairo_native::starknet::SyscallResult<Option<cairo_native::starknet::Secp256r1Point>> {
        todo!()
    }

    fn secp256r1_add(
        &mut self,
        _p0: cairo_native::starknet::Secp256r1Point,
        _p1: cairo_native::starknet::Secp256r1Point,
        _remaining_gas: &mut u64,
    ) -> cairo_native::starknet::SyscallResult<cairo_native::starknet::Secp256r1Point> {
        todo!()
    }

    fn secp256r1_mul(
        &mut self,
        _p: cairo_native::starknet::Secp256r1Point,
        _m: cairo_native::starknet::U256,
        _remaining_gas: &mut u64,
    ) -> cairo_native::starknet::SyscallResult<cairo_native::starknet::Secp256r1Point> {
        todo!()
    }

    fn secp256r1_get_point_from_x(
        &mut self,
        _x: cairo_native::starknet::U256,
        _y_parity: bool,
        _remaining_gas: &mut u64,
    ) -> cairo_native::starknet::SyscallResult<Option<cairo_native::starknet::Secp256r1Point>> {
        todo!()
    }

    fn secp256r1_get_xy(
        &mut self,
        _p: cairo_native::starknet::Secp256r1Point,
        _remaining_gas: &mut u64,
    ) -> cairo_native::starknet::SyscallResult<(
        cairo_native::starknet::U256,
        cairo_native::starknet::U256,
    )> {
        todo!()
    }

    fn sha256_process_block(
        &mut self,
        _state: &mut [u32; 8],
        _block: &[u32; 16],
        _remaining_gas: &mut u64,
    ) -> cairo_native::starknet::SyscallResult<()> {
        todo!()
    }

    fn get_class_hash_at(
        &mut self,
        _contract_address: Felt,
        _remaining_gas: &mut u64,
    ) -> cairo_native::starknet::SyscallResult<Felt> {
        todo!()
    }

    fn meta_tx_v0(
        &mut self,
        _address: Felt,
        _entry_point_selector: Felt,
        _calldata: &[Felt],
        _signature: &[Felt],
        _remaining_gas: &mut u64,
    ) -> cairo_native::starknet::SyscallResult<Vec<Felt>> {
        todo!()
    }
}

fn args_size(args: &[Arg]) -> usize {
    args.iter().map(Arg::size).sum()
}
