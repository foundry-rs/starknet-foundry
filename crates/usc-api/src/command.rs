use anyhow::Context;
use std::ffi::{OsStr, OsString};
use std::io;
use std::path::PathBuf;
use std::process::Command;
use thiserror::Error;

/// Error thrown while trying to execute `universal-sierra-compiler` command.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum UniversalSierraCompilerCommandError {
    /// Failed to read `universal-sierra-compiler` output.
    #[error("failed to read `universal-sierra-compiler` output")]
    Io(#[from] io::Error),
    /// Error during execution of `universal-sierra-compiler` command.
    #[error("`universal-sierra-compiler` exited with error")]
    UniversalSierraCompilerError,
}

/// A builder for `universal-sierra-compiler` command invocation.
#[derive(Clone, Debug, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct UniversalSierraCompilerCommand {
    args: Vec<OsString>,
    current_dir: Option<PathBuf>,
    universal_sierra_compiler_path: Option<PathBuf>,
}

impl UniversalSierraCompilerCommand {
    /// Creates a default `universal-sierra-compiler` command,
    /// which will look for `universal-sierra-compiler` in `$PATH`
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Ensures that `universal-sierra-compiler` binary is available in the system.
    pub fn ensure_available(&self) -> anyhow::Result<()> {
        which::which(self.binary_path())
            .context("Cannot find `universal-sierra-compiler` binary. Make sure you have USC installed https://github.com/software-mansion/universal-sierra-compiler")?;
        Ok(())
    }

    /// Path to `universal-sierra-compiler` executable.
    ///
    /// If not set it will simply be `universal-sierra-compiler`
    /// and the system will look it up in `$PATH`.
    pub fn universal_sierra_compiler_path(&mut self, path: impl Into<PathBuf>) -> &mut Self {
        self.universal_sierra_compiler_path = Some(path.into());
        self
    }

    /// Current directory of the `universal-sierra-compiler` process.
    pub fn current_dir(&mut self, path: impl Into<PathBuf>) -> &mut Self {
        self.current_dir = Some(path.into());
        self
    }

    /// Adds an argument to pass to `universal-sierra-compiler`.
    pub fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self {
        self.args.push(arg.as_ref().to_os_string());
        self
    }

    /// Build executable `universal-sierra-compiler` command.
    #[must_use]
    pub fn command(&self) -> Command {
        let universal_sierra_compiler = self.binary_path();

        let mut cmd = Command::new(universal_sierra_compiler);

        cmd.args(&self.args);

        if let Some(path) = &self.current_dir {
            cmd.current_dir(path);
        }

        cmd
    }

    fn binary_path(&self) -> PathBuf {
        self.universal_sierra_compiler_path
            .clone()
            .unwrap_or_else(|| PathBuf::from("universal_sierra_compiler"))
    }

    /// Runs configured `universal_sierra_compiler` command.
    pub fn run(&self) -> Result<(), UniversalSierraCompilerCommandError> {
        let mut cmd = self.command();
        if cmd.status()?.success() {
            Ok(())
        } else {
            Err(UniversalSierraCompilerCommandError::UniversalSierraCompilerError)
        }
    }
}
