use anyhow::Result;
use sncast::response::print::{print_command_result, OutputFormat, OutputValue};
use sncast::response::structs::CommandResponse;
use sncast::NumbersFormat;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub enum Verbosity {
    /// Silence all script output except for errors
    Quiet,

    /// Default verbosity level
    #[default]
    Normal,
}

#[derive(Debug)]
pub struct ScriptLogger {
    verbosity: Verbosity,
    numbers_format: NumbersFormat,
    output_format: OutputFormat,
}

impl ScriptLogger {
    pub fn new(is_quiet: bool, numbers_format: NumbersFormat, output_format: OutputFormat) -> Self {
        let verbosity = if is_quiet {
            Verbosity::Quiet
        } else {
            Verbosity::Normal
        };

        Self {
            verbosity,
            numbers_format,
            output_format,
        }
    }

    pub fn print_subcommand_response<T: CommandResponse>(
        &self,
        command: &str,
        response: T,
    ) -> Result<()> {
        if self.verbosity >= Verbosity::Normal {
            let header = (
                String::from("script_subcommand"),
                OutputValue::String(command.to_string()),
            );
            print_command_result(
                header,
                &mut Ok(response),
                self.numbers_format,
                &self.output_format,
            )?;
            println!();
            return Ok(());
        }
        Ok(())
    }
}
