//! Account model and persistence boundaries.

pub mod error;
pub mod model;
pub mod repository;
pub mod schema;

pub use error::AccountsError;
pub use model::{
    AccountName, AccountRecord, AccountRegistry, AccountType, ConnectedAccountRecord,
    DeployableAccountRecord, NetworkName,
};
pub use repository::{AccountRepository, MutationResult};
