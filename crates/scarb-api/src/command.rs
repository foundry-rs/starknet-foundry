use anyhow::Context;
use scarb_ui::args::PackagesFilter;
use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::io::{stderr, stdout, Write};
use std::marker::PhantomData;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::{env, io};
use thiserror::Error;

use crate::output_transform::{OutputTransform, PassByPrint};
use crate::ScarbMetadataCommand;

/// Error thrown while trying to execute `scarb` command.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum ScarbCommandError {
    /// Failed to read `scarb` output.
    #[error("failed to read `scarb` output")]
    Io(#[from] io::Error),
    /// Error during execution of `scarb` command.
    #[error("`scarb` exited with error")]
    ScarbError { stdout: String, stderr: String },
}

/// A builder for `scarb` command invocation.
#[derive(Clone, Debug, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct ScarbCommand<Stdout, Stderr> {
    args: Vec<OsString>,
    current_dir: Option<PathBuf>,
    env: HashMap<OsString, Option<OsString>>,
    env_clear: bool,
    print_stderr: bool,
    print_stdout: bool,
    json: bool,
    offline: bool,
    manifest_path: Option<PathBuf>,
    scarb_path: Option<PathBuf>,
    stdout_print: PhantomData<Stdout>,
    stderr_print: PhantomData<Stderr>,
}

impl ScarbCommand<PassByPrint, PassByPrint> {
    /// Creates a default `scarb` command, which will look for `scarb` in `$PATH` and
    /// for `Scarb.toml` in the current directory or its ancestors.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a default `scarb` command, with inherited standard error and standard output.
    #[must_use]
    pub fn new_with_stdio() -> Self {
        let mut cmd = Self::new();

        cmd.print_stderr().print_stdout();

        cmd
    }
}

macro_rules! impl_print {
    ($self:expr) => {
        ScarbCommand {
            args: $self.args,
            current_dir: $self.current_dir,
            env: $self.env,
            env_clear: $self.env_clear,
            print_stderr: $self.print_stderr,
            print_stdout: $self.print_stdout,
            json: $self.json,
            manifest_path: $self.manifest_path,
            offline: $self.offline,
            scarb_path: $self.scarb_path,
            stderr_print: PhantomData,
            stdout_print: PhantomData,
        }
    };
}

impl<Stdout, Stderr> ScarbCommand<Stdout, Stderr> {
    /// Print standard output, i.e. captures and format Scarb output, then writes in this process's standard output
    #[must_use]
    pub fn use_custom_print_stdout<T>(mut self) -> ScarbCommand<T, Stderr>
    where
        T: OutputTransform,
    {
        self.print_stdout();

        impl_print!(self)
    }

    /// Print standard error, i.e. captures and format Scarb error, then writes in this process's standard error
    #[must_use]
    pub fn use_custom_print_stderr<T>(mut self) -> ScarbCommand<Stdout, T>
    where
        T: OutputTransform,
    {
        self.print_stderr();

        impl_print!(self)
    }

    /// Creates [`ScarbMetadataCommand`] from this command
    #[must_use]
    pub fn metadata(&self) -> ScarbMetadataCommand<Stdout, Stderr>
    where
        Stdout: OutputTransform,
        Stderr: OutputTransform,
    {
        ScarbMetadataCommand::new(self.clone())
    }

    /// Ensures that `scarb` binary is available in the system.
    pub fn ensure_available(&self) -> anyhow::Result<()> {
        which::which(self.binary_path())
            .context("Cannot find `scarb` binary. Make sure you have Scarb installed https://github.com/software-mansion/scarb")?;
        Ok(())
    }

    /// Path to `scarb` executable.
    ///
    /// If not set, this will use the `$SCARB` environment variable, and if that is not set, it
    /// will simply be `scarb` and the system will look it up in `$PATH`.
    pub fn scarb_path(&mut self, path: impl Into<PathBuf>) -> &mut Self {
        self.scarb_path = Some(path.into());
        self
    }

    /// Path to `Scarb.toml`.
    ///
    /// If not set, this will look for `Scarb.toml` in the current directory or its ancestors.
    pub fn manifest_path(&mut self, path: impl Into<PathBuf>) -> &mut Self {
        self.manifest_path = Some(path.into());
        self
    }

