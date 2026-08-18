//! Account domain and persistence boundaries.

pub mod domain;
pub mod error;

pub use domain::{
    AccountName, AccountRecord, AccountRegistry, AccountType, ConnectedAccountRecord,
    DeployableAccountRecord, NetworkName,
};
pub use error::AccountsError;
