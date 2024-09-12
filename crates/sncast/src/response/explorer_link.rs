use super::print::OutputFormat;
use crate::helpers::block_explorer::{LinkProvider, Service};
use shared::print::print_as_warning;
use starknet::core::types::Felt;

pub trait OutputLink {
    const TITLE: &'static str;

    fn format_links(&self, provider: Box<dyn LinkProvider>) -> String;
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
    explorer_service: Option<Service>,
) {
    if let (Ok(response), OutputFormat::Human) = (result, output_format) {
        let Ok(network) = chain_id.try_into() else {
            print_as_warning(&ExplorerError::UnrecognizedNetwork.into());
            return;
        };

        match explorer_service.unwrap_or_default().as_provider(network) {
            Ok(provider) => {
                let title = T::TITLE;
                let urls = response.format_links(provider);

                println!("\nTo see {title} details, visit:\n{urls}");
            }
            Err(err) => print_as_warning(&err.into()),
        }
    }
}
