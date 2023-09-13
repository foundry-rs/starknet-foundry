use crate::starknet_commands::account::{prepare_account_json, write_account_to_accounts_file};
use anyhow::{anyhow, bail, Result};
use camino::Utf8PathBuf;
use cast::helpers::constants::OZ_CLASS_HASH;
use cast::helpers::response_structs::AccountCreateResponse;
use cast::helpers::scarb_utils::CastConfig;
use cast::{extract_or_generate_salt, get_chain_id, parse_number};
use clap::Args;
use serde_json::json;
use starknet::accounts::{AccountFactory, OpenZeppelinAccountFactory};
use starknet::core::types::{FeeEstimate, FieldElement};
use starknet::core::utils::get_contract_address;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use starknet::signers::{LocalWallet, SigningKey};

#[derive(Args, Debug)]
#[command(about = "Create an account with all important secrets")]
pub struct Create {
    /// Account name under which account information is going to be saved
    #[clap(short, long)]
    pub name: String,

    /// Salt for the address
    #[clap(short, long)]
    pub salt: Option<FieldElement>,

    /// If passed, a profile with corresponding data will be created in Scarb.toml
    #[clap(short, long)]
    pub add_profile: bool,
    // TODO (#253): think about supporting different account providers
    /// Custom open zeppelin contract class hash of declared contract
    #[clap(short, long)]
    pub class_hash: Option<String>,
}

#[allow(clippy::too_many_arguments)]
pub async fn create(
    config: &CastConfig,
    provider: &JsonRpcClient<HttpTransport>,
    path_to_scarb_toml: Option<Utf8PathBuf>,
    chain_id: FieldElement,
    salt: Option<FieldElement>,
    add_profile: bool,
    class_hash: Option<String>,
) -> Result<AccountCreateResponse> {
    let (account_json, max_fee) = {
        let salt = extract_or_generate_salt(salt);
        let class_hash = {
            let ch = match &class_hash {
                Some(class_hash) => class_hash,
                None => OZ_CLASS_HASH,
            };
            parse_number(ch)?
        };
        generate_account(provider, salt, class_hash).await?
    };

    let address = parse_number(
        account_json["address"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid address"))?,
    )?;

    write_account_to_accounts_file(
        &path_to_scarb_toml,
        &config.rpc_url,
        &config.account,
        &config.accounts_file,
        chain_id,
        account_json.clone(),
        add_profile,
    )?;

    let mut output = vec![("address", format!("{address:#x}"))];
    if account_json["deployed"] == json!(false) {
        println!("Account successfully created. Prefund generated address with at least {max_fee} tokens. It is good to send more in the case of higher demand, max_fee * 2 = {}", max_fee * 2);
        output.push(("max_fee", format!("{max_fee:#x}")));
    }

    if add_profile {
        output.push((
            "add-profile",
            "Profile successfully added to Scarb.toml".to_string(),
        ));
    }

    Ok(AccountCreateResponse {
        address,
        max_fee: FieldElement::from(max_fee),
        add_profile: if add_profile {
            "Profile successfully added to Scarb.toml".to_string()
        } else {
            "--add-profile flag was not set. No profile added to Scarb.toml".to_string()
        },
    })
}

async fn generate_account(
    provider: &JsonRpcClient<HttpTransport>,
    salt: FieldElement,
    class_hash: FieldElement,
) -> Result<(serde_json::Value, u64)> {
    let private_key = SigningKey::from_random();

    let address: FieldElement = get_contract_address(
        salt,
        class_hash,
        &[private_key.verifying_key().scalar()],
        FieldElement::ZERO,
    );

    let account_json =
        prepare_account_json(&private_key, address, false, Some(class_hash), Some(salt));

    let max_fee = get_account_deployment_fee(&private_key, class_hash, salt, provider)
        .await?
        .overall_fee;

    Ok((account_json, max_fee))
}

async fn get_account_deployment_fee(
    private_key: &SigningKey,
    class_hash: FieldElement,
    salt: FieldElement,
    provider: &JsonRpcClient<HttpTransport>,
) -> Result<FeeEstimate> {
    let signer = LocalWallet::from_signing_key(private_key.clone());
    let chain_id = get_chain_id(provider).await?;
    let factory = OpenZeppelinAccountFactory::new(class_hash, chain_id, signer, provider).await?;
    let deployment = factory.deploy(salt);

    let fee_estimate = deployment.estimate_fee().await;

    if let Err(err) = &fee_estimate {
        if err
            .to_string()
            .contains("StarknetErrorCode.UNDECLARED_CLASS")
        {
            bail!("The class {class_hash} is undeclared, try using --class-hash with a class hash that is already declared");
        }
    }

    Ok(fee_estimate?)
}

#[cfg(test)]
mod tests {
    use cast::helpers::constants::DEFAULT_ACCOUNTS_FILE;
    use cast::helpers::scarb_utils::CastConfig;
    use sealed_test::prelude::rusty_fork_test;
    use sealed_test::prelude::sealed_test;
    use std::fs;

    use crate::starknet_commands::account::add_created_profile_to_configuration;

    #[sealed_test(files = ["tests/data/contracts/v1/balance/Scarb.toml"])]
    fn test_add_created_profile_to_configuration_happy_case() {
        let config = CastConfig {
            rpc_url: String::from("http://some-url"),
            account: String::from("some-name"),
            accounts_file: "accounts".into(),
        };
        let res = add_created_profile_to_configuration(&None, &config);

        assert!(res.is_ok());

        let contents = fs::read_to_string("Scarb.toml").expect("Unable to read Scarb.toml");
        assert!(contents.contains("[tool.sncast.some-name]"));
        assert!(contents.contains("account = \"some-name\""));
        assert!(contents.contains("url = \"http://some-url\""));
        assert!(contents.contains("accounts-file = \"accounts\""));
    }

    #[sealed_test(files = ["tests/data/contracts/v1/balance/Scarb.toml"])]
    fn test_add_created_profile_to_configuration_profile_already_exists() {
        let config = CastConfig {
            rpc_url: String::from("http://some-url"),
            account: String::from("myprofile"),
            accounts_file: DEFAULT_ACCOUNTS_FILE.into(),
        };
        let res = add_created_profile_to_configuration(&None, &config);

        assert!(res.is_err());
    }
}
