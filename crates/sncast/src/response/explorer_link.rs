use super::print::OutputFormat;
use crate::helpers::block_explorer::{LinkProvider, Service};
use starknet_types_core::felt::Felt;

pub trait OutputLink {
    const TITLE: &'static str;

    fn format_links(&self, provider: Box<dyn LinkProvider>) -> String;

    fn print_links(&self, provider: Box<dyn LinkProvider>) {
        let title = Self::TITLE;
        let links = self.format_links(provider);
        println!("\nTo see {title} details, visit:\n{links}");
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ExplorerError {
    #[error("The chosen block explorer service is not available for Sepolia Network")]
    SepoliaNotSupported,
    #[error("Custom network is not recognized by block explorer service")]
    UnrecognizedNetwork,
}

pub fn print_block_explorer_link_if_allowed<T: OutputLink>(
    result: &anyhow::Result<T>,
    output_format: OutputFormat,
    chain_id: Felt,
    show_links: bool,
    explorer: Option<Service>,
) {
    if !show_links {
        return;
    }
    if output_format != OutputFormat::Human {
        return;
    }
    let Ok(response) = result else {
        return;
    };
    let Ok(network) = chain_id.try_into() else {
        return;
    };

    if let Ok(provider) = explorer.unwrap_or_default().as_provider(network) {
        response.print_links(provider);
    }
}
