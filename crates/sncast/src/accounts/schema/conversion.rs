use std::collections::BTreeMap;

use crate::accounts::schema::{v1, v2};
use crate::accounts::{
    AccountName, AccountRecord, AccountRegistry, AccountsError, AccountsFileError, NetworkName,
};
use crate::helpers::ledger::{parse_derivation_path, validate_derivation_path};
use crate::signers::{KeystoreSpec, LedgerSpec, PrivateKeySpec, SignerSpec};

impl TryFrom<v1::AccountsFile> for AccountRegistry {
    type Error = AccountsError;

    fn try_from(accounts_data: v1::AccountsFile) -> Result<Self, Self::Error> {
        convert_accounts_data(accounts_data.0, |network, account_name, account| {
            let signer = match account.signer {
                v1::Signer::PrivateKey { private_key } => {
                    SignerSpec::PrivateKey(PrivateKeySpec::new(private_key))
                }
                v1::Signer::Ledger { ledger_path } => {
                    validate_derivation_path(&ledger_path).map_err(|error| {
                        AccountsError::InvalidAccountEntry {
                            network: network.to_owned(),
                            account: account_name.to_owned(),
                            message: error.to_string(),
                        }
                    })?;
                    SignerSpec::Ledger(LedgerSpec::new(ledger_path))
                }
            };

            Ok(AccountRecord {
                public_key: account.public_key,
                address: account.address,
                salt: account.salt,
                deployed: account.deployed,
                class_hash: account.class_hash,
                legacy: account.legacy,
                account_type: account.account_type,
                signer,
            })
        })
    }
}

impl TryFrom<v2::AccountsFile> for AccountRegistry {
    type Error = AccountsError;

    fn try_from(file: v2::AccountsFile) -> Result<Self, Self::Error> {
        if file.version != 2 {
            return Err(AccountsFileError::Version {
                version: file.version.to_string(),
            }
            .into());
        }

        convert_accounts_data(file.accounts, |network, account_name, account| {
            let signer = match account.signer {
                v2::Signer::PrivateKey { private_key } => {
                    SignerSpec::PrivateKey(PrivateKeySpec::new(private_key))
                }
                v2::Signer::Keystore { path, password_env } => {
                    SignerSpec::Keystore(KeystoreSpec::new(path, password_env))
                }
                v2::Signer::Ledger { derivation_path } => {
                    let derivation_path =
                        parse_derivation_path(&derivation_path).map_err(|error| {
                            AccountsError::InvalidAccountEntry {
                                network: network.to_owned(),
                                account: account_name.to_owned(),
                                message: error.to_string(),
                            }
                        })?;
                    SignerSpec::Ledger(LedgerSpec::new(derivation_path))
                }
            };

            Ok(AccountRecord {
                public_key: account.public_key,
                address: account.address,
                salt: account.salt,
                deployed: account.deployed,
                class_hash: account.class_hash,
                legacy: account.legacy,
                account_type: account.account_type,
                signer,
            })
        })
    }
}

impl From<&AccountRegistry> for v2::AccountsFile {
    fn from(registry: &AccountRegistry) -> Self {
        let accounts = registry
            .networks()
            .iter()
            .map(|(network, accounts)| {
                let accounts = accounts
                    .iter()
                    .map(|(name, account)| {
                        let signer = match &account.signer {
                            SignerSpec::PrivateKey(spec) => v2::Signer::PrivateKey {
                                private_key: spec.private_key(),
                            },
                            SignerSpec::Keystore(spec) => v2::Signer::Keystore {
                                path: spec.path().to_owned(),
                                password_env: spec.password_env().map(ToOwned::to_owned),
                            },
                            SignerSpec::Ledger(spec) => v2::Signer::Ledger {
                                derivation_path: spec.derivation_path().derivation_string(),
                            },
                        };

                        let account = v2::Account {
                            public_key: account.public_key,
                            address: account.address,
                            salt: account.salt,
                            deployed: account.deployed,
                            class_hash: account.class_hash,
                            legacy: account.legacy,
                            account_type: account.account_type,
                            signer,
                        };

                        (name.to_string(), account)
                    })
                    .collect::<BTreeMap<_, _>>();
                (network.to_string(), accounts)
            })
            .collect::<BTreeMap<_, _>>();

        v2::AccountsFile {
            version: v2::VERSION,
            accounts,
        }
    }
}

fn convert_accounts_data<T, F>(
    accounts_data: BTreeMap<String, BTreeMap<String, T>>,
    convert: F,
) -> Result<AccountRegistry, AccountsError>
where
    F: Fn(&str, &str, T) -> Result<AccountRecord, AccountsError>,
{
    let mut converted = BTreeMap::new();
    for (network, accounts) in accounts_data {
        let mut converted_accounts = BTreeMap::new();
        for (name, account) in accounts {
            let account = convert(&network, &name, account)?;
            converted_accounts.insert(AccountName::new(name)?, account);
        }
        converted.insert(NetworkName::new(network)?, converted_accounts);
    }
    Ok(AccountRegistry::new(converted))
}
