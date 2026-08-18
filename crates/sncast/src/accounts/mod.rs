//! Account domain and persistence boundaries.

pub mod deployment;
pub mod domain;
pub mod error;
pub mod repository;
pub mod schema;
pub mod selector;
pub mod service;
pub mod storage;

pub use deployment::AccountDeploymentService;
pub use domain::{
    AccountName, AccountRecord, AccountRegistry, AccountType, ConnectedAccountRecord,
    DeployableAccountRecord, NetworkName,
};
pub use error::AccountsError;
pub use repository::{AccountRepository, MutationResult};
pub use selector::AccountSelector;
pub use service::AccountService;
