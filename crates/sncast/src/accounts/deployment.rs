use anyhow::{Result, anyhow};
use conversions::IntoConv;
use starknet_rust::accounts::{
    AccountDeploymentV3, AccountFactory, AccountFactoryError, ArgentAccountFactory,
    OpenZeppelinAccountFactory,
};
use starknet_rust::core::types::{
    ContractExecutionError, FeeEstimate, StarknetError::ClassHashNotFound,
    StarknetError::TransactionExecutionError, TransactionExecutionErrorData,
};
use starknet_rust::providers::ProviderError::StarknetError;
use starknet_rust::providers::jsonrpc::{HttpTransport, JsonRpcClient};
use starknet_rust::signers::Signer;
use starknet_types_core::felt::Felt;

use crate::accounts::AccountType;
use crate::apply_optional_fields;
use crate::helpers::braavos::BraavosAccountFactory;
use crate::helpers::constants::BRAAVOS_BASE_ACCOUNT_CLASS_HASH;
use crate::helpers::dry_run::DryRunArgs;
use crate::helpers::fee::{FeeArgs, FeeSettings};
use crate::response::errors::{SNCastProviderError, SNCastStarknetError};
use crate::response::invoke::{InvokeResponse, InvokeTransactionResponse};
use crate::response::ui::UI;
use crate::{WaitForTx, handle_account_factory_error, handle_rpc_error, handle_wait_for_tx};

pub struct AccountDeploymentService;

impl AccountDeploymentService {
    pub async fn compute_address<S>(
        provider: &JsonRpcClient<HttpTransport>,
        account_type: AccountType,
        class_hash: Felt,
        signer: S,
        salt: Felt,
        chain_id: Felt,
    ) -> Result<Felt>
    where
        S: Signer + Send + Sync,
        S::GetPublicKeyError: 'static,
    {
        match account_type {
            AccountType::OpenZeppelin => {
                let factory =
                    OpenZeppelinAccountFactory::new(class_hash, chain_id, signer, provider).await?;
                Ok(factory.deploy_v3(salt).address())
            }
            AccountType::Ready => {
                let factory =
                    ArgentAccountFactory::new(class_hash, chain_id, None, signer, provider).await?;
                Ok(factory.deploy_v3(salt).address())
            }
            AccountType::Braavos => {
                let factory = BraavosAccountFactory::new(
                    class_hash,
                    BRAAVOS_BASE_ACCOUNT_CLASS_HASH,
                    chain_id,
                    signer,
                    provider,
                )
                .await?;
                Ok(factory.deploy_v3(salt).address())
            }
        }
    }

    pub async fn estimate_fee<S>(
        provider: &JsonRpcClient<HttpTransport>,
        account_type: AccountType,
        class_hash: Felt,
        signer: S,
        salt: Felt,
        chain_id: Felt,
    ) -> Result<(Felt, FeeEstimate)>
    where
        S: Signer + Send + Sync,
        S::GetPublicKeyError: 'static,
    {
        match account_type {
            AccountType::OpenZeppelin => {
                let factory =
                    OpenZeppelinAccountFactory::new(class_hash, chain_id, signer, provider).await?;
                estimate_factory(factory, salt).await
            }
            AccountType::Ready => {
                let factory =
                    ArgentAccountFactory::new(class_hash, chain_id, None, signer, provider).await?;
                estimate_factory(factory, salt).await
            }
            AccountType::Braavos => {
                let factory = BraavosAccountFactory::new(
                    class_hash,
                    BRAAVOS_BASE_ACCOUNT_CLASS_HASH,
                    chain_id,
                    signer,
                    provider,
                )
                .await?;
                estimate_factory(factory, salt).await
            }
        }
    }

    #[expect(clippy::too_many_arguments)]
    pub async fn deploy<S>(
        provider: &JsonRpcClient<HttpTransport>,
        account_type: AccountType,
        class_hash: Felt,
        signer: S,
        salt: Felt,
        chain_id: Felt,
        fee_args: FeeArgs,
        dry_run_args: DryRunArgs,
        wait_config: WaitForTx,
        ui: &UI,
    ) -> Result<(Felt, InvokeResponse)>
    where
        S: Signer + Send + Sync,
        S::GetPublicKeyError: 'static,
        S::SignError: 'static,
    {
        match account_type {
            AccountType::Ready => {
                let factory =
                    ArgentAccountFactory::new(class_hash, chain_id, None, signer, provider).await?;
                deploy_factory(
                    factory,
                    provider,
                    salt,
                    fee_args,
                    dry_run_args,
                    wait_config,
                    class_hash,
                    ui,
                )
                .await
            }
            AccountType::OpenZeppelin => {
                let factory =
                    OpenZeppelinAccountFactory::new(class_hash, chain_id, signer, provider).await?;
                deploy_factory(
                    factory,
                    provider,
                    salt,
                    fee_args,
                    dry_run_args,
                    wait_config,
                    class_hash,
                    ui,
                )
                .await
            }
            AccountType::Braavos => {
                let factory = BraavosAccountFactory::new(
                    class_hash,
                    BRAAVOS_BASE_ACCOUNT_CLASS_HASH,
                    chain_id,
                    signer,
                    provider,
                )
                .await?;
                deploy_factory(
                    factory,
                    provider,
                    salt,
                    fee_args,
                    dry_run_args,
                    wait_config,
                    class_hash,
                    ui,
                )
                .await
            }
        }
    }
}

