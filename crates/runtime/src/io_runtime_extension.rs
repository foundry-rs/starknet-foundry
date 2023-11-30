use cairo_felt::Felt252;
use cairo_lang_runner::short_string::as_cairo_short_string;
use cairo_vm::vm::errors::hint_errors::HintError;
use cheatnet::{
    cheatcodes::EnhancedHintError, execution::cheatable_syscall_handler::CheatableSyscallHandler,
};

use crate::{CheatcodeHandlingResult, ExtendedRuntime, ExtensionLogic, RuntimeExtension};

pub struct IORuntimeState {}

pub type IORuntime<'a> =
    ExtendedRuntime<RuntimeExtension<IORuntimeState, CheatableSyscallHandler<'a>>>;

impl<'a> ExtensionLogic for RuntimeExtension<IORuntimeState, CheatableSyscallHandler<'a>> {
    type Runtime = CheatableSyscallHandler<'a>;

    fn get_extended_runtime_mut(&mut self) -> &mut CheatableSyscallHandler<'a> {
        &mut self.extended_runtime
    }

    fn get_extended_runtime(&self) -> &CheatableSyscallHandler<'a> {
        &self.extended_runtime
    }

    fn handle_cheatcode(
        &mut self,
        selector: &str,
        inputs: Vec<Felt252>,
    ) -> Result<CheatcodeHandlingResult, EnhancedHintError> {
        match selector {
            "print" => {
                print(inputs);
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            _ => Err(EnhancedHintError::from(HintError::CustomHint(
                "Only `print` cheatcode is available in contracts.".into(),
            ))),
        }
    }
}

fn as_printable_short_string(value: &Felt252) -> Option<String> {
    let bytes: Vec<u8> = value.to_bytes_be();
    if bytes.iter().any(u8::is_ascii_control) {
        return None;
    }

    as_cairo_short_string(value)
}

pub fn print(inputs: Vec<Felt252>) {
    for value in inputs {
        if let Some(short_string) = as_printable_short_string(&value) {
            println!("original value: [{value}], converted to a string: [{short_string}]",);
        } else {
            println!("original value: [{value}]");
        }
    }
}
