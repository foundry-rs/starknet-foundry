//! Account model and persistence boundaries.

pub mod error;
pub mod model;
pub mod repository;
pub mod schema;

pub use error::{AccountsError, AccountsFileError};
pub use model::{
    AccountName, AccountRecord, AccountRegistry, AccountType, DeployableAccountRecord, NetworkName,
};
pub use repository::{AccountRepository, MigrationOutcome, MutationResult};
