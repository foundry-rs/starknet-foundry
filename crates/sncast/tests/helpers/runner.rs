use snapbox::cargo_bin;
use snapbox::cmd::{Command, OutputAssert};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

#[must_use]
pub fn runner(args: &[&str]) -> CastCommand {
    Cast::new().command().args(args)
}

struct CastState {
    config_dir: EnvPath,
}

#[must_use]
pub struct Cast {
    state: CastState,
    sncast_bin: PathBuf,
}

impl Cast {
    pub fn new() -> Self {
        Self {
            state: CastState {
                config_dir: EnvPath::temp_dir(),
            },
            sncast_bin: cargo_bin!("sncast").to_path_buf(),
        }
    }

    /// Override the global config dir (`SNFOUNDRY_CONFIG`).
    pub fn config_dir(mut self, path: impl AsRef<Path>) -> Self {
        self.state.config_dir = EnvPath::borrow(path);
        self
    }

    #[must_use]
    pub fn command(self) -> CastCommand {
        let inner = Command::new(self.sncast_bin)
            .env("SNFOUNDRY_CONFIG", self.state.config_dir.path());
        CastCommand {
            inner,
            _state: self.state,
        }
    }
}

impl Default for Cast {
    fn default() -> Self {
        Self::new()
    }
}

pub struct CastCommand {
    inner: Command,
    _state: CastState,
}

impl CastCommand {
    pub fn arg(mut self, arg: impl AsRef<OsStr>) -> Self {
        self.inner = self.inner.arg(arg);
        self
    }

    pub fn args(mut self, args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> Self {
        self.inner = self.inner.args(args);
        self
    }

    pub fn env(mut self, key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> Self {
        self.inner = self.inner.env(key, value);
        self
    }

    pub fn env_remove(mut self, key: impl AsRef<OsStr>) -> Self {
        self.inner = self.inner.env_remove(key);
        self
    }

    pub fn current_dir(self, dir: impl AsRef<Path>) -> Self {
        Self {
            inner: self.inner.current_dir(dir),
            _state: self._state,
        }
    }

    pub fn stdin(mut self, input: impl snapbox::IntoData) -> Self {
        self.inner = self.inner.stdin(input);
        self
    }

    #[must_use]
    pub fn assert(self) -> OutputAssert {
        let CastCommand { inner, _state: _ } = self;
        inner.assert()
    }

    pub fn output(self) -> Result<std::process::Output, std::io::Error> {
        let CastCommand { inner, _state: _ } = self;
        inner.output()
    }
}

enum EnvPath {
    Managed(TempDir),
    Unmanaged(PathBuf),
}

impl EnvPath {
    fn temp_dir() -> Self {
        Self::Managed(TempDir::new().expect("Failed to create temp dir"))
    }

    fn borrow(path: impl AsRef<Path>) -> Self {
        Self::Unmanaged(path.as_ref().to_path_buf())
    }

    fn path(&self) -> &Path {
        match self {
            EnvPath::Managed(temp) => temp.path(),
            EnvPath::Unmanaged(p) => p,
        }
    }
}
