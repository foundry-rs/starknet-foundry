use anyhow::{Context, Result};
use clap::{Args, Command, FromArgMatches};
use sncast::response::multicall::run::MulticallRunResponse;

use crate::starknet_commands::multicall::{ctx::MulticallCtx, deploy::MulticallDeploy};

pub fn run_with_calls(tokens: &[String]) -> Result<MulticallRunResponse> {
    let input = tokens.join(" ");
    let commands = input.split('/').map(|s| s.trim()).filter(|s| !s.is_empty());

    let mut ctx = MulticallCtx::default();

    for command in commands {
        let mut args = shell_words::split(command)?;
        let command = args
            .first()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Empty command"))?;

        match command.as_str() {
            "deploy" => {
                let deploy_args = parse_args::<MulticallDeploy>(command.clone(), &args[1..])?;
                
            }   
            _ => {
                println!("Unknown command: {}", command);
            }
        }
    }

    Ok(MulticallRunResponse { transaction_hash: Default::default() })
}

fn parse_args<T>(command_name: String, tokens: &[String]) -> anyhow::Result<T>
where
    T: Args + FromArgMatches,
{
    let cmd = T::augment_args(Command::new(command_name.clone()));

    let argv = std::iter::once(command_name.to_string())
        .chain(tokens.iter().cloned())
        .collect::<Vec<_>>();

    let matches = cmd
        .try_get_matches_from(argv)
        .with_context(|| format!("Failed to parse args for `{command_name}`"))?;

    T::from_arg_matches(&matches).map_err(Into::into)
}
