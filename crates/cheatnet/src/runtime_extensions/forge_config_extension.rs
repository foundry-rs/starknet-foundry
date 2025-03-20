use config::RawForgeConfig;
use conversions::serde::deserialize::BufferReader;
use runtime::{CheatcodeHandlingResult, EnhancedHintError, ExtensionLogic, StarknetRuntime};

pub mod config;

pub struct ForgeConfigExtension<'a> {
    pub config: &'a mut RawForgeConfig,
}

// This runtime extension provides an implementation logic for snforge configuration run.
impl<'a> ExtensionLogic for ForgeConfigExtension<'a> {
    type Runtime = StarknetRuntime<'a>;

    fn handle_cheatcode(
        &mut self,
        selector: &str,
        mut input_reader: BufferReader<'_>,
        _extended_runtime: &mut Self::Runtime,
    ) -> Result<CheatcodeHandlingResult, EnhancedHintError> {
        macro_rules! config_cheatcode {
            ( $prop:ident) => {{
                self.config.$prop = Some(input_reader.read()?);

                Ok(CheatcodeHandlingResult::from_serializable(()))
            }};
        }

        match selector {
            "set_config_fork" => config_cheatcode!(fork),
            "set_config_available_gas" => config_cheatcode!(available_gas),
            "set_config_ignore" => config_cheatcode!(ignore),
            "set_config_should_panic" => config_cheatcode!(should_panic),
            "set_config_fuzzer" => config_cheatcode!(fuzzer),
            "is_config_mode" => Ok(CheatcodeHandlingResult::from_serializable(true)),
            _ => Ok(CheatcodeHandlingResult::Forwarded),
        }
    }
}
