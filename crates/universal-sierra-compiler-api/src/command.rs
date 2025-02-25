use anyhow::Context;
use std::env;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// A builder for `universal-sierra-compiler` command invocation.
#[derive(Clone, Debug, Default)]
pub struct UniversalSierraCompilerCommand {
    args: Vec<OsString>,
    current_dir: Option<PathBuf>,
    inherit_stderr: bool,
}

impl UniversalSierraCompilerCommand {
    /// Creates a default `universal-sierra-compiler` command,
    /// which will look for `universal-sierra-compiler` in `$PATH`
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Ensures that `universal-sierra-compiler` binary is available in the system.
    pub fn ensure_available() -> anyhow::Result<()> {
        which::which(UniversalSierraCompilerCommand::binary_path())
            .context("Cannot find `universal-sierra-compiler` binary. \
                      Make sure you have USC installed https://github.com/software-mansion/universal-sierra-compiler \
                      and added to PATH (or set at UNIVERSAL_SIERRA_COMPILER env var)"
            )?;
        Ok(())
    }

    /// Current directory of the `universal-sierra-compiler` process.
    pub fn current_dir(&mut self, path: impl Into<PathBuf>) -> &mut Self {
        self.current_dir = Some(path.into());
        self
    }

    /// Inherit standard error, i.e. show USC errors in this process's standard error.
    pub fn inherit_stderr(&mut self) -> &mut Self {
        self.inherit_stderr = true;
        self
    }

    /// Adds an argument to pass to `universal-sierra-compiler`.
    pub fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self {
        self.args.push(arg.as_ref().to_os_string());
        self
    }

    /// Adds multiple arguments to pass to `universal-sierra-compiler`.
    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.args
            .extend(args.into_iter().map(|s| s.as_ref().to_os_string()));
        self
    }

    /// Build executable `universal-sierra-compiler` command.
    #[must_use]
    pub fn command(&self) -> Command {
        let universal_sierra_compiler = UniversalSierraCompilerCommand::binary_path();

        let mut cmd = Command::new(universal_sierra_compiler);

        cmd.args(&self.args);

        if let Some(path) = &self.current_dir {
            cmd.current_dir(path);
        }

        if self.inherit_stderr {
            cmd.stderr(Stdio::inherit());
        }

        cmd
    }

    fn binary_path() -> PathBuf {
        env::var("UNIVERSAL_SIERRA_COMPILER")
            .map(PathBuf::from)
            .ok()
            .unwrap_or_else(|| PathBuf::from("universal-sierra-compiler"))
    }
}
