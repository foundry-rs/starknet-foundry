use crate::helpers::block_explorer::{LinkProvider, Service};
use foundry_ui::formats::OutputFormat;
use starknet_types_core::felt::Felt;

pub trait OutputLink {
    const TITLE: &'static str;

    fn format_links(&self, provider: Box<dyn LinkProvider>) -> String;

    fn print_links(&self, provider: Box<dyn LinkProvider>) -> String {
        let title = Self::TITLE;
        let links = self.format_links(provider);
        format!("\nTo see {title} details, visit:\n{links}")
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ExplorerError {
    #[error("The chosen block explorer service is not available for Sepolia Network")]
    SepoliaNotSupported,
    #[error("Custom network is not recognized by block explorer service")]
    UnrecognizedNetwork,
}

pub fn block_explorer_link_if_allowed<T: OutputLink>(
    result: &anyhow::Result<T>,
    output_format: OutputFormat,
    chain_id: Felt,
    show_links: bool,
    explorer: Option<Service>,
) -> Option<String> {
    if !show_links {
        return None;
    }
    if output_format != OutputFormat::Human {
        return None;
    }
    let Ok(response) = result else {
        return None;
    };
    let Ok(network) = chain_id.try_into() else {
        return None;
    };

    if let Ok(provider) = explorer.unwrap_or_default().as_provider(network) {
        return Some(response.print_links(provider));
    }

    None
}
