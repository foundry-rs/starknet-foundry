use anyhow::{Result, bail};
use clap::{ArgGroup, Args};
use foundry_ui::components::warning::WarningMessage;
use promptly::prompt_default;
use sncast::accounts::{AccountName, AccountRepository, ChainId};
use sncast::get_chain_id;
use sncast::helpers::configuration::CastConfig;
use sncast::helpers::rpc::RpcArgs;
use sncast::response::account::delete::AccountDeleteResponse;
use sncast::response::ui::UI;

#[derive(Args, Debug)]
#[command(about = "Delete account information from the accounts file")]
#[command(group(ArgGroup::new("networks")
    .args(&["url", "network", "chain_id", "network_name"])
    .required(true)
    .multiple(false)))]
pub struct Delete {
    /// Name of the account to be deleted
    #[arg(short, long)]
    pub name: String,

    /// Assume "yes" as answer to confirmation prompt and run non-interactively
    #[arg(long, default_value = "false")]
    pub yes: bool,

    #[command(flatten)]
    pub rpc: RpcArgs,

    /// Chain ID of the network in accounts file, e.g. `SN_SEPOLIA`
    #[arg(long)]
    pub chain_id: Option<ChainId>,

    /// Deprecated, use `--chain-id` instead
    #[arg(long)]
    pub network_name: Option<String>,
}

pub fn delete(
    name: &AccountName,
    repository: &AccountRepository,
    chain_id: &ChainId,
    yes: bool,
    ui: &UI,
) -> Result<AccountDeleteResponse> {
    repository.find(chain_id, name)?;

    if !yes {
        let prompt_text = format!(
            "Do you want to remove the account {name} for chain ID {chain_id} from local file {}?",
            repository.path()
        );

        if !prompt_default(prompt_text, false)? {
            bail!("Delete aborted");
        }
    }

    repository
        .remove(chain_id, name)
        .map_err(anyhow::Error::from)?
        .migration_outcome
        .notify_if_migrated(ui);

    let result = "Account successfully removed".to_string();

    Ok(AccountDeleteResponse { result })
}

pub(crate) async fn get_chain_id_to_delete_from(
    delete: &Delete,
    config: &CastConfig,
    ui: &UI,
) -> Result<ChainId> {
    if let Some(chain_id) = delete.chain_id {
        return Ok(chain_id);
    }

    if let Some(network_name) = &delete.network_name {
        if let Some(chain_id) = ChainId::from_legacy_network_name(network_name) {
            ui.print_warning(WarningMessage::new(format!(
                "Network name `{network_name}` is deprecated, use `--chain-id {chain_id}` instead"
            )));
            return Ok(chain_id);
        }
        return Ok(network_name.parse()?);
    }

    let provider = delete.rpc.get_provider(config, ui).await?;
    Ok(ChainId::from(get_chain_id(&provider).await?))
}
