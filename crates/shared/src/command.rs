use anyhow::{bail, Context, Ok, Result};
use std::process::{Command, Output};

pub trait CommandExt {
    fn output_checked(&mut self) -> Result<Output>;
}

impl CommandExt for Command {
    fn output_checked(&mut self) -> Result<Output> {
        let command = self.get_program().to_string_lossy().to_string();

        let output = self
            .output()
            .with_context(|| format!("Failed to run {command}"))?;

        if !output.status.success() {
            bail!("Command {command} failed with status {}", output.status);
        }

        Ok(output)
    }
}
