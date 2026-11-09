//! Account model and persistence boundaries.

pub mod deployment;
pub mod error;
pub mod model;
pub mod repository;
pub mod schema;
pub mod selector;
pub mod service;

pub use deployment::AccountDeploymentService;
pub use error::{AccountsError, AccountsFileError};
pub use model::{
    AccountName, AccountRecord, AccountRegistry, AccountType, ChainId, DeployableAccountRecord,
};
pub use repository::{AccountRepository, MigrationOutcome, MutationResult};
pub use selector::AccountSelector;
pub use service::AccountService;
