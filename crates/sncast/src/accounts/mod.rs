//! Account domain and persistence boundaries.

pub mod domain;
pub mod error;
pub mod schema;

pub use domain::{
    AccountName, AccountRecord, AccountRegistry, AccountType, ConnectedAccountRecord,
    DeployableAccountRecord, NetworkName,
};
pub use error::AccountsError;
