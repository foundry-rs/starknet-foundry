use crate::helpers::{block_explorer::LinkProvider, configuration::CastConfig, rpc::RpcArgs};
use foundry_ui::Message;
use serde::Serialize;
use serde_json::{Value, json};
use starknet_types_core::felt::Felt;

const SNCAST_FORCE_SHOW_EXPLORER_LINKS_ENV: &str = "SNCAST_FORCE_SHOW_EXPLORER_LINKS";

// TODO(#3391): This code should be refactored to either use common `Message` trait or be directly
// included in `sncast` output messages.
pub trait OutputLink {
    const TITLE: &'static str;

    fn format_links(&self, provider: Box<dyn LinkProvider>) -> String;
}

#[derive(Serialize)]
pub struct ExplorerLinksMessage {
    title: String,
    links: String,
}

impl ExplorerLinksMessage {
    pub fn new<T>(response: &T, provider: Box<dyn LinkProvider>) -> Self
    where
        T: OutputLink,
    {
        Self {
            title: T::TITLE.to_string(),
            links: response.format_links(provider),
        }
    }
}

impl Message for ExplorerLinksMessage {
    fn text(&self) -> String {
        format!("\nTo see {} details, visit:\n{}", self.title, self.links)
    }

    fn json(&self) -> Value {
        json!(self)
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ExplorerError {
    #[error("The chosen block explorer service is not available for Sepolia Network")]
    SepoliaNotSupported,
    #[error("Custom network is not recognized by block explorer service")]
    UnrecognizedNetwork,
}

pub fn block_explorer_link_if_allowed<T>(
    result: &anyhow::Result<T>,
    chain_id: Felt,
    rpc: &RpcArgs,
    config: &CastConfig,
) -> Option<ExplorerLinksMessage>
where
    T: OutputLink + Clone,
{
    if (!config.show_explorer_links || rpc.is_localhost(&config.url))
        && !is_explorer_link_overridden()
    {
        return None;
    }

    let Ok(response) = result else {
        return None;
    };

    let network = chain_id.try_into().ok()?;

    config
        .block_explorer
        .unwrap_or_default()
        .as_provider(network)
        .ok()
        .map(|provider| ExplorerLinksMessage::new(response, provider))
}

#[must_use]
pub fn is_explorer_link_overridden() -> bool {
    std::env::var(SNCAST_FORCE_SHOW_EXPLORER_LINKS_ENV)
        .map(|value| value == "1")
        .unwrap_or(false)
}