    /// Pass packages filter to `scarb` call.
    pub fn packages_filter(&mut self, filter: PackagesFilter) -> &mut Self {
        self.env("SCARB_PACKAGES_FILTER", filter.to_env());
        self
    }

    /// Current directory of the `scarb` process.
    pub fn current_dir(&mut self, path: impl Into<PathBuf>) -> &mut Self {
        self.current_dir = Some(path.into());
        self
    }

    /// Adds an argument to pass to `scarb`.
    pub fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self {
        self.args.push(arg.as_ref().to_os_string());
        self
    }

    /// Adds multiple arguments to pass to `scarb`.
    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.args
            .extend(args.into_iter().map(|s| s.as_ref().to_os_string()));
        self
    }

    /// Inserts or updates an environment variable mapping.
    pub fn env(&mut self, key: impl AsRef<OsStr>, val: impl AsRef<OsStr>) -> &mut Self {
        self.env.insert(
            key.as_ref().to_os_string(),
            Some(val.as_ref().to_os_string()),
        );
        self
    }

    /// Adds or updates multiple environment variable mappings.
    pub fn envs<I, K, V>(&mut self, vars: I) -> &mut Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        for (ref key, ref val) in vars {
            self.env(key, val);
        }
        self
    }

    /// Removes an environment variable mapping.
    pub fn env_remove(&mut self, key: impl AsRef<OsStr>) -> &mut Self {
        let key = key.as_ref();
        self.env.insert(key.to_os_string(), None);
        self
    }

    /// Clears the entire environment map for the child process.
    pub fn env_clear(&mut self) -> &mut Self {
        self.env.clear();
        self.env_clear = true;
        self
    }

    /// Enable manipulation of standard error
    /// if not set with [`Self::use_custom_print_stderr()`] writes to this process standard error
    pub fn print_stderr(&mut self) -> &mut Self {
        self.print_stderr = true;
        self
    }

    /// Enable manipulation of standard output
    /// if not set with [`Self::use_custom_print_stdout()`] writes to this process standard output
    pub fn print_stdout(&mut self) -> &mut Self {
        self.print_stdout = true;
        self
    }

    /// Set output format to JSON.
    pub fn json(&mut self) -> &mut Self {
        self.json = true;
        self
    }

    /// Enables offline mode.
    pub fn offline(&mut self) -> &mut Self {
        self.offline = true;
        self
    }

    /// Build executable `scarb` command.
    #[must_use]
    pub fn command(&self) -> Command {
        let scarb = self.binary_path();

        let mut cmd = Command::new(scarb);

        if self.json {
            cmd.arg("--json");
        }

        if self.offline {
            cmd.arg("--offline");
        }

        if let Some(manifest_path) = &self.manifest_path {
            cmd.arg("--manifest-path").arg(manifest_path);
        }

        cmd.args(&self.args);

        if let Some(path) = &self.current_dir {
            cmd.current_dir(path);
        }

        for (key, val) in &self.env {
            if let Some(val) = val {
                cmd.env(key, val);
            } else {
                cmd.env_remove(key);
            }
        }

        if self.env_clear {
            cmd.env_clear();
        }

        cmd
    }

    fn binary_path(&self) -> PathBuf {
        self.scarb_path
            .clone()
            .or_else(|| env::var("SCARB").map(PathBuf::from).ok())
            .unwrap_or_else(|| PathBuf::from("scarb"))
    }

    /// Runs configured `scarb` command.
    pub fn run(&self) -> Result<Output, ScarbCommandError>
    where
        Stdout: OutputTransform,
        Stderr: OutputTransform,
    {
        let output = self.command().output()?;

        if self.print_stdout {
            stdout().write_all(&Stdout::transform_stdout(&output.stdout))?;
        }
        if self.print_stderr {
            stderr().write_all(&Stderr::transform_stderr(&output.stderr))?;
        }

        if output.status.success() {
            Ok(output)
        } else {
            Err(ScarbCommandError::ScarbError {
                stdout: String::from_utf8_lossy(&output.stdout).into(),
                stderr: String::from_utf8_lossy(&output.stderr).into(),
            })
        }
    }
}
