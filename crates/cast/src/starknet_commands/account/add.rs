use crate::starknet_commands::account::{
    add_created_profile_to_configuration, prepare_account_json, write_account_to_accounts_file,
};
use anyhow::{ensure, Result};
use camino::Utf8PathBuf;
use cast::get_chain_id;
use cast::helpers::response_structs::AccountAddResponse;
use cast::helpers::scarb_utils::CastConfig;
use clap::Args;
use starknet::core::types::BlockTag::Pending;
use starknet::core::types::{BlockId, FieldElement};
use starknet::providers::{
    jsonrpc::{HttpTransport, JsonRpcClient},
    Provider,
};
use starknet::signers::SigningKey;

#[derive(Args, Debug)]
#[command(about = "Add an account to the accounts file")]
pub struct Add {
    /// Name of the account to be added
    #[clap(short, long)]
    pub name: String,

    /// Address of the account
    #[clap(short, long)]
    pub address: FieldElement,

    /// Class hash of the account
    #[clap(short, long)]
    pub class_hash: Option<FieldElement>,

    /// Account deployment status
    /// If not passed, sncast will check whether the account is deployed or not
    #[clap(short, long)]
    pub deployed: bool,

    /// Account private key
    #[clap(long)]
    pub private_key: FieldElement,

    /// Account public key
    #[clap(long)]
    pub public_key: Option<FieldElement>,

    /// Salt for the address
    #[clap(short, long)]
    pub salt: Option<FieldElement>,

    /// If passed, a profile with corresponding data will be created in Scarb.toml
    #[clap(long)]
    pub add_profile: bool,
}

pub async fn add(
    rpc_url: &str,
    account: &str,
    accounts_file: &Utf8PathBuf,
    path_to_scarb_toml: &Option<Utf8PathBuf>,
    provider: &JsonRpcClient<HttpTransport>,
    add: &Add,
) -> Result<AccountAddResponse> {
    let private_key = &SigningKey::from_secret_scalar(add.private_key);
    if let Some(public_key) = &add.public_key {
        ensure!(
            public_key == &private_key.verifying_key().scalar(),
            "The private key does not match the public key"
        );
    }

    let deployed = if add.deployed {
        true
    } else if provider
        .get_class_hash_at(BlockId::Tag(Pending), add.address)
        .await
        .is_ok()
    {
        println!("Contract detected as deployed on chain");
        true
    } else {
        false
    };

    let account_json =
        prepare_account_json(private_key, add.address, deployed, add.class_hash, add.salt);

    let chain_id = get_chain_id(provider).await?;
    write_account_to_accounts_file(account, accounts_file, chain_id, account_json.clone())?;

    if add.add_profile {
        let config = CastConfig {
            rpc_url: rpc_url.into(),
            account: account.into(),
            accounts_file: accounts_file.into(),
            keystore: Utf8PathBuf::default(),
        };
        add_created_profile_to_configuration(path_to_scarb_toml, &config)?;
    }

    Ok(AccountAddResponse {
        add_profile: if add.add_profile {
            "Profile successfully added to Scarb.toml".to_string()
        } else {
            "--add-profile flag was not set. No profile added to Scarb.toml".to_string()
        },
    })
}
