use super::print::OutputFormat;
use crate::helpers::block_explorer::Service;

pub trait OutputLink {
    const TITLE: &'static str;

    fn format_links(&self, service: Service) -> String;
}

pub fn print_block_explorer_link_if_allowed<T: OutputLink>(
    result: &anyhow::Result<T>,
    output_format: OutputFormat,
    explorer_service: Option<Service>,
) {
    if let (Ok(response), OutputFormat::Human) = (result, output_format) {
        let title = T::TITLE;
        let urls = response.format_links(explorer_service.unwrap_or_default());

        println!("\nTo see {title} details, visit:\n{urls}");
    }
}
