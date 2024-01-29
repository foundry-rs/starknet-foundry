use anyhow::Result;
use sncast::response::print::{get_formatted_output, OutputFormattingConfig};
use sncast::response::structs::CommandResponse;

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
    output_config: OutputFormattingConfig,
}

impl ScriptLogger {
    pub fn new(is_quiet: bool, output_config: OutputFormattingConfig) -> Self {
        let verbosity = if is_quiet {
            Verbosity::Quiet
        } else {
            Verbosity::Normal
        };

        Self {
            verbosity,
            output_config,
        }
    }

    pub fn print_subcommand_response<T: CommandResponse>(
        &self,
        command: &str,
        response: T,
    ) -> Result<()> {
        if self.verbosity >= Verbosity::Normal {
            let formatted_output = get_formatted_output(
                &mut Ok(response),
                String::from("script_subcommand"),
                command.to_string(),
                self.output_config,
            )?;

            for val in formatted_output {
                println!("{val}");
            }
            println!();
        }
        Ok(())
    }
}
