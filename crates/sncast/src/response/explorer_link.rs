use crate::helpers::block_explorer::{LinkProvider, Service};
use foundry_ui::Message;
use serde::Serialize;
use starknet_types_core::felt::Felt;

pub trait OutputLink {
    const TITLE: &'static str;

    fn format_links(&self, provider: Box<dyn LinkProvider>) -> String;
}

#[derive(Serialize)]
pub struct OutputLinkMessage {
    title: String,
    links: String,
}

impl OutputLinkMessage {
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

impl Message for OutputLinkMessage {
    fn text(&self) -> String {
        format!("\nTo see {} details, visit:\n{}", self.title, self.links)
    }

    fn json(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize as JSON")
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
) -> Option<OutputLinkMessage>
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
        return Some(OutputLinkMessage::new(response, provider));
    }

    None
}
