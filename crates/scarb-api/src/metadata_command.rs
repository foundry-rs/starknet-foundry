use crate::{output_transform::PrintOutput, ScarbCommand, ScarbCommandError};
use scarb_metadata::{Metadata, MetadataCommandError, VersionPin};
use serde_json::Value;
use std::io::BufRead;

impl From<ScarbCommandError> for MetadataCommandError {
    fn from(value: ScarbCommandError) -> Self {
        match value {
            ScarbCommandError::Io(io) => MetadataCommandError::Io(io),
            ScarbCommandError::ScarbError { stdout, stderr } => {
                MetadataCommandError::ScarbError { stdout, stderr }
            }
        }
    }
}

#[derive(Default, Clone)]
pub struct ScarbErrorPrettyPrint;

fn pretty_print(out: &[u8], print: impl Fn(&str) + Copy) -> Result<(), std::io::Error> {
    for line in BufRead::split(out, b'\n') {
        let line = line?;
        let value: Value = match serde_json::from_slice(&line) {
            Ok(ok) => ok,
            Err(_) => {
                print(&String::from_utf8_lossy(&line));
                continue;
            }
        };

        print("Scarb returned error:\n");

        let message = value["message"].as_str();

        if value["type"] == "error" && message.is_some() {
            message.unwrap().lines().for_each(print);
        } else {
            serde_json::to_string_pretty(&value)
                .unwrap()
                .lines()
                .for_each(print);
        }
    }

    Ok(())
}

impl PrintOutput for ScarbErrorPrettyPrint {
    fn print_stdout(stdout: &[u8]) -> Result<(), std::io::Error> {
        pretty_print(&stdout, |line| {
            println!("Scarb error:    {}", line);
        })?;

        Ok(())
    }
    fn print_stderr(stderr: &[u8]) -> Result<(), std::io::Error> {
        pretty_print(&stderr, |line| {
            eprintln!("Scarb error:    {}", line);
        })?;

        Ok(())
    }
}

pub struct ScarbMetadataCommand<Stdout, Stderr> {
    inner: ScarbCommand<Stdout, Stderr>,
    no_deps: bool,
}

impl<Stdout, Stderr> ScarbMetadataCommand<Stdout, Stderr> {
    pub(crate) fn new(inner: ScarbCommand<Stdout, Stderr>) -> Self {
        Self {
            inner,
            no_deps: false,
        }
    }

    /// Output information only about workspace members and don't fetch dependencies.
    pub fn no_deps(&mut self) -> &mut Self {
        self.no_deps = true;
        self
    }

    /// Pretty prints Scarb error
    ///
    /// It will override previously set transformers
    pub fn error_pretty_print(
        self,
    ) -> ScarbMetadataCommand<ScarbErrorPrettyPrint, ScarbErrorPrettyPrint>
    where
        Stdout: Default,
        Stderr: Default,
    {
        ScarbMetadataCommand {
            inner: self
                .inner
                .use_custom_print_stdout::<ScarbErrorPrettyPrint>()
                .use_custom_print_stderr::<ScarbErrorPrettyPrint>(),
            no_deps: self.no_deps,
        }
    }

    /// Runs configured `scarb metadata` and returns parsed `Metadata`.
    pub fn run(mut self) -> Result<Metadata, MetadataCommandError>
    where
        Stdout: PrintOutput,
        Stderr: PrintOutput,
    {
        self.inner
            .json()
            .args(["metadata", "--format-version"])
            .arg(VersionPin.numeric().to_string());

        if self.no_deps {
            self.inner.arg("--no-deps");
        }

        let output = self.inner.run()?;
        let data = output.stdout.as_slice();

        let mut err = None;
        for line in BufRead::split(data, b'\n') {
            let line = line?;

            if !line.starts_with(br#"{"version":"#) {
                continue;
            }
            match serde_json::from_slice(&line) {
                Ok(metadata) => return Ok(metadata),
                Err(serde_err) => err = Some(serde_err.into()),
            }
        }

        Err(err.unwrap_or_else(|| MetadataCommandError::NotFound {
            stdout: String::from_utf8_lossy(data).into(),
        }))
    }
}
