use crate::helpers::block_explorer::{LinkProvider, Service};
use foundry_ui::Message;
use serde::Serialize;
use serde_json::{Value, json};
use starknet_types_core::felt::Felt;

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
    show_links: bool,
    explorer: Option<Service>,
) -> Option<ExplorerLinksMessage>
where
    T: OutputLink + Clone,
{
    if !show_links {
        return None;
    }

    let Ok(response) = result else {
        return None;
    };
    let Ok(network) = chain_id.try_into() else {
        return None;
    };

    if let Ok(provider) = explorer.unwrap_or_default().as_provider(network) {
        return Some(ExplorerLinksMessage::new(response, provider));
    }

    None
}
