use crate::metadata::MetadataCommand;
use crate::version::VersionCommand;
use anyhow::Context;
use scarb_ui::args::{FeaturesSpec, PackagesFilter, ProfileSpec, ToEnvVars};
use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::{env, io};
use thiserror::Error;

/// Error thrown while trying to execute `scarb` command.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum ScarbCommandError {
    /// Failed to read `scarb` output.
    #[error("failed to read `scarb` output")]
    Io(#[from] io::Error),
    /// Error during execution of `scarb` command.
    #[error("`scarb` exited with error")]
    ScarbError,
}

/// A builder for `scarb` command invocation.
#[derive(Clone, Debug, Default)]
#[expect(clippy::struct_excessive_bools)]
pub struct ScarbCommand {
    args: Vec<OsString>,
    current_dir: Option<PathBuf>,
    env: HashMap<OsString, Option<OsString>>,
    inherit_stderr: bool,
    inherit_stdout: bool,
    json: bool,
    offline: bool,
    manifest_path: Option<PathBuf>,
    scarb_path: Option<PathBuf>,
}

impl ScarbCommand {
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
        cmd.inherit_stderr();
        cmd.inherit_stdout();
        cmd
    }

    /// Creates [`MetadataCommand`] command
    #[must_use]
    pub fn metadata() -> MetadataCommand {
        MetadataCommand::new()
    }

    /// Creates [`VersionCommand`] command
    #[must_use]
    pub fn version() -> VersionCommand {
        VersionCommand::new()
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
        self.envs(filter.to_env_vars());
        self
    }

    /// Pass features specification filter to `scarb` call.
    pub fn features(&mut self, features: FeaturesSpec) -> &mut Self {
        self.envs(features.to_env_vars());
        self
    }

    /// Pass profile to `scarb` call.
    pub fn profile(&mut self, profile: ProfileSpec) -> &mut Self {
        self.envs(profile.to_env_vars());
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

    /// Inherit standard error, i.e. show Scarb errors in this process's standard error.
    pub fn inherit_stderr(&mut self) -> &mut Self {
        self.inherit_stderr = true;
        self
    }

    /// Inherit standard output, i.e. show Scarb output in this process's standard output.
    pub fn inherit_stdout(&mut self) -> &mut Self {
        self.inherit_stdout = true;
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

        if self.inherit_stderr {
            cmd.stderr(Stdio::inherit());
        }

        if self.inherit_stdout {
            cmd.stdout(Stdio::inherit());
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
    pub fn run(&self) -> Result<(), ScarbCommandError> {
        let mut cmd = self.command();
        if cmd.status()?.success() {
            Ok(())
        } else {
            Err(ScarbCommandError::ScarbError)
        }
    }
}
