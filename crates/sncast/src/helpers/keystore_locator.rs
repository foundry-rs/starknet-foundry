use camino::Utf8PathBuf;
use clap::Args;

// TODO: consider compact UX: 
// `--keystore` / `--keystore-path PATH` as one flag with 
// `num_args = 0..=1` instead of multiple flags, with bare flag = default path.

#[derive(Args, Debug)]
#[group(id = "registry_keystore_locator", multiple = false)]
pub struct RegistryKeystoreLocator {
    /// Store the secret in an encrypted keystore, and register `keystore_path` in the accounts file
    #[arg(long)]
    pub keystore: bool,

    /// Store the secret in an encrypted keystore at the specified path, and register  `keystore_path` in the accounts file
    #[arg(long = "keystore-path", value_name = "PATH")]
    pub keystore_path: Option<Utf8PathBuf>,
}

impl RegistryKeystoreLocator {
    #[must_use]
    pub fn is_set(&self) -> bool {
        self.keystore || self.keystore_path.is_some()
    }

    #[must_use]
    pub fn resolve(&self, accounts_file: &Utf8PathBuf, account_name: &str) -> Option<Utf8PathBuf> {
        if let Some(path) = &self.keystore_path {
            Some(path.clone())
        } else if self.keystore {
            Some(default_registry_keystore_path(accounts_file, account_name))
        } else {
            None
        }
    }
}

fn default_registry_keystore_path(accounts_file: &Utf8PathBuf, account_name: &str) -> Utf8PathBuf {
    let parent = accounts_file
        .parent()
        .map_or_else(|| Utf8PathBuf::from("."), Utf8PathBuf::from);
    parent
        .join("keystores")
        .join(format!("{account_name}.json"))
}

#[derive(Args, Debug)]
pub struct KeystoreImport {
    /// Path to an encrypted keystore file to register in the accounts file
    #[arg(long)]
    pub keystore: Option<Utf8PathBuf>,

    /// Path to a starkli account JSON
    #[arg(long = "keystore-account", requires = "keystore")]
    pub keystore_account: Option<Utf8PathBuf>,
}
