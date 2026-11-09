use anyhow::{Context, Result, anyhow, bail, ensure};
use camino::Utf8PathBuf;
use clap::Args;
use serde_json::Map;
use sncast::accounts::{
    AccountDeploymentService, AccountName, AccountRecord, AccountRepository, AccountsError, ChainId,
};
use sncast::helpers::dry_run::DryRunArgs;
use sncast::helpers::fee::FeeArgs;
use sncast::helpers::rpc::RpcArgs;
use sncast::response::account::deploy::AccountDeployResponse;
use sncast::response::invoke::InvokeResponse;
use sncast::response::ui::UI;
use sncast::signers::RuntimeSigner;
use sncast::{
    AccountType, WaitForTx, check_account_file_exists, get_account_data_from_keystore,
    get_account_record_from_repository,
};
use starknet_rust::providers::JsonRpcClient;
use starknet_rust::providers::jsonrpc::HttpTransport;
use starknet_rust::signers::{LocalWallet, Signer, SigningKey};
use starknet_types_core::felt::Felt;

#[derive(Args, Debug)]
#[command(about = "Deploy an account to the Starknet")]
pub struct Deploy {
    /// Name of the account to be deployed
    #[arg(short, long)]
    pub name: Option<String>,

    #[command(flatten)]
    pub fee_args: FeeArgs,

    #[command(flatten)]
    pub dry_run_args: DryRunArgs,

    #[command(flatten)]
    pub rpc: RpcArgs,

    /// If passed, the command will not trigger an interactive prompt to add an account as a default
    #[arg(long)]
    pub silent: bool,
}

#[expect(clippy::too_many_arguments)]
pub async fn deploy(
    provider: &JsonRpcClient<HttpTransport>,
    repository: &AccountRepository,
    deploy_args: &Deploy,
    chain_id: Felt,
    wait_config: WaitForTx,
    account: &str,
    keystore_path: Option<Utf8PathBuf>,
    fee_args: FeeArgs,
    dry_run_args: DryRunArgs,
    ui: &UI,
) -> Result<AccountDeployResponse> {
    if let Some(keystore_path) = keystore_path {
        deploy_from_keystore(
            provider,
            chain_id,
            fee_args,
            dry_run_args,
            wait_config,
            account,
            keystore_path,
            ui,
        )
        .await
        .map(Into::into)
    } else {
        let account_name = deploy_args
            .name
            .clone()
            .ok_or_else(|| anyhow!("Required argument `--name` not provided"))?;
        check_account_file_exists(repository)?;
        deploy_from_accounts_file(
            provider,
            repository,
            account_name,
            chain_id,
            fee_args,
            dry_run_args,
            wait_config,
            ui,
        )
        .await
        .map(Into::into)
    }
}

#[expect(clippy::too_many_arguments)]
async fn deploy_from_keystore(
    provider: &JsonRpcClient<HttpTransport>,
    chain_id: Felt,
    fee_args: FeeArgs,
    dry_run_args: DryRunArgs,
    wait_config: WaitForTx,
    account: &str,
    keystore_path: Utf8PathBuf,
    ui: &UI,
) -> Result<InvokeResponse> {
    let account_data = get_account_data_from_keystore(account, &keystore_path)?;

    let is_deployed = account_data
        .deployed
        .ok_or_else(|| anyhow!("Failed to get status key from account JSON file"))?;
    if is_deployed {
        bail!("Account already deployed");
    }

    let private_key_felt = account_data
        .signer_type
        .private_key()
        .context("Private key not found in keystore account")?;
    let private_key = SigningKey::from_secret_scalar(private_key_felt);
    let public_key = account_data.public_key;

    if public_key != private_key.verifying_key().scalar() {
        bail!("Public key and private key from keystore do not match");
    }

    let account_type = account_data
        .account_type
        .ok_or(AccountsError::MissingField {
            field: "type",
            operation: "account deployment",
        })?;
    let class_hash = account_data.class_hash.ok_or(AccountsError::MissingField {
        field: "class_hash",
        operation: "account deployment",
    })?;
    let salt = account_data.salt.ok_or(AccountsError::MissingField {
        field: "salt",
        operation: "account deployment",
    })?;

    let signer = LocalWallet::from_signing_key(private_key);
    let (address, result) = AccountDeploymentService::deploy(
        provider,
        account_type,
        class_hash,
        signer,
        salt,
        chain_id,
        fee_args,
        dry_run_args,
        wait_config,
        ui,
    )
    .await?;

    if let InvokeResponse::Transaction(_) = &result {
        update_keystore_account(account, address)?;
    }

    Ok(result)
}

#[expect(clippy::too_many_arguments)]
async fn deploy_from_accounts_file(
    provider: &JsonRpcClient<HttpTransport>,
    repository: &AccountRepository,
    name: String,
    chain_id: Felt,
    fee_args: FeeArgs,
    dry_run_args: DryRunArgs,
    wait_config: WaitForTx,
    ui: &UI,
) -> Result<InvokeResponse> {
    let account_data = get_account_record_from_repository(&name, chain_id, repository)?;
    let (account_type, class_hash, salt) = extract_deployment_fields(&account_data)?;
    let signer = RuntimeSigner::from_spec(account_data.signer.clone(), ui).await?;
    let actual_public_key = signer.get_public_key().await?.scalar();
    ensure!(
        actual_public_key == account_data.public_key,
        "{} signer public key does not match the account: expected {:#x}, got {actual_public_key:#x}",
        signer.kind(),
        account_data.public_key
    );
    let (_, result) = AccountDeploymentService::deploy(
        provider,
        account_type,
        class_hash,
        signer,
        salt,
        chain_id,
        fee_args,
        dry_run_args,
        wait_config,
        ui,
    )
    .await?;

    if let InvokeResponse::Transaction(_) = &result {
        update_account_in_accounts_file(repository, &name, chain_id, ui)?;
    }

    Ok(result)
}

