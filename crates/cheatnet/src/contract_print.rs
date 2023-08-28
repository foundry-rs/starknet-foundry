use cairo_felt::Felt252;
use cairo_lang_casm::{
    hints::{Hint, StarknetHint},
    operand::ResOperand,
};
use cairo_lang_runner::{
    casm_run::{extract_relocatable, vm_get_range},
    short_string::as_cairo_short_string,
};
use cairo_vm::vm::{errors::hint_errors::HintError, vm_core::VirtualMachine};

pub enum PrintingResult {
    Printed,
    Passed,
    Err(HintError),
}

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

pub fn contract_print(
    vm: &mut VirtualMachine,
    maybe_extended_hint: Option<&Hint>,
) -> PrintingResult {
    if let Some(Hint::Starknet(StarknetHint::Cheatcode {
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
            Err(err) => return PrintingResult::Err(err),
        };

        match selector {
            "print" => {
                for value in inputs {
                    if let Some(short_string) = as_cairo_short_string(&value) {
                        println!("PRINT: {value} => {short_string}",);
                    } else {
                        println!("PRINT: {value}");
                    }
                }
                return PrintingResult::Printed;
            }
            _ => {
                return PrintingResult::Err(HintError::CustomHint(
                    "Only `print` cheatcode is available in contracts.".into(),
                ))
            }
        }
    };
    PrintingResult::Passed
}