async fn estimate_factory<T>(factory: T, salt: Felt) -> Result<(Felt, FeeEstimate)>
where
    T: AccountFactory + Sync,
{
    let deployment = factory.deploy_v3(salt);
    let address = deployment.address();
    let fee = deployment.estimate_fee().await.map_err(|error| {
        anyhow!(
            "Failed to estimate account deployment fee. Reason: {}",
            handle_account_factory_error::<T>(error)
        )
    })?;
    Ok((address, fee))
}

fn execution_error_message(error: &ContractExecutionError) -> &str {
    match error {
        ContractExecutionError::Message(message) => message,
        ContractExecutionError::Nested(inner) => execution_error_message(&inner.error),
    }
}

#[expect(clippy::too_many_arguments)]
async fn deploy_factory<T>(
    account_factory: T,
    provider: &JsonRpcClient<HttpTransport>,
    salt: Felt,
    fee_args: FeeArgs,
    dry_run_args: DryRunArgs,
    wait_config: WaitForTx,
    class_hash: Felt,
    ui: &UI,
) -> Result<(Felt, InvokeResponse)>
where
    T: AccountFactory + Sync,
{
    let deployment = account_factory.deploy_v3(salt);
    let address = deployment.address();

    if dry_run_args.dry_run {
        return dry_run_args
            .estimate(|| deployment.estimate_fee())
            .await
            .map(|response| (address, InvokeResponse::DryRun(response)))
            .map_err(|error| anyhow!("Failed to estimate fee for dry run: {error}"));
    }

    let fee_settings = if fee_args.max_fee.is_some() {
        let fee_estimate = deployment
            .estimate_fee()
            .await
            .expect("Failed to estimate fee");
        fee_args.try_into_fee_settings(Some(&fee_estimate))
    } else {
        fee_args.try_into_fee_settings(None)
    };

    let FeeSettings {
        l1_gas,
        l1_gas_price,
        l2_gas,
        l2_gas_price,
        l1_data_gas,
        l1_data_gas_price,
        tip,
    } = fee_settings.expect("Failed to convert to fee settings");

    let deployment = apply_optional_fields!(
        deployment,
        l1_gas => AccountDeploymentV3::l1_gas,
        l1_gas_price => AccountDeploymentV3::l1_gas_price,
        l2_gas => AccountDeploymentV3::l2_gas,
        l2_gas_price => AccountDeploymentV3::l2_gas_price,
        l1_data_gas => AccountDeploymentV3::l1_data_gas,
        l1_data_gas_price => AccountDeploymentV3::l1_data_gas_price,
        tip => AccountDeploymentV3::tip
    );

    match deployment.send().await {
        Err(AccountFactoryError::Provider(error)) => match error {
            StarknetError(ClassHashNotFound) => Err(anyhow!(
                "Provided class hash {class_hash:#x} does not exist"
            )),
            StarknetError(TransactionExecutionError(TransactionExecutionErrorData {
                ref execution_error,
                ..
            })) => Err(anyhow!(execution_error_message(execution_error).to_owned())),
            StarknetError(error) => Err(SNCastProviderError::StarknetError(
                SNCastStarknetError::from_starknet_error_with_account(error, address.into_()),
            )
            .into()),
            _ => Err(handle_rpc_error(error)),
        },
        Err(_) => Err(anyhow!("Unknown AccountFactoryError")),
        Ok(result) => {
            let response = InvokeResponse::Transaction(InvokeTransactionResponse {
                transaction_hash: result.transaction_hash.into_(),
            });
            if let Err(message) = handle_wait_for_tx(
                provider,
                result.transaction_hash,
                response.clone(),
                wait_config,
                ui,
            )
            .await
            {
                return Err(anyhow!(message));
            }
            Ok((address, response))
        }
    }
}