fn extract_deployment_fields(account: &AccountRecord) -> Result<(AccountType, Felt, Felt)> {
    let deployable = account.as_deployable()?;
    Ok((
        deployable.account_type(),
        deployable.class_hash(),
        deployable.salt(),
    ))
}

fn update_account_in_accounts_file(
    repository: &AccountRepository,
    account_name: &str,
    chain_id: Felt,
    ui: &UI,
) -> Result<()> {
    repository
        .update_record(
            &ChainId::from(chain_id),
            &AccountName::new(account_name)?,
            |record| {
                record.deployed = Some(true);
                Ok(())
            },
        )?
        .migration_outcome
        .notify_if_migrated(ui);

    Ok(())
}

fn update_keystore_account(account: &str, address: Felt) -> Result<()> {
    let account_path = Utf8PathBuf::from(account.to_string());
    let contents =
        std::fs::read_to_string(account_path.clone()).context("Failed to read account file")?;
    let mut items: Map<String, serde_json::Value> = serde_json::from_str(&contents)
        .map_err(|_| anyhow!("Failed to parse account file at {account_path}"))?;

    items["deployment"]["status"] = serde_json::Value::from("deployed");
    items
        .get_mut("deployment")
        .and_then(|deployment| deployment.as_object_mut())
        .expect("Failed to get deployment as an object")
        .retain(|key, _| key != "salt" && key != "context");

    items["deployment"]["address"] = format!("{address:#x}").into();

    std::fs::write(&account_path, serde_json::to_string_pretty(&items).unwrap())
        .context("Failed to write to account file")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use sncast::ValidatedWaitParams;
    use starknet_rust::core::types::FeeEstimate;
    use test_case::test_case;
    use wiremock::matchers::body_partial_json;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test_case(true, false, "signer public key does not match the account"; "wrong signer")]
    #[test_case(false, true, "Failed to estimate fee:"; "failed estimation")]
    #[test_case(false, false, "Estimated fee (4400) is higher than provided max fee (1)"; "max fee too low")]
    #[tokio::test]
    async fn deployment_errors_do_not_modify_accounts_file(
        wrong_key: bool,
        failed_estimation: bool,
        expected: &str,
    ) {
        let directory = tempfile::tempdir().unwrap();
        let path = Utf8PathBuf::from_path_buf(directory.path().join("accounts.json")).unwrap();
        let mut accounts: serde_json::Value =
            serde_json::from_str(include_str!("../../../tests/data/accounts/accounts.json"))
                .unwrap();
        if wrong_key {
            accounts["accounts"]["SN_SEPOLIA"]["oz"]["public_key"] = json!("0x1");
        }
        let original = serde_json::to_vec(&accounts).unwrap();
        std::fs::write(&path, &original).unwrap();
        let repository = AccountRepository::try_new(&path).unwrap();

        let server = MockServer::start().await;
        Mock::given(body_partial_json(json!({"method": "starknet_getNonce"})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "jsonrpc": "2.0", "id": 1, "result": "0x0"
            })))
            .mount(&server)
            .await;
        let fee_response = if failed_estimation {
            json!({"error": {"code": -32603, "message": "estimation unavailable"}})
        } else {
            json!({"result": [FeeEstimate {
                l1_gas_consumed: 10,
                l1_gas_price: 60,
                l2_gas_consumed: 30,
                l2_gas_price: 40,
                l1_data_gas_consumed: 50,
                l1_data_gas_price: 20,
                overall_fee: 4400,
            }]})
        };
        Mock::given(body_partial_json(json!({"method": "starknet_estimateFee"})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "jsonrpc": "2.0", "id": 1,
                "result": fee_response.get("result"),
                "error": fee_response.get("error"),
            })))
            .mount(&server)
            .await;
        let provider = sncast::get_provider(&server.uri().parse().unwrap()).unwrap();
        let fee_args = FeeArgs {
            max_fee: Some(Felt::ONE.try_into().unwrap()),
            ..Default::default()
        };
        let args = Deploy {
            name: Some("oz".to_owned()),
            fee_args,
            dry_run_args: DryRunArgs::default(),
            rpc: RpcArgs::default(),
            silent: true,
        };
        let error = deploy(
            &provider,
            &repository,
            &args,
            sncast::SEPOLIA,
            WaitForTx {
                wait: false,
                wait_params: ValidatedWaitParams::default(),
                show_ui_outputs: false,
            },
            "",
            None,
            fee_args,
            DryRunArgs::default(),
            &UI::default(),
        )
        .await
        .unwrap_err();
        assert!(error.to_string().contains(expected), "{error:#}");
        assert_eq!(std::fs::read(&path).unwrap(), original);
    }
}
