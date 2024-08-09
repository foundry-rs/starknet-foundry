use super::print::OutputFormat;
use crate::helpers::block_explorer::{LinkProvider, Service};

pub trait OutputLink {
    const TITLE: &'static str;

    fn format_links(&self, provider: Box<dyn LinkProvider>) -> String;
}

pub fn print_block_explorer_link_if_allowed<T: OutputLink>(
    result: &anyhow::Result<T>,
    output_format: OutputFormat,
    explorer_service: Option<Service>,
) {
    if let (Ok(response), OutputFormat::Human) = (result, output_format) {
        let title = T::TITLE;
        let urls = response.format_links(explorer_service.unwrap_or_default().as_provider());

        println!("\nTo see {title} details, visit:\n{urls}");
    }
}
