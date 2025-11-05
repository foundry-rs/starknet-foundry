//! Functionality for fetching `scarb` metadata.
//!
//! # Note:
//! To allow more flexibility when changing internals, please make public as few items as possible
//! and try using more general functions like `metadata` and `metadata_for_dir`
//! instead of `metadata_with_opts`.

use crate::{ScarbUnavailableError, ensure_scarb_available};
use anyhow::Result;
pub use scarb_metadata::{Metadata, MetadataCommand, MetadataCommandError, PackageMetadata};
use std::path::PathBuf;

/// Errors that can occur when fetching `scarb` metadata.
#[derive(thiserror::Error, Debug)]
pub enum MetadataError {
    #[error(transparent)]
    ScarbNotFound(#[from] ScarbUnavailableError),
    #[error(transparent)]
    ScarbExecutionFailed(#[from] MetadataCommandError),
}

/// Options for fetching `scarb` metadata.
#[derive(Default)]
pub struct MetadataOpts {
    pub current_dir: Option<PathBuf>,
    pub no_deps: bool,
    pub profile: Option<String>,
}

/// Fetches `scarb` metadata for a specific directory.
pub fn metadata_for_dir(dir: impl Into<PathBuf>) -> Result<Metadata, MetadataError> {
    metadata_with_opts(MetadataOpts {
        current_dir: Some(dir.into()),
        ..MetadataOpts::default()
    })
}

/// Fetches `scarb` metadata for the current directory.
pub fn metadata() -> Result<Metadata, MetadataError> {
    metadata_with_opts(MetadataOpts::default())
}

/// Fetches `scarb` metadata with specified options.
pub fn metadata_with_opts(
    MetadataOpts {
        current_dir,
        no_deps,
        profile,
    }: MetadataOpts,
) -> Result<Metadata, MetadataError> {
    ensure_scarb_available()?;

    let mut command = MetadataCommand::new();

    if let Some(dir) = current_dir {
        command.current_dir(dir);
    }

    if let Some(profile) = profile {
        command.profile(profile);
    }

    if no_deps {
        command.no_deps();
    }

    command
        .inherit_stderr()
        .inherit_stdout()
        .exec()
        .map_err(MetadataError::ScarbExecutionFailed)
}
