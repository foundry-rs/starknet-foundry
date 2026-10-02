//! Account model and persistence boundaries.

pub mod error;
pub mod model;

pub use error::AccountsError;
pub use model::{
    AccountName, AccountRecord, AccountRegistry, AccountType, DeployableAccountRecord, NetworkName,
};
