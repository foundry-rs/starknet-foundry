use crate::execution::syscall_handler::{
    Passable, StackableHintProcessorLogic, StackableResourceTracker, StackableSyscallHandler,
};
use cairo_felt::Felt252;
use cairo_lang_casm::{
    hints::{Hint, StarknetHint},
    operand::ResOperand,
};
use cairo_lang_runner::{
    casm_run::{extract_relocatable, vm_get_range},
    short_string::as_cairo_short_string,
};
use cairo_vm::types::exec_scope::ExecutionScopes;
use cairo_vm::vm::{errors::hint_errors::HintError, vm_core::VirtualMachine};
use std::any::Any;
use std::collections::HashMap;

fn extract_input(
    vm: &mut VirtualMachine,
    input_start: &ResOperand,
    input_end: &ResOperand,
) -> Result<Vec<Felt252>, HintError> {
    let input_start = extract_relocatable(vm, input_start)?;
    let input_end = extract_relocatable(vm, input_end)?;
    vm_get_range(vm, input_start, input_end)
        .map_err(|_| HintError::CustomHint("Failed to read input data".into()))
}

pub struct ContractExecutionSyscallHandler;

impl StackableHintProcessorLogic for ContractExecutionSyscallHandler {
    fn execute_hint(
        &mut self,
        vm: &mut VirtualMachine,
        _exec_scopes: &mut ExecutionScopes,
        hint_data: &Box<dyn Any>,
        _constants: &HashMap<String, Felt252>,
    ) -> Passable<Result<(), HintError>> {
        let maybe_extended_hint = hint_data.downcast_ref::<Hint>();

        return if let Some(Hint::Starknet(StarknetHint::Cheatcode {
            selector,
            input_start,
            input_end,
            ..
        })) = maybe_extended_hint
        {
            let selector = &selector.value.to_bytes_be().1;
            let selector = std::str::from_utf8(selector).unwrap();
            let inputs = match extract_input(vm, input_start, input_end) {
                Ok(inputs) => inputs,
                Err(err) => return Passable::Done(Err(err)),
            };

            Passable::Done(match selector {
                "print" => {
                    for value in inputs {
                        if let Some(short_string) = as_cairo_short_string(&value) {
                            println!(
                                "original value: [{value}], converted to a string: [{short_string}]",
                            );
                        } else {
                            println!("original value: [{value}]");
                        }
                    }
                    Ok(())
                }
                _ => Err(HintError::CustomHint(
                    "Only `print` cheatcode is available in contracts.".into(),
                )),
            })
        } else {
            Passable::Pass
        };
    }
}
impl StackableResourceTracker for ContractExecutionSyscallHandler {}
impl StackableSyscallHandler for ContractExecutionSyscallHandler {}
