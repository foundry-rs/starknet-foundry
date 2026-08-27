use anyhow::{Result, bail};
use clap::{ArgGroup, Args};
use promptly::prompt;
use sncast::accounts::AccountRepository;
use sncast::helpers::configuration::CastConfig;
use sncast::helpers::rpc::RpcArgs;
use sncast::response::account::delete::AccountDeleteResponse;
use sncast::response::ui::UI;
use sncast::{chain_id_to_network_name, get_chain_id};

#[derive(Args, Debug)]
#[command(about = "Delete account information from the accounts file")]
#[command(group(ArgGroup::new("networks")
    .args(&["url", "network", "network_name"])
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

    /// Literal name of the network used in accounts file
    #[arg(long)]
    pub network_name: Option<String>,
}

pub fn delete(
    name: &str,
    repository: &AccountRepository,
    network_name: &str,
    yes: bool,
) -> Result<AccountDeleteResponse> {
    let registry = repository.load()?.registry;
    let Some(accounts) = registry.networks().get(network_name) else {
        bail!("No accounts defined for network = {network_name}");
    };
    if !accounts.contains_key(name) {
        bail!("Account with name {name} does not exist");
    }

    // Let's ask confirmation
    if !yes {
        let prompt_text = format!(
            "Do you want to remove the account {name} deployed to network {network_name} from local file {}? (Y/n)",
            repository.path()
        );
        let input: String = prompt(prompt_text)?;

        if !input.starts_with('Y') {
            bail!("Delete aborted");
        }
    }

    repository
        .remove(network_name, name)
        .map_err(|error| anyhow::anyhow!(error))?;
    let result = "Account successfully removed".to_string();
    Ok(AccountDeleteResponse { result })
}

pub(crate) async fn get_network_name(
    delete: &Delete,
    config: &CastConfig,
    ui: &UI,
) -> Result<String> {
    if let Some(network_name) = &delete.network_name {
        return Ok(network_name.clone());
    }

    let provider = delete.rpc.get_provider(config, ui).await?;
    Ok(chain_id_to_network_name(get_chain_id(&provider).await?))
}
