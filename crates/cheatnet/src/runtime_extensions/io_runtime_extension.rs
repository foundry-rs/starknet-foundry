use std::marker::PhantomData;

use cairo_felt::Felt252;
use cairo_lang_runner::short_string::as_cairo_short_string;

use runtime::{CheatcodeHandlingResult, EnhancedHintError, ExtendedRuntime, ExtensionLogic};

use super::cheatable_starknet_runtime_extension::CheatableStarknetRuntime;

pub struct IORuntimeExtension<'a> {
    pub lifetime: &'a PhantomData<()>,
}

pub type IORuntime<'a> = ExtendedRuntime<IORuntimeExtension<'a>>;

impl<'a> ExtensionLogic for IORuntimeExtension<'a> {
    type Runtime = CheatableStarknetRuntime<'a>;

    fn handle_cheatcode(
        &mut self,
        selector: &str,
        inputs: Vec<Felt252>,
        _extended_runtime: &mut Self::Runtime,
    ) -> Result<CheatcodeHandlingResult, EnhancedHintError> {
        match selector {
            "print" => {
                print(inputs);
                Ok(CheatcodeHandlingResult::Handled(vec![]))
            }
            _ => Ok(CheatcodeHandlingResult::Forwarded),
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
