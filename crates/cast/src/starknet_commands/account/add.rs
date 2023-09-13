use crate::starknet_commands::account::{prepare_account_json, write_account_to_accounts_file};
use anyhow::{ensure, Result};
use camino::Utf8PathBuf;
use cast::helpers::response_structs::AccountAddResponse;
use cast::helpers::scarb_utils::CastConfig;
use clap::Args;
use starknet::core::types::FieldElement;
use starknet::signers::SigningKey;

#[derive(Args, Debug)]
#[command(about = "Add an account to the accounts file")]
pub struct Add {
    #[clap(short, long)]
    pub address: FieldElement,

    #[clap(short, long)]
    pub class_hash: Option<FieldElement>,

    #[clap(short, long)]
    pub deployed: bool,

    #[clap(short, long)]
    pub private_key: FieldElement,

    #[clap(short, long)]
    pub public_key: Option<FieldElement>,

    /// Salt for the address
    #[clap(short, long)]
    pub salt: Option<FieldElement>,

    /// If passed, a profile with corresponding data will be created in Scarb.toml
    #[clap(short, long)]
    pub add_profile: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn add(
    config: &CastConfig,
    path_to_scarb_toml: &Option<Utf8PathBuf>,
    chain_id: FieldElement,
    add: &Add,
) -> Result<AccountAddResponse> {
    let private_key = &SigningKey::from_secret_scalar(add.private_key);
    if let Some(public_key) = &add.public_key {
        ensure!(
            public_key == &private_key.verifying_key().scalar(),
            "public key mismatch"
        );
    }

    let account_json = prepare_account_json(
        private_key,
        add.address,
        add.deployed,
        add.class_hash,
        add.salt,
    );

    write_account_to_accounts_file(
        path_to_scarb_toml,
        &config.rpc_url,
        &config.account,
        &config.accounts_file,
        chain_id,
        account_json.clone(),
        add.add_profile,
    )?;

    Ok(AccountAddResponse {
        add_profile: if add.add_profile {
            "Profile successfully added to Scarb.toml".to_string()
        } else {
            "--add-profile flag was not set. No profile added to Scarb.toml".to_string()
        },
    })
}
